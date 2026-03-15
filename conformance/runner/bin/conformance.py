#!/usr/bin/env python3
from __future__ import annotations

import argparse
import difflib
import json
import os
import pty
import re
import select
import shutil
import shlex
import subprocess
import sys
import tempfile
import time
import tomllib
import uuid
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
CONF_ROOT = ROOT / "conformance"
CASES_ROOT = CONF_ROOT / "cases"
ARTIFACTS_ROOT = CONF_ROOT / "artifacts"

IMPLEMENTATIONS = ("opendoas", "opendoas-rs")
CASE_TIMEOUT_SECS = 20
STREAM_LABELS = ("stdout", "stderr", "tty", "syslog")
CASE_MODES = {"ignore", "exact", "contains", "empty"}


def sh(cmd: list[str], *, input_data: bytes | None = None, check: bool = True) -> subprocess.CompletedProcess:
    return subprocess.run(cmd, input=input_data, capture_output=True, check=check)


def image_name(implementation: str, variant: str) -> str:
    return f"localhost/opendoas-rs-conformance-{implementation}:{variant}"


def parse_variant(variant: str) -> tuple[str, str]:
    try:
        auth_backend, timestamp = variant.split("-", 1)
    except ValueError as exc:
        raise SystemExit(f"invalid variant {variant!r}, expected <auth>-<on|off>") from exc
    if auth_backend not in {"shadow", "pam", "plain", "none"}:
        raise SystemExit(f"unsupported auth backend {auth_backend!r}")
    if timestamp not in {"on", "off"}:
        raise SystemExit(f"unsupported timestamp mode {timestamp!r}")
    return auth_backend, timestamp


def build_image(implementation: str, variant: str, rebuild: bool) -> None:
    tag = image_name(implementation, variant)
    exists = subprocess.run(["podman", "image", "exists", tag]).returncode == 0
    if exists and not rebuild:
        return

    auth_backend, timestamp = parse_variant(variant)
    if implementation == "opendoas":
        containerfile = CONF_ROOT / "images" / "opendoas" / "Containerfile"
        if auth_backend == "plain":
            auth_backend = "shadow"
        elif auth_backend == "none":
            auth_backend = "shadow"
    elif implementation == "opendoas-rs":
        containerfile = CONF_ROOT / "images" / "opendoas-rs" / "Containerfile"
        if auth_backend == "shadow":
            auth_backend = "plain"
    else:
        raise SystemExit(f"unknown implementation {implementation!r}")

    cmd = [
        "podman",
        "build",
        "--network=host",
        "-t",
        tag,
        "-f",
        str(containerfile),
        "--build-arg",
        f"AUTH_BACKEND={auth_backend}",
        "--build-arg",
        f"TIMESTAMP={timestamp}",
        str(ROOT),
    ]
    subprocess.run(cmd, check=True)


def image_exists(implementation: str, variant: str) -> bool:
    return subprocess.run(["podman", "image", "exists", image_name(implementation, variant)]).returncode == 0


def parse_case_env(case_dir: Path) -> dict[str, str]:
    env_path = case_dir / "case.env"
    if not env_path.exists():
        return {}

    data: dict[str, str] = {}
    for line_no, raw_line in enumerate(env_path.read_text().splitlines(), start=1):
        line = raw_line.strip()
        if not line or line.startswith("#"):
            continue
        if "=" not in line:
            raise SystemExit(f"{env_path}:{line_no}: expected KEY=VALUE")
        key, value = line.split("=", 1)
        key = key.strip()
        value = value.strip()
        if value[:1] == value[-1:] and value[:1] in {'"', "'"}:
            value = value[1:-1]
        data[key] = value
    return data


def parse_bool(value: bool | str | int, *, field: str, case_dir: Path) -> bool:
    if isinstance(value, bool):
        return value
    if isinstance(value, int):
        return bool(value)

    lowered = str(value).strip().lower()
    if lowered in {"1", "true", "yes", "on"}:
        return True
    if lowered in {"0", "false", "no", "off"}:
        return False
    raise SystemExit(f"{case_dir}: invalid boolean for {field}: {value!r}")


def expected_file_for(case_dir: Path, label: str) -> Path | None:
    def usable(path: Path) -> Path | None:
        if not path.exists():
            return None
        text = path.read_text(encoding="utf-8", errors="replace")
        if text and not text.strip("\n"):
            return None
        return path

    candidate = case_dir / f"expect.{label}"
    candidate = usable(candidate)
    if candidate is not None:
        return candidate
    if label == "syslog":
        candidate = case_dir / "log"
    else:
        candidate = case_dir / label
    return usable(candidate)


def default_mode_for(label: str, expected_path: Path | None, *, legacy_case: bool) -> str:
    if expected_path is not None:
        return "exact"
    if legacy_case and label in {"stdout", "stderr"}:
        return "empty"
    return "ignore"


def parse_compare_mode(mode: str, *, field: str, case_dir: Path) -> str:
    normalized = mode.strip().lower()
    if normalized not in CASE_MODES:
        raise SystemExit(f"{case_dir}: invalid compare mode for {field}: {mode!r}")
    return normalized


def resolve_variant(implementation: str, variant: str, rebuild: bool) -> tuple[str, bool]:
    if image_exists(implementation, variant) and not rebuild:
        return variant, False
    try:
        build_image(implementation, variant, rebuild)
        return variant, False
    except subprocess.CalledProcessError:
        if variant.endswith("-on"):
            fallback = f"{variant.rsplit('-', 1)[0]}-off"
            if image_exists(implementation, fallback):
                return fallback, True
            build_image(implementation, fallback, rebuild)
            return fallback, True
        raise


def load_case(case_dir: Path) -> dict:
    case_toml = case_dir / "case.toml"
    legacy_case = not case_toml.exists() and (case_dir / "case.env").exists()
    case_env = parse_case_env(case_dir)
    case: dict = {}
    if case_toml.exists():
        data = tomllib.loads(case_toml.read_text())
        case.update(data.get("case", {}))

    case.setdefault("name", case_dir.name)
    case.setdefault("compare", "baseline")
    case["run_as"] = case.get("run_as") or case_env.get("RUN_AS") or case_env.get("ACTOR") or "alice"
    case["tty"] = parse_bool(case.get("tty", case_env.get("TTY", False)), field="tty", case_dir=case_dir)
    case["install_conf"] = parse_bool(
        case.get("install_conf", True),
        field="install_conf",
        case_dir=case_dir,
    )

    if "oracle_variant" not in case and "VARIANT" in case_env:
        case["oracle_variant"] = case_env["VARIANT"]
    if "subject_variant" not in case and "VARIANT" in case_env:
        case["subject_variant"] = case_env["VARIANT"]
    if "oracle_variant" not in case or "subject_variant" not in case:
        auth = case_env.get("AUTH", "plain")
        timestamp = case_env.get("TIMESTAMP", "off")
        if "oracle_variant" not in case:
            case["oracle_variant"] = f"{auth}-{timestamp}"
        if "subject_variant" not in case:
            case["subject_variant"] = f"{auth}-{timestamp}"
    case.setdefault("oracle_variant", "shadow-off")
    case.setdefault("subject_variant", "plain-off")

    compare_modes: dict[str, str] = {}
    expected_files: dict[str, Path | None] = {}
    for label in STREAM_LABELS:
        expected_path = expected_file_for(case_dir, label)
        expected_files[label] = expected_path
        legacy_field = f"COMPARE_{label.upper()}"
        if label == "syslog":
            mode_field = "EXPECT_LOG_MODE"
        else:
            mode_field = f"EXPECT_{label.upper()}_MODE"
        raw_mode = case_env.get(mode_field) or case_env.get(legacy_field) or default_mode_for(
            label,
            expected_path,
            legacy_case=legacy_case,
        )
        compare_modes[label] = parse_compare_mode(raw_mode, field=mode_field, case_dir=case_dir)

    capture_syslog = case.get("capture_syslog")
    if capture_syslog is None:
        capture_syslog = case_env.get("CAPTURE_SYSLOG")
    if capture_syslog is None:
        case["capture_syslog"] = compare_modes["syslog"] != "ignore"
    else:
        case["capture_syslog"] = parse_bool(capture_syslog, field="capture_syslog", case_dir=case_dir)

    expected_exit = None
    expect_exit_path = case_dir / "expect.exit"
    if expect_exit_path.exists():
        expected_exit = int(expect_exit_path.read_text().strip())
    elif "EXPECT_EXIT" in case_env:
        expected_exit = int(case_env["EXPECT_EXIT"])

    case["pam_profile"] = case_env.get("PAM_PROFILE")
    case["legacy_case"] = legacy_case
    case["case_env"] = case_env
    case["compare_modes"] = compare_modes
    case["expected_files"] = expected_files
    case["expected_exit"] = expected_exit
    if "invoke.sh" not in {p.name for p in case_dir.iterdir()}:
        raise SystemExit(f"{case_dir}: missing invoke.sh")
    return case


def rel_case_id(case_dir: Path) -> str:
    return case_dir.relative_to(CASES_ROOT).as_posix()


def materialize_rootfs(image: str) -> Path:
    rootfs = Path(tempfile.mkdtemp(prefix="conformance-rootfs-"))
    container_name = f"conformance-export-{uuid.uuid4().hex}"
    subprocess.run(
        ["podman", "create", "--name", container_name, image, "/bin/true"],
        check=True,
        stdout=subprocess.DEVNULL,
    )
    proc = subprocess.Popen(
        ["podman", "export", container_name],
        stdout=subprocess.PIPE,
    )
    try:
        assert proc.stdout is not None
        subprocess.run(["tar", "-C", str(rootfs), "-xf", "-"], stdin=proc.stdout, check=True)
        proc.stdout.close()
        if proc.wait() != 0:
            shutil.rmtree(rootfs, ignore_errors=True)
            raise subprocess.CalledProcessError(proc.returncode, proc.args)
        return rootfs
    finally:
        subprocess.run(
            ["podman", "rm", "-f", container_name],
            check=False,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        )


def install_case(rootfs: Path, case_dir: Path, case: dict) -> None:
    case_target = rootfs / "case"
    if case_target.exists():
        shutil.rmtree(case_target)
    shutil.copytree(case_dir, case_target)
    for path in case_target.glob("*.sh"):
        path.chmod(0o755)
    if case["install_conf"] and (case_dir / "doas.conf").exists():
        conf_target = rootfs / "etc" / "doas.conf"
        conf_target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(case_dir / "doas.conf", conf_target)
        conf_target.chmod(0o400)
    pam_target = rootfs / "etc" / "pam.d" / "doas"
    if (case_dir / "pam.doas").exists():
        pam_target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(case_dir / "pam.doas", pam_target)
    elif case.get("pam_profile"):
        pam_source = CONF_ROOT / "fixtures" / "pam" / f"doas-{case['pam_profile']}"
        pam_target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(pam_source, pam_target)


def read_rootfs_file(rootfs: Path, path: str) -> str:
    target = rootfs / path.lstrip("/")
    if not target.exists():
        return ""
    return target.read_text(encoding="utf-8", errors="replace")


def chroot_command(rootfs: Path, user: str, capture_syslog: bool, command_path: str = "/case/invoke.sh") -> list[str]:
    script = r'''
rootfs=$1
user=$2
capture=$3
mkdir -p "$rootfs/proc" "$rootfs/dev"
: > "$rootfs/dev/null"
: > "$rootfs/dev/tty"
mount -t proc proc "$rootfs/proc"
mount --bind /dev/null "$rootfs/dev/null"
if [ -c /dev/tty ]; then
    mount --bind /dev/tty "$rootfs/dev/tty" 2>/dev/null || true
fi
cleanup() {
    umount "$rootfs/dev/tty" 2>/dev/null || true
    umount "$rootfs/dev/null" 2>/dev/null || true
    umount "$rootfs/proc" 2>/dev/null || true
}
trap cleanup EXIT
if [ -f "$rootfs/case/setup.sh" ]; then
    chroot "$rootfs" /bin/sh -eu /case/setup.sh
fi
if [ "$capture" = "1" ]; then
    chroot "$rootfs" /bin/sh -lc 'rm -f /tmp/conformance-syslog.log; syslogd -O /tmp/conformance-syslog.log'
fi
if [ "$user" = "root" ]; then
    exec chroot "$rootfs" /bin/sh "$4"
else
    exec chroot "$rootfs" /bin/sh -lc "su -s /bin/sh $user -c $4"
fi
'''
    return [
        "unshare",
        "--user",
        "--map-root-user",
        "--mount",
        "--pid",
        "--fork",
        "/bin/sh",
        "-ceu",
        script,
        "_",
        str(rootfs),
        user,
        "1" if capture_syslog else "0",
        command_path,
    ]


def run_non_tty(rootfs: Path, user: str, capture_syslog: bool, stdin_data: bytes | None) -> tuple[int, str, str, str]:
    try:
        proc = subprocess.run(
            chroot_command(rootfs, user, capture_syslog),
            input=stdin_data,
            capture_output=True,
            check=False,
            timeout=CASE_TIMEOUT_SECS,
        )
    except subprocess.TimeoutExpired as exc:
        stdout = (exc.stdout or b"").decode("utf-8", "replace")
        stderr = (exc.stderr or b"").decode("utf-8", "replace")
        return (124, stdout, stderr, "")
    return (
        proc.returncode,
        proc.stdout.decode("utf-8", "replace"),
        proc.stderr.decode("utf-8", "replace"),
        "",
    )


def run_tty(rootfs: Path, user: str, capture_syslog: bool, stdin_data: bytes | None) -> tuple[int, str, str, str]:
    master_fd, slave_fd = pty.openpty()
    proc = subprocess.Popen(
        chroot_command(rootfs, user, capture_syslog),
        stdin=slave_fd,
        stdout=slave_fd,
        stderr=slave_fd,
        close_fds=True,
    )
    os.close(slave_fd)
    transcript = bytearray()
    sent = False
    deadline = time.monotonic() + CASE_TIMEOUT_SECS
    try:
        while True:
            if time.monotonic() > deadline:
                proc.kill()
                proc.wait(timeout=5)
                transcript.extend(b"\n[conformance timeout]\n")
                return (124, "", "", transcript.decode("utf-8", "replace"))
            if stdin_data and not sent and proc.poll() is None:
                try:
                    os.write(master_fd, stdin_data)
                    sent = True
                except OSError:
                    pass
            ready, _, _ = select.select([master_fd], [], [], 0.05)
            if ready:
                try:
                    chunk = os.read(master_fd, 4096)
                except OSError:
                    chunk = b""
                if chunk:
                    transcript.extend(chunk)
            if proc.poll() is not None and not ready:
                break
    finally:
        os.close(master_fd)
    return proc.returncode, "", "", transcript.decode("utf-8", "replace")


def variant_rebuild_script(implementation: str, requested_variant: str) -> str:
    auth_backend, timestamp = parse_variant(requested_variant)
    if implementation == "opendoas" and auth_backend == "plain":
        auth_backend = "shadow"
    if implementation == "opendoas-rs" and auth_backend == "shadow":
        auth_backend = "plain"

    if implementation == "opendoas":
        return f"""
set -eu
cd /src/OpenDoas
make clean >/dev/null 2>&1 || true
auth_flags=--without-pam
if [ "{auth_backend}" = "pam" ]; then auth_flags=--without-shadow; fi
ts_flag=
if [ "{timestamp}" = "on" ]; then ts_flag=--with-timestamp; fi
./configure --prefix=/usr $auth_flags $ts_flag
make
make install
chmod 4755 /usr/bin/doas
"""

    return f"""
set -eu
cd /src/OpenDoas-rs
features=auth-plain
case "{auth_backend}" in
    none) features=auth-none ;;
    plain) features=auth-plain ;;
    pam) features=auth-pam ;;
    *) echo unsupported auth backend >&2; exit 1 ;;
esac
AUTH_MODE="{auth_backend}" OPENDOAS_RS_TIMESTAMP="{timestamp}" cargo build --release --no-default-features --features "$features"
install -Dm4755 target/release/OpenDoas-rs /usr/bin/doas
"""


def run_case_in_container(
    image: str,
    implementation: str,
    case_dir: Path,
    result_dir: Path,
    case: dict,
    *,
    requested_variant: str,
    mutate_variant: bool,
) -> None:
    env_items = dict(case["case_env"])
    env_items["RUN_AS"] = case["run_as"]
    env_items["TTY"] = "1" if case["tty"] else "0"
    env_items["CAPTURE_SYSLOG"] = "1" if case["capture_syslog"] else "0"
    env_items["INSTALL_CONF"] = "1" if case["install_conf"] else "0"
    if case.get("pam_profile"):
        env_items["PAM_PROFILE"] = case["pam_profile"]

    mounts = [
        "--volume",
        f"{case_dir}:/case:Z",
        "--volume",
        f"{result_dir}:/results:Z",
        "--volume",
        f"{CONF_ROOT / 'fixtures'}:/conformance/fixtures:ro,Z",
        "--volume",
        f"{CONF_ROOT / 'runner' / 'bin'}:/conformance/runner/bin:ro,Z",
    ]
    env_args: list[str] = []
    for key, value in sorted(env_items.items()):
        env_args.extend(["--env", f"{key}={value}"])
    script = []
    if mutate_variant:
        script.append(variant_rebuild_script(implementation, requested_variant))
    script.append(f'exec /bin/sh /conformance/runner/bin/container-run.sh "{implementation}" /case /results')

    proc = subprocess.run(
        [
            "podman",
            "run",
            "--rm",
            *env_args,
            *mounts,
            image,
            "/bin/sh",
            "-ceu",
            "\n".join(script),
        ],
        capture_output=True,
        check=False,
        timeout=CASE_TIMEOUT_SECS * 8,
    )
    if proc.returncode != 0:
        stderr = proc.stderr.decode("utf-8", "replace")
        stdout = proc.stdout.decode("utf-8", "replace")
        raise subprocess.CalledProcessError(proc.returncode, proc.args, output=stdout, stderr=stderr)


def rebuild_variant_in_rootfs(rootfs: Path, implementation: str, requested_variant: str) -> None:
    auth_backend, timestamp = parse_variant(requested_variant)
    if implementation == "opendoas-rs" and auth_backend == "shadow":
        auth_backend = "plain"
    if implementation == "opendoas" and auth_backend == "plain":
        auth_backend = "shadow"

    if implementation == "opendoas":
        script = f"""
set -eu
cd /src/OpenDoas
make clean >/dev/null 2>&1 || true
auth_flags=--without-pam
if [ "{auth_backend}" = "pam" ]; then auth_flags=--without-shadow; fi
ts_flag=
if [ "{timestamp}" = "on" ]; then ts_flag=--with-timestamp; fi
./configure --prefix=/usr $auth_flags $ts_flag
make
make install
chmod 4755 /usr/bin/doas
"""
    else:
        script = f"""
set -eu
cd /src/OpenDoas-rs
features=auth-plain
case "{auth_backend}" in
    none) features=auth-none ;;
    plain) features=auth-plain ;;
    pam) features=auth-pam ;;
    *) echo unsupported auth backend >&2; exit 1 ;;
esac
AUTH_MODE="{auth_backend}" OPENDOAS_RS_TIMESTAMP="{timestamp}" cargo build --release --no-default-features --features "$features"
install -Dm4755 target/release/OpenDoas-rs /usr/bin/doas
"""

    script_path = rootfs / "tmp" / f"conformance-variant-{uuid.uuid4().hex}.sh"
    script_path.parent.mkdir(parents=True, exist_ok=True)
    script_path.write_text(script)
    script_path.chmod(0o755)
    proc = subprocess.run(
        chroot_command(rootfs, "root", False, f"/tmp/{script_path.name}"),
        capture_output=True,
        check=False,
        timeout=CASE_TIMEOUT_SECS * 4,
    )
    if proc.returncode != 0:
        raise subprocess.CalledProcessError(
            proc.returncode,
            proc.args,
            output=proc.stdout,
            stderr=proc.stderr,
        )
    script_path.unlink(missing_ok=True)


def normalize_text(label: str, text: str) -> str:
    text = text.replace("\r\n", "\n")
    if label == "syslog":
        lines = []
        for line in text.splitlines():
            line = re.sub(r"^(?:<\d+>)?[A-Z][a-z]{2}\s+\d+\s+\d\d:\d\d:\d\d\s+\S+\s+", "", line)
            if re.match(r"^\+\s+\S+\s+\S+:\S+$", line):
                continue
            lines.append(line)
        text = "\n".join(lines)
        if lines:
            text += "\n"
    return text


def normalize_expected_text(label: str, text: str) -> str:
    text = normalize_text(label, text)
    if text and not text.strip("\n"):
        return ""
    if text.endswith("\n"):
        return text.rstrip("\n") + "\n"
    return text


def write_result(implementation: str, case_id: str, result: dict) -> None:
    out_dir = ARTIFACTS_ROOT / "runs" / implementation / case_id
    out_dir.mkdir(parents=True, exist_ok=True)
    for label in ("stdout", "stderr", "tty", "syslog"):
        (out_dir / label).write_text(result[label])
    (out_dir / "result.json").write_text(json.dumps(result, indent=2, sort_keys=True))


def write_baseline(case_id: str, result: dict) -> None:
    out_path = ARTIFACTS_ROOT / "baselines" / f"{case_id.replace('/', '__')}.json"
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(json.dumps(result, indent=2, sort_keys=True))


def compare_texts(left: str, right: str, *, fromfile: str, tofile: str) -> str:
    return "".join(
        difflib.unified_diff(
            left.splitlines(keepends=True),
            right.splitlines(keepends=True),
            fromfile=fromfile,
            tofile=tofile,
        )
    )


def validate_stream(case_id: str, implementation: str, label: str, mode: str, expected: str | None, actual: str) -> list[str]:
    prefix = f"{case_id}: {implementation} {label}"
    if mode == "ignore":
        return []
    if expected is None and mode in {"exact", "contains"}:
        return []
    if mode == "exact":
        assert expected is not None
        if expected == actual:
            return []
        diff = compare_texts(expected, actual, fromfile=f"expected/{label}", tofile=f"{implementation}/{label}")
        return [f"{prefix} mismatch\n{diff}"]
    if mode == "contains":
        assert expected is not None
        if expected in actual:
            return []
        return [f"{prefix} missing expected content\nexpected substring:\n{expected}\nactual:\n{actual}"]
    if mode == "empty":
        if not actual:
            return []
        return [f"{prefix} expected empty output\nactual:\n{actual}"]
    raise AssertionError(f"unexpected compare mode: {mode}")


def validate_result(case_id: str, implementation: str, case: dict, result: dict) -> list[str]:
    problems: list[str] = []
    expected_exit = case["expected_exit"]
    if expected_exit is not None and result["exit_code"] != expected_exit:
        problems.append(
            f"{case_id}: {implementation} exit code {result['exit_code']} != expected {expected_exit}"
        )

    for label in STREAM_LABELS:
        expected_path = case["expected_files"][label]
        expected_text = None
        if expected_path is not None:
            expected_text = normalize_expected_text(label, expected_path.read_text(encoding="utf-8", errors="replace"))
            if case["compare_modes"][label] == "contains":
                expected_text = expected_text.rstrip("\n")
        problems.extend(
            validate_stream(
                case_id,
                implementation,
                label,
                case["compare_modes"][label],
                expected_text,
                result[label],
            )
        )
    return problems


def compare_results(case_id: str, case: dict, oracle: dict, subject: dict) -> list[str]:
    problems = validate_result(case_id, "opendoas", case, oracle)
    problems.extend(validate_result(case_id, "opendoas-rs", case, subject))

    if case["expected_exit"] is None and oracle["exit_code"] != subject["exit_code"]:
        problems.append(f"{case_id}: exit code {subject['exit_code']} != {oracle['exit_code']}")

    for label in STREAM_LABELS:
        mode = case["compare_modes"][label]
        expected_path = case["expected_files"][label]
        if mode == "ignore":
            if expected_path is None and case["compare"] == "baseline" and oracle[label] != subject[label]:
                diff = compare_texts(
                    oracle[label],
                    subject[label],
                    fromfile=f"oracle/{label}",
                    tofile=f"subject/{label}",
                )
                problems.append(f"{case_id}: mismatch in {label}\n{diff}")
            continue
        if mode == "empty":
            continue
        if expected_path is not None:
            if mode == "exact":
                continue
            if mode == "contains":
                continue
        if mode == "contains":
            if oracle[label] not in subject[label]:
                problems.append(
                    f"{case_id}: mismatch in {label}\nexpected substring from oracle:\n{oracle[label]}\nsubject:\n{subject[label]}"
                )
            continue
        if oracle[label] != subject[label]:
            diff = compare_texts(
                oracle[label],
                subject[label],
                fromfile=f"oracle/{label}",
                tofile=f"subject/{label}",
            )
            problems.append(f"{case_id}: mismatch in {label}\n{diff}")
    return problems


def execute_case(implementation: str, case_dir: Path, case: dict, rebuild: bool) -> dict:
    requested_variant = case["oracle_variant"] if implementation == "opendoas" else case["subject_variant"]
    image_variant, mutate_variant = resolve_variant(implementation, requested_variant, rebuild)
    image = image_name(implementation, image_variant)
    result_dir = Path(tempfile.mkdtemp(prefix="conformance-results-"))
    try:
        run_case_in_container(
            image,
            implementation,
            case_dir,
            result_dir,
            case,
            requested_variant=requested_variant,
            mutate_variant=mutate_variant,
        )
        exit_code = int((result_dir / "exit_code").read_text().strip())
        stdout = (result_dir / "stdout").read_text(encoding="utf-8", errors="replace")
        stderr = (result_dir / "stderr").read_text(encoding="utf-8", errors="replace")
        tty = (result_dir / "tty").read_text(encoding="utf-8", errors="replace")
        syslog = (result_dir / "syslog").read_text(encoding="utf-8", errors="replace")
        result = {
            "implementation": implementation,
            "variant": requested_variant,
            "case": rel_case_id(case_dir),
            "exit_code": exit_code,
            "stdout": normalize_text("stdout", stdout),
            "stderr": normalize_text("stderr", stderr),
            "tty": normalize_text("tty", tty),
            "syslog": normalize_text("syslog", syslog),
        }
        write_result(implementation, rel_case_id(case_dir), result)
        return result
    finally:
        try:
            subprocess.run(
                [
                    "python3",
                    "-c",
                    "from pathlib import Path; import shutil, sys; shutil.rmtree(Path(sys.argv[1]), ignore_errors=True)",
                    str(result_dir),
                ],
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
                timeout=CASE_TIMEOUT_SECS,
                check=False,
            )
        except subprocess.TimeoutExpired:
            pass


def collect_cases(paths: list[str]) -> list[Path]:
    def is_case_dir(path: Path) -> bool:
        return path.is_dir() and ((path / "case.toml").exists() or (path / "case.env").exists())

    if not paths:
        result = [path for path in CASES_ROOT.rglob("*") if is_case_dir(path)]
        return sorted(dict.fromkeys(result))
    result: list[Path] = []
    for value in paths:
        path = (ROOT / value).resolve()
        if is_case_dir(path):
            result.append(path)
        elif path.is_dir():
            result.extend(sorted(candidate for candidate in path.rglob("*") if is_case_dir(candidate)))
        else:
            raise SystemExit(f"{value}: no such case path")
    return sorted(dict.fromkeys(result))


def cmd_build(args: argparse.Namespace) -> int:
    for implementation in args.implementations:
        for variant in args.variants:
            build_image(implementation, variant, rebuild=args.rebuild)
    return 0


def cmd_run_case(args: argparse.Namespace) -> int:
    case_dir = (ROOT / args.case).resolve()
    case = load_case(case_dir)
    oracle = execute_case("opendoas", case_dir, case, rebuild=args.rebuild)
    write_baseline(rel_case_id(case_dir), oracle)
    subject = execute_case("opendoas-rs", case_dir, case, rebuild=args.rebuild)
    problems = compare_results(rel_case_id(case_dir), case, oracle, subject)
    if problems:
        print("\n\n".join(problems), file=sys.stderr)
        return 1
    print(f"PASS {rel_case_id(case_dir)}")
    return 0


def cmd_run_suite(args: argparse.Namespace) -> int:
    cases = collect_cases(args.paths)
    failures: list[str] = []
    for case_dir in cases:
        case = load_case(case_dir)
        oracle = execute_case("opendoas", case_dir, case, rebuild=args.rebuild)
        write_baseline(rel_case_id(case_dir), oracle)
        subject = execute_case("opendoas-rs", case_dir, case, rebuild=args.rebuild)
        case_failures = compare_results(rel_case_id(case_dir), case, oracle, subject)
        failures.extend(case_failures)
        if not case_failures:
            print(f"PASS {rel_case_id(case_dir)}")
    if failures:
        print("\n\n".join(failures), file=sys.stderr)
        return 1
    print(f"PASS {len(cases)} cases")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser()
    sub = parser.add_subparsers(dest="cmd", required=True)

    p_build = sub.add_parser("build")
    p_build.add_argument("--rebuild", action="store_true")
    p_build.add_argument("--implementations", nargs="+", default=list(IMPLEMENTATIONS))
    p_build.add_argument("--variants", nargs="+", default=["shadow-off", "shadow-on", "plain-off", "plain-on", "pam-off", "pam-on"])
    p_build.set_defaults(func=cmd_build)

    p_case = sub.add_parser("run-case")
    p_case.add_argument("case")
    p_case.add_argument("--rebuild", action="store_true")
    p_case.set_defaults(func=cmd_run_case)

    p_suite = sub.add_parser("run-suite")
    p_suite.add_argument("paths", nargs="*")
    p_suite.add_argument("--rebuild", action="store_true")
    p_suite.set_defaults(func=cmd_run_suite)

    args = parser.parse_args()
    return args.func(args)


if __name__ == "__main__":
    raise SystemExit(main())
