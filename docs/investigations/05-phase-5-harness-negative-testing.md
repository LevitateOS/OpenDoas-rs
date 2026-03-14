# Phase 5: Harness And Negative-Testing Hardening

## Objective

- verify that the conformance harness actually exercises the intended corpus
- verify that negative tests assert known-bad behavior rather than only subject-oracle parity
- determine whether the current parser-stress and backlog tracking are strong enough to close this phase

## Scope

- [conformance.py](/home/vince/Projects/rsudoas/conformance/runner/bin/conformance.py)
- [parser_stress.py](/home/vince/Projects/rsudoas/conformance/runner/bin/parser_stress.py)
- [run-all.sh](/home/vince/Projects/rsudoas/conformance/runner/bin/run-all.sh)
- [run-case.sh](/home/vince/Projects/rsudoas/conformance/runner/bin/run-case.sh)
- [lib.sh](/home/vince/Projects/rsudoas/conformance/runner/bin/lib.sh)
- [MISSING-CASES.md](/home/vince/Projects/rsudoas/conformance/MISSING-CASES.md)
- [EDGE-CASE-BACKLOG.md](/home/vince/Projects/rsudoas/conformance/EDGE-CASE-BACKLOG.md)
- representative case-corpus entries, including [default-target-env/case.env](/home/vince/Projects/rsudoas/conformance/cases/env/default-target-env/case.env#L1), [permit-output/case.env](/home/vince/Projects/rsudoas/conformance/cases/check/permit-output/case.env#L1), [shell-fallback-passwd-shell/case.env](/home/vince/Projects/rsudoas/conformance/cases/shell/shell-fallback-passwd-shell/case.env#L1), [valid-basic/case.toml](/home/vince/Projects/rsudoas/conformance/cases/check/valid-basic/case.toml#L1), and [nopass-id-root/case.toml](/home/vince/Projects/rsudoas/conformance/cases/runtime/nopass-id-root/case.toml#L1)

## Commands Run

```sh
git status --short
sed -n '1,260p' conformance/runner/bin/conformance.py
sed -n '261,520p' conformance/runner/bin/conformance.py
sed -n '1,220p' conformance/runner/bin/parser_stress.py
sed -n '1,240p' conformance/runner/bin/lib.sh
sed -n '1,220p' conformance/runner/bin/run-all.sh
sed -n '1,220p' conformance/runner/bin/run-case.sh
sed -n '1,220p' conformance/MISSING-CASES.md
sed -n '1,220p' conformance/EDGE-CASE-BACKLOG.md
find conformance/cases -mindepth 2 -maxdepth 2 -type d | wc -l
find conformance/cases -type f -name case.toml | wc -l
find conformance/cases -mindepth 2 -maxdepth 2 -type d | while read -r d; do [ -f "$d/case.toml" ] || printf '%s\n' "$d"; done | sort
rg -n "compare\\b|expect\\.(exit|stdout|stderr)|stdout\\b|stderr\\b" conformance/runner conformance/cases
python3 - <<'PY'
import importlib.util
from pathlib import Path
path = Path('conformance/runner/bin/parser_stress.py')
spec = importlib.util.spec_from_file_location('parser_stress', path)
mod = importlib.util.module_from_spec(spec)
spec.loader.exec_module(mod)
rng = mod.random.Random(20260314)
items = [mod.case_bytes(i, rng) for i in range(30)]
print('total', len(items))
print('unique', len(set(items)))
PY
./conformance/runner/bin/run-all.sh opendoas
```

## Findings

1. High: `run-suite` silently excludes 41 existing case directories, so the harness does not execute the full checked-in corpus.
   The Python runner hard-requires `case.toml` in [conformance.py#L108](/home/vince/Projects/rsudoas/conformance/runner/bin/conformance.py#L108) and only discovers cases by walking `case.toml` parents in [conformance.py#L438](/home/vince/Projects/rsudoas/conformance/runner/bin/conformance.py#L438). The current tree has 154 case directories but only 113 `case.toml` files, leaving 41 directories unreachable from `run-suite`. Every skipped directory still carries runnable legacy metadata via `case.env`, for example [default-target-env/case.env](/home/vince/Projects/rsudoas/conformance/cases/env/default-target-env/case.env#L1), [permit-output/case.env](/home/vince/Projects/rsudoas/conformance/cases/check/permit-output/case.env#L1), and [shell-fallback-passwd-shell/case.env](/home/vince/Projects/rsudoas/conformance/cases/shell/shell-fallback-passwd-shell/case.env#L1). A green suite therefore overstates real corpus coverage across `auth`, `check`, `config`, `env`, `logging`, `match`, `persist`, `runtime`, and `shell`.

2. High: the Python harness no longer consumes checked-in expected outputs, so negative tests are parity-only and shared regressions can pass cleanly.
   `load_case()` assigns a `compare` field in [conformance.py#L112](/home/vince/Projects/rsudoas/conformance/runner/bin/conformance.py#L112), but the runner never reads it. Instead, [write_baseline() in conformance.py#L366](/home/vince/Projects/rsudoas/conformance/runner/bin/conformance.py#L366) only writes oracle artifacts, and [compare_results() in conformance.py#L372](/home/vince/Projects/rsudoas/conformance/runner/bin/conformance.py#L372) checks only oracle-versus-subject equality. That leaves the older fixture model in [lib.sh#L71](/home/vince/Projects/rsudoas/conformance/runner/bin/lib.sh#L71), [lib.sh#L88](/home/vince/Projects/rsudoas/conformance/runner/bin/lib.sh#L88), and [lib.sh#L154](/home/vince/Projects/rsudoas/conformance/runner/bin/lib.sh#L154) effectively dead. I counted 100 expectation-like fixture files under `conformance/cases/`, including active TOML-enabled cases such as [valid-basic/expect.exit](/home/vince/Projects/rsudoas/conformance/cases/check/valid-basic/expect.exit#L1) and [nopass-id-root/expect.stdout](/home/vince/Projects/rsudoas/conformance/cases/runtime/nopass-id-root/expect.stdout#L1). If both implementations drift in the same wrong direction, the current harness will still report success.

3. Medium: `parser_stress.py` is a thin smoke test, not hardening-grade negative coverage.
   The generator chooses cases by `index % 10` in [parser_stress.py#L47](/home/vince/Projects/rsudoas/conformance/runner/bin/parser_stress.py#L47), and eight of those ten modes are fixed byte strings in [parser_stress.py#L54](/home/vince/Projects/rsudoas/conformance/runner/bin/parser_stress.py#L54) through [parser_stress.py#L69](/home/vince/Projects/rsudoas/conformance/runner/bin/parser_stress.py#L69). With the default `--count 30` in [parser_stress.py#L117](/home/vince/Projects/rsudoas/conformance/runner/bin/parser_stress.py#L117), the local measurement above produced only 14 unique configs. The execution path in [parser_stress.py#L93](/home/vince/Projects/rsudoas/conformance/runner/bin/parser_stress.py#L93) through [parser_stress.py#L109](/home/vince/Projects/rsudoas/conformance/runner/bin/parser_stress.py#L109) reuses the same parity-only `compare_results()` flow from [conformance.py#L372](/home/vince/Projects/rsudoas/conformance/runner/bin/conformance.py#L372), so this stress runner cannot catch shared parser regressions either.

4. Medium: `run-all.sh` is stale and currently broken against the Python runner.
   The wrapper still advertises `run-all.sh [--rebuild-image] <opendoas|opendoas-rs>` in [run-all.sh#L7](/home/vince/Projects/rsudoas/conformance/runner/bin/run-all.sh#L7) through [run-all.sh#L13](/home/vince/Projects/rsudoas/conformance/runner/bin/run-all.sh#L13), then passes both an implementation selector and a case path into [run-case.sh#L1](/home/vince/Projects/rsudoas/conformance/runner/bin/run-case.sh#L1), even though [conformance.py#L502](/home/vince/Projects/rsudoas/conformance/runner/bin/conformance.py#L502) accepts only a single `case` positional and `--rebuild`. A direct invocation of `./conformance/runner/bin/run-all.sh opendoas` failed immediately with `conformance.py: error: unrecognized arguments`, so one of the harness entrypoints is already unusable.

5. Medium: the backlog trackers are ahead of executable evidence.
   [MISSING-CASES.md#L3](/home/vince/Projects/rsudoas/conformance/MISSING-CASES.md#L3) through [MISSING-CASES.md#L22](/home/vince/Projects/rsudoas/conformance/MISSING-CASES.md#L22) and [EDGE-CASE-BACKLOG.md#L3](/home/vince/Projects/rsudoas/conformance/EDGE-CASE-BACKLOG.md#L3) through [EDGE-CASE-BACKLOG.md#L10](/home/vince/Projects/rsudoas/conformance/EDGE-CASE-BACKLOG.md#L10) state that the tracked upstream backlog is implemented in `conformance/cases/`. That statement is not reliable while `run-suite` skips 41 checked-in case directories and the Python runner ignores the fixture-based assertions that still anchor many negative cases. The docs currently describe corpus presence, not exercised and asserted coverage.

## Remaining Risks

- A green `run-suite` currently proves only that `OpenDoas-rs` matches `OpenDoas` on the 113 TOML-enabled cases that the Python runner can discover.
- Negative cases backed by golden fixtures can regress silently if both implementations move together.
- Parser-stress currently adds only narrow, mostly repeated parser inputs and inherits the same shared-regression blind spot.
- Stale wrapper scripts and optimistic backlog docs can mislead maintainers about what the harness actually exercises.

## Exit Decision

Open.

Phase 5 should not be closed yet. The harness needs a single authoritative case format, the Python runner needs to either consume or deliberately retire the existing expectation fixtures, parser stress needs broader and less repetitive negative coverage, and the status docs need to be brought back in line with what the executable harness really proves.
