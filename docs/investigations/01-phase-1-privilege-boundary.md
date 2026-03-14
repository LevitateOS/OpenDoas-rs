# Phase 1: Privilege Boundary And Execution Audit

## Objective

- review the highest-risk execution paths where `OpenDoas-rs` changes process
  identity, prepares the child environment, and executes commands

## Scope

- [src/exec/privilege.rs](/home/vince/Projects/rsudoas/src/exec/privilege.rs)
- [src/exec/spawn.rs](/home/vince/Projects/rsudoas/src/exec/spawn.rs)
- [src/exec/run.rs](/home/vince/Projects/rsudoas/src/exec/run.rs)
- [src/exec/path.rs](/home/vince/Projects/rsudoas/src/exec/path.rs)
- [src/platform/passwd.rs](/home/vince/Projects/rsudoas/src/platform/passwd.rs)
- [src/platform/groups.rs](/home/vince/Projects/rsudoas/src/platform/groups.rs)
- [src/platform/tty.rs](/home/vince/Projects/rsudoas/src/platform/tty.rs)
- [src/app/execute.rs](/home/vince/Projects/rsudoas/src/app/execute.rs)

## Commands Run

```sh
git status --short
nl -ba src/exec/privilege.rs
nl -ba src/exec/spawn.rs
nl -ba src/exec/run.rs
nl -ba src/exec/path.rs
nl -ba src/platform/passwd.rs
nl -ba src/platform/groups.rs
nl -ba src/platform/tty.rs
nl -ba src/app/execute.rs
nl -ba src/main.rs | sed -n '1,280p'
nl -ba src/exec/env.rs
nl -ba src/config/validate.rs
nl -ba src/policy/command.rs
nl -ba src/auth/pam.rs | sed -n '1,220p'
nl -ba src/auth/plain.rs | sed -n '1,220p'
nl -ba src/persist/timestamp.rs | sed -n '1,220p'
rg -n "ensure_setuid_root|drop_to_real_uid|switch_to_target|execute_plan|load_rules|ConfigRequest|current_group_info|current_tty_name|stdin_is_tty|write_prompt_to_tty|reset_process_path|safe_path|spawn_and_wait" src
rg -n "build_exec_env|get_cmdline|validate_runtime_config_metadata|config_file|setresuid|setresgid|initgroups|posix_spawnp|ENOEXEC" src
rg -n "signal|sig|waitpid|process group|setpgid|kill\\(|SIG" src/exec src/main.rs src/platform
rg -n "ttyname\\(|/dev/tty|is_terminal\\(|set_tty\\(" src
cargo test -- --nocapture
cargo check --no-default-features --features auth-pam
```

## Findings

- Medium: the privilege boundary is crossed in the supervising process before
  child creation, so spawn failures and the entire wait path run under the
  target identity rather than in a short-lived child only.
  Evidence:
  [src/exec/run.rs:58](/home/vince/Projects/rsudoas/src/exec/run.rs#L58)
  [src/exec/spawn.rs:27](/home/vince/Projects/rsudoas/src/exec/spawn.rs#L27)
  [src/exec/spawn.rs:43](/home/vince/Projects/rsudoas/src/exec/spawn.rs#L43)
  [src/exec/privilege.rs:30](/home/vince/Projects/rsudoas/src/exec/privilege.rs#L30)
  Why it matters:
  a failed `posix_spawnp` or shell fallback still happens after
  `switch_to_target`, so the parent has already become the target user and
  cannot restore the original privilege state. That widens the privileged code
  surface to include spawn error handling, waiting, and any later cleanup added
  after `execute_plan`.

- Medium: child supervision is limited to a blocking `waitpid` loop; there is
  no signal forwarding or explicit parent-death handling in the execution path.
  Evidence:
  [src/exec/spawn.rs:43](/home/vince/Projects/rsudoas/src/exec/spawn.rs#L43)
  [src/exec/spawn.rs:46](/home/vince/Projects/rsudoas/src/exec/spawn.rs#L46)
  Why it matters:
  signals delivered to the `doas` wrapper PID are not relayed to the spawned
  command. A permitted target process can therefore outlive the wrapper under
  external termination or orchestration failures, which leaves privileged
  execution less controlled than the CLI surface suggests.

- Low: tty discovery is tied to `stdin` instead of the controlling terminal.
  Evidence:
  [src/platform/tty.rs:18](/home/vince/Projects/rsudoas/src/platform/tty.rs#L18)
  [src/platform/tty.rs:11](/home/vince/Projects/rsudoas/src/platform/tty.rs#L11)
  Why it matters:
  callers using `current_tty_name()` lose tty attribution whenever `stdin` is
  redirected, even if `/dev/tty` still exists and is usable for prompting. That
  weakens tty-bound audit or policy context on PAM-backed builds.

- Informational: no direct defect was found in the uid/gid transition order,
  source identity lookup, or config metadata check within the reviewed files.
  Evidence:
  [src/exec/privilege.rs:20](/home/vince/Projects/rsudoas/src/exec/privilege.rs#L20)
  [src/exec/privilege.rs:28](/home/vince/Projects/rsudoas/src/exec/privilege.rs#L28)
  [src/platform/passwd.rs:4](/home/vince/Projects/rsudoas/src/platform/passwd.rs#L4)
  [src/platform/groups.rs:12](/home/vince/Projects/rsudoas/src/platform/groups.rs#L12)
  [src/exec/path.rs:9](/home/vince/Projects/rsudoas/src/exec/path.rs#L9)
  [src/app/execute.rs:42](/home/vince/Projects/rsudoas/src/app/execute.rs#L42)
  Notes:
  the reviewed code does switch `gid` before `initgroups` and `uid`, the source
  user lookup is based on the real uid, the primary gid is forced into the
  evaluated group set, `PATH` is reset before command lookup, and runtime config
  metadata validation is performed on the already opened file descriptor when
  permission checks are enabled.

## Remaining Risks

- No automated tests exercise the audited execution boundary yet. `cargo test
  -- --nocapture` completed successfully but discovered `0` unit tests and `0`
  doc tests for this path.
- The PAM-specific branch was not exercised dynamically in this session.
  `cargo check --no-default-features --features auth-pam` failed in the
  `pam-sys` build script with `SIGSEGV`, so the tty-related finding is based on
  static review rather than a compiled PAM build.
- File-descriptor closing still depends on
  [src/exec/spawn.rs:122](/home/vince/Projects/rsudoas/src/exec/spawn.rs#L122)
  reading `/proc/self/fd`, and this audit did not validate behavior on a
  proc-restricted or non-Linux runtime.

## Exit Decision

Open.

This pass did not find an immediately exploitable critical defect in the audited
files, but the phase should stay open. The parent-side privilege transition in
[src/exec/run.rs:58](/home/vince/Projects/rsudoas/src/exec/run.rs#L58), the
missing signal-supervision story in
[src/exec/spawn.rs:43](/home/vince/Projects/rsudoas/src/exec/spawn.rs#L43), and
the complete absence of targeted execution-path tests mean the privilege
boundary is reviewed but not yet evidenced strongly enough to close.
