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
    data = tomllib.loads((case_dir / "case.toml").read_text())
    case = data.get("case", {})
    case.setdefault("name", case_dir.name)
    case.setdefault("compare", "baseline")
    case.setdefault("run_as", "alice")
    case.setdefault("tty", False)
    case.setdefault("install_conf", True)
    case.setdefault("capture_syslog", False)
    case.setdefault("oracle_variant", "shadow-off")
    case.setdefault("subject_variant", "plain-off")
    if "invoke.sh" not in {p.name for p in case_dir.iterdir()}:
        raise SystemExit(f"{case_dir}: missing invoke.sh")
    return case


def rel_case_id(case_dir: Path) -> str:
    return case_dir.relative_to(CASES_ROOT).as_posix()


def materialize_rootfs(image: str) -> Path:
    rootfs = Path(tempfile.mkdtemp(prefix="conformance-rootfs-"))
    script = 'rootfs=$(podman image mount "$1"); tar -C "$rootfs" -cf - .; podman image unmount "$1" >/dev/null'
    proc = subprocess.Popen(
        ["podman", "unshare", "/bin/sh", "-c", script, "_", image],
        stdout=subprocess.PIPE,
    )
    assert proc.stdout is not None
    subprocess.run(["tar", "-C", str(rootfs), "-xf", "-"], stdin=proc.stdout, check=True)
    proc.stdout.close()
    if proc.wait() != 0:
        shutil.rmtree(rootfs, ignore_errors=True)
        raise subprocess.CalledProcessError(proc.returncode, proc.args)
    return rootfs


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
            line = re.sub(r"^[A-Z][a-z]{2}\s+\d+\s+\d\d:\d\d:\d\d\s+\S+\s+", "", line)
            lines.append(line)
        text = "\n".join(lines)
        if lines:
            text += "\n"
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


def compare_results(case_id: str, oracle: dict, subject: dict) -> list[str]:
    problems: list[str] = []
    for label in ("exit_code", "stdout", "stderr", "tty", "syslog"):
        if oracle[label] != subject[label]:
            if label == "exit_code":
                problems.append(f"{case_id}: exit code {subject[label]} != {oracle[label]}")
            else:
                diff = "".join(
                    difflib.unified_diff(
                        oracle[label].splitlines(keepends=True),
                        subject[label].splitlines(keepends=True),
                        fromfile=f"oracle/{label}",
                        tofile=f"subject/{label}",
                    )
                )
                problems.append(f"{case_id}: mismatch in {label}\n{diff}")
    return problems


def execute_case(implementation: str, case_dir: Path, case: dict, rebuild: bool) -> dict:
    requested_variant = case["oracle_variant"] if implementation == "opendoas" else case["subject_variant"]
    image_variant, mutate_variant = resolve_variant(implementation, requested_variant, rebuild)
    image = image_name(implementation, image_variant)
    rootfs = materialize_rootfs(image)
    try:
        install_case(rootfs, case_dir, case)
        if mutate_variant:
            rebuild_variant_in_rootfs(rootfs, implementation, requested_variant)
        stdin_data = None
        if (case_dir / "stdin.txt").exists():
            stdin_data = (case_dir / "stdin.txt").read_bytes()
        if case["tty"]:
            exit_code, stdout, stderr, tty = run_tty(rootfs, case["run_as"], case["capture_syslog"], stdin_data)
        else:
            exit_code, stdout, stderr, tty = run_non_tty(rootfs, case["run_as"], case["capture_syslog"], stdin_data)
        syslog = read_rootfs_file(rootfs, "/tmp/conformance-syslog.log") if case["capture_syslog"] else ""
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
                    str(rootfs),
                ],
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
                timeout=CASE_TIMEOUT_SECS,
                check=False,
            )
        except subprocess.TimeoutExpired:
            pass


def collect_cases(paths: list[str]) -> list[Path]:
    if not paths:
        return sorted(path.parent for path in CASES_ROOT.rglob("case.toml"))
    result: list[Path] = []
    for value in paths:
        path = (ROOT / value).resolve()
        if path.is_dir() and (path / "case.toml").exists():
            result.append(path)
        elif path.is_dir():
            result.extend(sorted(p.parent for p in path.rglob("case.toml")))
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
    problems = compare_results(rel_case_id(case_dir), oracle, subject)
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
        failures.extend(compare_results(rel_case_id(case_dir), oracle, subject))
        if not failures:
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
