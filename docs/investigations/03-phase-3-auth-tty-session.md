# Phase 3: Auth, TTY, And Session Audit

## Objective

- audit the authentication backends, TTY handling, and session/audit flow for
  the current implementation
- determine whether the Phase 3 surface is ready to sign off

## Scope

- [src/auth/plain.rs](/home/vince/Projects/rsudoas/src/auth/plain.rs)
- [src/auth/pam.rs](/home/vince/Projects/rsudoas/src/auth/pam.rs)
- [src/auth/none.rs](/home/vince/Projects/rsudoas/src/auth/none.rs)
- [src/platform/tty.rs](/home/vince/Projects/rsudoas/src/platform/tty.rs)
- [src/main.rs](/home/vince/Projects/rsudoas/src/main.rs)
- Supporting flow reads:
  [src/logging/audit.rs](/home/vince/Projects/rsudoas/src/logging/audit.rs),
  [src/persist/timestamp.rs](/home/vince/Projects/rsudoas/src/persist/timestamp.rs),
  [src/exec/run.rs](/home/vince/Projects/rsudoas/src/exec/run.rs),
  [src/cli/args.rs](/home/vince/Projects/rsudoas/src/cli/args.rs)

## Commands Run

```sh
rg --files src docs/investigations | rg '^(src/auth/(plain|pam|none)\.rs|src/platform/tty\.rs|src/main\.rs|docs/investigations/03-phase-3-auth-tty-session\.md)$'
nl -ba src/auth/plain.rs
nl -ba src/auth/pam.rs
nl -ba src/auth/none.rs
nl -ba src/platform/tty.rs
nl -ba src/main.rs
rg -n "challenge_user|authenticate\(|ensure_nopass|open_session|close\(|refresh\(|log_failed_auth|log_permitted_command|open_timestamp|is_valid\(" src/main.rs src/auth src/platform
rg -n "pub fn (open_timestamp|deauth)|struct .*Timestamp|fn is_valid|fn refresh" src
rg -n "pub fn (log_failed_auth|log_permitted_command|log_denied_command)" src
nl -ba src/logging/audit.rs
nl -ba src/persist/timestamp.rs
rg -n "interactive" src -g '*.rs'
nl -ba src/cli/args.rs | sed -n '1,90p'
rg -n "pub fn read_password|pub fn prompt_password" ~/.cargo/registry/src ~/.cargo/git/checkouts -g '*.rs'
sed -n '1,220p' /home/vince/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rpassword-7.3.1/src/lib.rs
rg -n "trait ConversationHandler|fn open_session|fn close\(|fn set_tty|fn set_user|fn reinitialize_credentials" ~/.cargo/git/checkouts ~/.cargo/registry/src -g '*.rs'
sed -n '1,240p' /home/vince/.cargo/git/checkouts/rust-pam-client-2d4a29d5c3105916/e8a406c/src/conversation.rs
sed -n '320,420p' /home/vince/.cargo/git/checkouts/rust-pam-client-2d4a29d5c3105916/e8a406c/src/context.rs
sed -n '140,220p' /home/vince/.cargo/git/checkouts/rust-pam-client-2d4a29d5c3105916/e8a406c/src/session.rs
nl -ba src/exec/run.rs | sed -n '1,160p'
```

## Findings

- High: the plain backend turns ordinary error handling into undefined behavior
  inside the setuid authentication path.
  `verify_hash()` uses `CString::new(...).unwrap_unchecked()` for both the
  stored hash and the user response, then dereferences the raw `crypt()`
  pointer without checking for null. That means malformed input or a `crypt()`
  failure does not degrade into a clean auth failure; it enters UB instead.
  Evidence:
  [src/auth/plain.rs:37](/home/vince/Projects/rsudoas/src/auth/plain.rs#L37),
  [src/auth/plain.rs:44](/home/vince/Projects/rsudoas/src/auth/plain.rs#L44),
  [src/auth/plain.rs:46](/home/vince/Projects/rsudoas/src/auth/plain.rs#L46),
  [src/auth/plain.rs:48](/home/vince/Projects/rsudoas/src/auth/plain.rs#L48)

- High: PAM session hooks are skipped entirely whenever authorization is
  satisfied by `nopass` or by a valid persistence ticket.
  The only branch that authenticates also opens the PAM session, and that
  branch is gated by `!rule_opts.nopass && !reuse_persist`. Commands still run
  afterwards for both cached and `nopass` executions, so those executions never
  call `pam_open_session()` or `pam_close_session()`. This leaves PAM-backed
  session setup, accounting, and teardown absent on common real-world paths.
  Evidence:
  [src/main.rs:143](/home/vince/Projects/rsudoas/src/main.rs#L143),
  [src/main.rs:147](/home/vince/Projects/rsudoas/src/main.rs#L147),
  [src/main.rs:150](/home/vince/Projects/rsudoas/src/main.rs#L150),
  [src/main.rs:165](/home/vince/Projects/rsudoas/src/main.rs#L165),
  [src/main.rs:170](/home/vince/Projects/rsudoas/src/main.rs#L170),
  [src/main.rs:172](/home/vince/Projects/rsudoas/src/main.rs#L172)

- Medium: PAM TTY attribution is bound to `stdin`, so redirected stdin drops
  `PAM_TTY` even when the user still has a controlling terminal.
  `current_tty_name()` resolves the tty from `std::io::stdin()`, not from the
  controlling terminal. In the PAM path, failure is silently ignored and
  `set_tty()` is skipped. At the same time, interactive execution remains the
  default and the password prompt comes from `/dev/tty`, so pipeline-style
  invocations can still authenticate while PAM loses terminal context.
  Evidence:
  [src/platform/tty.rs:17](/home/vince/Projects/rsudoas/src/platform/tty.rs#L17),
  [src/platform/tty.rs:18](/home/vince/Projects/rsudoas/src/platform/tty.rs#L18),
  [src/auth/pam.rs:67](/home/vince/Projects/rsudoas/src/auth/pam.rs#L67),
  [src/auth/pam.rs:68](/home/vince/Projects/rsudoas/src/auth/pam.rs#L68),
  [src/main.rs:143](/home/vince/Projects/rsudoas/src/main.rs#L143),
  [src/cli/args.rs:32](/home/vince/Projects/rsudoas/src/cli/args.rs#L32)

- Medium: PAM session-close and credential-delete failures are dropped on the
  floor.
  The code explicitly discards the result of `session.close(...)`. If
  `pam_close_session()` or the subsequent credential deletion fails, the caller
  gets no error, no warning, and no audit signal that teardown was incomplete.
  Evidence:
  [src/main.rs:172](/home/vince/Projects/rsudoas/src/main.rs#L172),
  [src/main.rs:173](/home/vince/Projects/rsudoas/src/main.rs#L173)

- Medium: the permitted-command audit entry is written before the privileged
  execution path can still fail.
  `run_permitted_command()` logs "`ran command`" before calling
  `execute_plan()`. `execute_plan()` can still reject invalid command data,
  environment setup, target switching, or spawn/wait failures. The current log
  message therefore overstates execution and weakens audit accuracy.
  Evidence:
  [src/main.rs:209](/home/vince/Projects/rsudoas/src/main.rs#L209),
  [src/main.rs:212](/home/vince/Projects/rsudoas/src/main.rs#L212),
  [src/logging/audit.rs:14](/home/vince/Projects/rsudoas/src/logging/audit.rs#L14),
  [src/exec/run.rs:40](/home/vince/Projects/rsudoas/src/exec/run.rs#L40),
  [src/exec/run.rs:58](/home/vince/Projects/rsudoas/src/exec/run.rs#L58),
  [src/exec/run.rs:61](/home/vince/Projects/rsudoas/src/exec/run.rs#L61)

- Medium: the `auth-none` backend reports success to the caller when the rule
  requires authentication and no auth backend is available.
  `ensure_nopass()` correctly returns an error for authenticated rules, but the
  caller only prints the message and returns from `execute()` instead of
  exiting non-zero. The privileged command does not run, yet the process status
  still indicates success.
  Evidence:
  [src/auth/none.rs:8](/home/vince/Projects/rsudoas/src/auth/none.rs#L8),
  [src/auth/none.rs:12](/home/vince/Projects/rsudoas/src/auth/none.rs#L12),
  [src/main.rs:126](/home/vince/Projects/rsudoas/src/main.rs#L126),
  [src/main.rs:128](/home/vince/Projects/rsudoas/src/main.rs#L128),
  [src/main.rs:130](/home/vince/Projects/rsudoas/src/main.rs#L130)

## Remaining Risks

- This was a static audit only. I did not execute the PAM, plain, or `auth-none`
  paths against a live PAM stack, syslog, or TTY matrix.
- The redirected-stdin case needs runtime verification against at least one PAM
  stack that consumes `PAM_TTY` for audit or policy.
- Persistence reuse, `nopass`, and PAM teardown need regression tests so the
  session and audit behavior is fixed by code rather than by assumption.
- The plain backend still needs negative tests around malformed hashes,
  unsupported hash formats, and non-standard terminal input.

## Exit Decision

Phase 3 is not ready to sign off.

The audit found multiple correctness and audit-integrity gaps, including a
high-severity unsafe path in plain auth and a high-severity PAM-session gap on
cached or `nopass` execution. The Phase 3 surface should stay open until those
issues are fixed and covered by regression tests.
