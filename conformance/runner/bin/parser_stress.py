#!/usr/bin/env python3
from __future__ import annotations

import argparse
import importlib.util
import random
import shutil
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
GENERATED_ROOT = ROOT / "conformance" / "cases" / ".generated" / "parser-stress"
RUNNER_PATH = Path(__file__).resolve().with_name("conformance.py")


def load_runner():
    spec = importlib.util.spec_from_file_location("conformance_runner", RUNNER_PATH)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"failed to load {RUNNER_PATH}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def valid_lines(rng: random.Random) -> list[bytes]:
    commands = [
        b"permit nopass alice as root cmd /usr/bin/id",
        b"permit bob as root cmd /usr/bin/true",
        b"permit keepenv alice as root cmd /usr/bin/env",
        b"permit setenv { PATH=/usr/bin HOME=/root } alice as root cmd /usr/bin/env",
        b"deny alice as root cmd /usr/bin/false",
        b"permit :wheel as root",
    ]
    count = rng.randint(1, 4)
    selected = [rng.choice(commands) for _ in range(count)]
    rendered: list[bytes] = []
    for line in selected:
        indent = b" " * rng.randint(0, 2) + b"\t" * rng.randint(0, 1)
        suffix = b""
        if rng.random() < 0.5:
            suffix = b"  # generated " + str(rng.randint(0, 999)).encode()
        rendered.append(indent + line + suffix)
    return rendered


def case_bytes(index: int, rng: random.Random) -> bytes:
    mode = index % 10
    if mode == 0:
        return b"\n".join(valid_lines(rng)) + b"\n"
    if mode == 1:
        base = valid_lines(rng)
        return b"# prelude\n\n" + b"\n".join(base) + b"\n# trailer\n"
    if mode == 2:
        return b'permit alice as root cmd "/usr/bin/id\n'
    if mode == 3:
        return b"permit alice as root cmd /usr/bin/id \\\n"
    if mode == 4:
        return b"permit alice as root \\\n cmd /usr/bin/id\n"
    if mode == 5:
        return b"permit setenv { PATH=/usr/bin PATH=/bin } alice as root cmd /usr/bin/env\n"
    if mode == 6:
        line = b"permit alice as root cmd /usr/bin/" + (b"x" * 9000)
        return line + b"\n"
    if mode == 7:
        return b"# ignored\0nul\npermit nopass alice as root cmd /usr/bin/id\n"
    if mode == 8:
        return b'permit "permit" as root cmd "/usr/bin/id"\n'
    return b"permit alice as root cmd /usr/bin/\xffid\n"


def write_case(case_dir: Path, conf_bytes: bytes) -> None:
    case_dir.mkdir(parents=True, exist_ok=True)
    (case_dir / "case.toml").write_text(
        '[case]\n'
        'oracle_variant = "shadow-off"\n'
        'subject_variant = "plain-off"\n'
        'run_as = "alice"\n'
    )
    (case_dir / "invoke.sh").write_text("#!/bin/sh\nexec doas -C /etc/doas.conf\n")
    (case_dir / "invoke.sh").chmod(0o755)
    (case_dir / "doas.conf").write_bytes(conf_bytes)


def run_stress(count: int, rebuild: bool, seed: int) -> int:
    runner = load_runner()
    rng = random.Random(seed)
    GENERATED_ROOT.mkdir(parents=True, exist_ok=True)
    failures: list[str] = []
    try:
        runner.build_image("opendoas", "shadow-off", rebuild)
        runner.build_image("opendoas-rs", "plain-off", rebuild)
        for index in range(count):
            case_id = f".generated/parser-stress/{index:03d}"
            case_dir = GENERATED_ROOT / f"{index:03d}"
            if case_dir.exists():
                shutil.rmtree(case_dir)
            write_case(case_dir, case_bytes(index, rng))
            case = runner.load_case(case_dir)
            oracle = runner.execute_case("opendoas", case_dir, case, rebuild=False)
            runner.write_baseline(case_id, oracle)
            subject = runner.execute_case("opendoas-rs", case_dir, case, rebuild=False)
            failures.extend(runner.compare_results(case_id, oracle, subject))
            if not failures:
                print(f"PASS {case_id}", flush=True)
        if failures:
            print("\n\n".join(failures), file=sys.stderr)
            return 1
        print(f"PASS parser-stress {count} cases", flush=True)
        return 0
    finally:
        shutil.rmtree(GENERATED_ROOT, ignore_errors=True)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--count", type=int, default=30)
    parser.add_argument("--seed", type=int, default=20260314)
    parser.add_argument("--rebuild", action="store_true")
    args = parser.parse_args()
    return run_stress(args.count, args.rebuild, args.seed)


if __name__ == "__main__":
    raise SystemExit(main())
