# Phase 4: Persist And Logging Audit

## Objective

- verify timestamp safety and security-relevant audit/logging behavior under
  hostile filesystem and failure conditions

## Scope

- [src/persist/timestamp.rs](/home/vince/Projects/rsudoas/src/persist/timestamp.rs)
- [src/persist/deauth.rs](/home/vince/Projects/rsudoas/src/persist/deauth.rs)
- [src/logging/audit.rs](/home/vince/Projects/rsudoas/src/logging/audit.rs)
- [src/main.rs](/home/vince/Projects/rsudoas/src/main.rs)
- supporting read-only context:
  [src/exec/run.rs](/home/vince/Projects/rsudoas/src/exec/run.rs),
  [src/policy/command.rs](/home/vince/Projects/rsudoas/src/policy/command.rs),
  [syslog-c-0.1.3/src/lib.rs](/home/vince/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/syslog-c-0.1.3/src/lib.rs)

## Commands Run

```sh
sed -n '1,260p' src/persist/timestamp.rs
sed -n '1,200p' src/persist/deauth.rs
sed -n '1,200p' src/logging/audit.rs
nl -ba src/main.rs | sed -n '1,280p'
nl -ba src/exec/run.rs | sed -n '1,220p'
nl -ba src/policy/command.rs | sed -n '1,220p'
nl -ba /home/vince/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/syslog-c-0.1.3/src/lib.rs | sed -n '1,220p'
cargo test -- --list
python3 conformance/runner/bin/conformance.py run-case conformance/cases/persist/persist-reuse
python3 conformance/runner/bin/conformance.py run-case conformance/cases/persist/timestamp-file-symlink
python3 conformance/runner/bin/conformance.py run-case conformance/cases/logging/permit-log-cwd-failed
python3 conformance/runner/bin/conformance.py run-case conformance/cases/logging/auth-failed-wrong-password
python3 -c 'import ctypes; libc=ctypes.CDLL(None); libc.syslog(5, b"%n")' ; echo $?
```

## Findings

- High: audit logging passes attacker-controlled text into C `syslog(3)` as a
  raw format string, creating a format-string vulnerability and a
  user-triggerable crash path.
  Evidence:
  [src/logging/audit.rs:4](/home/vince/Projects/rsudoas/src/logging/audit.rs#L4),
  [src/logging/audit.rs:14](/home/vince/Projects/rsudoas/src/logging/audit.rs#L14),
  [src/main.rs:98](/home/vince/Projects/rsudoas/src/main.rs#L98),
  [src/main.rs:212](/home/vince/Projects/rsudoas/src/main.rs#L212),
  [src/policy/command.rs:1](/home/vince/Projects/rsudoas/src/policy/command.rs#L1),
  [src/exec/run.rs:33](/home/vince/Projects/rsudoas/src/exec/run.rs#L33),
  [syslog-c-0.1.3/src/lib.rs:38](/home/vince/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/syslog-c-0.1.3/src/lib.rs#L38).
  `log_denied_command()` and `log_permitted_command()` format strings with raw
  command-line and cwd data, then `syslog-c` forwards that message as the C
  format string instead of `"%s"`. The direct repro command above returned
  `139` for a `"%n"` payload, which shows the call site can crash when the
  message contains a format directive that expects missing varargs. In the
  product path, an unprivileged caller controls the denied-command cmdline and
  can also control permit-path arguments and working-directory names.

- Medium: the permit audit record is emitted before privilege switching and
  spawn succeed, so the log can claim a command "ran" when execution actually
  failed.
  Evidence:
  [src/main.rs:209](/home/vince/Projects/rsudoas/src/main.rs#L209),
  [src/main.rs:212](/home/vince/Projects/rsudoas/src/main.rs#L212),
  [src/exec/run.rs:40](/home/vince/Projects/rsudoas/src/exec/run.rs#L40),
  [src/exec/run.rs:58](/home/vince/Projects/rsudoas/src/exec/run.rs#L58),
  [src/exec/spawn.rs:21](/home/vince/Projects/rsudoas/src/exec/spawn.rs#L21),
  [src/exec/privilege.rs:19](/home/vince/Projects/rsudoas/src/exec/privilege.rs#L19).
  `run_permitted_command()` logs first and only then calls `execute_plan()`.
  `execute_plan()` can still fail during `CString` conversion, target identity
  switching, or `spawn_and_wait()`. That means `command not found`,
  `setresuid`, `initgroups`, or similar runtime failures can still leave a
  success-sounding audit record behind.

- Low: unsafe or broken timestamp storage is handled fail-closed for privilege
  reuse, but largely invisible to operators because persist setup failures are
  silently downgraded to "no timestamp".
  Evidence:
  [src/persist/timestamp.rs:56](/home/vince/Projects/rsudoas/src/persist/timestamp.rs#L56),
  [src/persist/timestamp.rs:61](/home/vince/Projects/rsudoas/src/persist/timestamp.rs#L61),
  [src/persist/timestamp.rs:64](/home/vince/Projects/rsudoas/src/persist/timestamp.rs#L64),
  [src/persist/timestamp.rs:73](/home/vince/Projects/rsudoas/src/persist/timestamp.rs#L73),
  [src/persist/timestamp.rs:93](/home/vince/Projects/rsudoas/src/persist/timestamp.rs#L93),
  [src/main.rs:115](/home/vince/Projects/rsudoas/src/main.rs#L115),
  [src/main.rs:120](/home/vince/Projects/rsudoas/src/main.rs#L120).
  `open_timestamp()` converts several hostile-filesystem and setup failures into
  `Ok(None)` instead of surfacing an error or audit signal. The exercised cases
  `persist/persist-reuse` and `persist/timestamp-file-symlink` passed, which is
  good evidence that reuse stays closed on invalid state, but the operator gets
  no direct signal that persist has been suppressed by a bad `/run/doas`
  condition.

- Informational: the exercised coverage that already passes in this scope is
  useful but incomplete. `persist/persist-reuse`,
  `persist/timestamp-file-symlink`, `logging/permit-log-cwd-failed`, and
  `logging/auth-failed-wrong-password` all passed, which supports the existing
  timestamp validation path, cwd fallback path, and wrong-password logging path.
  None of the exercised cases covered `%`-bearing audit payloads or a
  permit-path execution failure after the audit log call.

## Remaining Risks

- No existing case in [conformance/cases/logging](/home/vince/Projects/rsudoas/conformance/cases/logging)
  exercises `%` or control-character payloads through deny or permit logging, so
  the current high-severity audit bug would not be caught by the present suite.
- No existing captured-syslog case proves that
  [src/main.rs:212](/home/vince/Projects/rsudoas/src/main.rs#L212) is only
  reached after successful execution. A targeted `command-not-found` or forced
  pre-exec failure case is still missing.
- Persist negative coverage exercises symlink and reuse handling, but there is
  still no explicit evidence that operators receive a useful warning when
  [src/persist/timestamp.rs:56](/home/vince/Projects/rsudoas/src/persist/timestamp.rs#L56)
  suppresses persist state due to hostile filesystem conditions.

## Exit Decision

Open.

Phase 4 should not exit yet. The audit path still contains a high-severity
format-string bug, and the permit log still overstates execution success on
pre-exec failures. The timestamp path looks fail-closed on the exercised safety
cases, but its hostile-filesystem failure mode is still too quiet for a clean
sign-off.
