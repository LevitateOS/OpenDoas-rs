# Security Review Checklist

This checklist is for manual review of the privilege boundary.

Current status:

- no formal sign-off recorded yet

## Command Execution

Review:

- [ ] target uid/gid switching in `src/exec/privilege.rs`
- [ ] process spawn and exit propagation in `src/exec/spawn.rs`
- [ ] shell fallback and command path handling in `src/exec/run.rs` and
      `src/exec/path.rs`
- [ ] file descriptor closing behavior before exec

## Config Parsing And Policy

Review:

- [ ] tokenization and byte handling in `src/config/lexer.rs`
- [ ] parser error paths in `src/config/parser.rs`
- [ ] runtime config safety checks in `src/config/validate.rs`
- [ ] ordered rule evaluation and last-match-wins behavior in `src/policy`

## Authentication

Review:

- [ ] plain/shadow backend flow in `src/auth/plain.rs`
- [ ] PAM conversation, session, and error handling in `src/auth/pam.rs`
- [ ] no-auth backend isolation in `src/auth/none.rs`
- [ ] TTY assumptions in `src/platform/tty.rs`

## Environment And Logging

Review:

- [ ] environment inheritance and override logic in `src/exec/env.rs`
- [ ] unsafe path inheritance and `PATH` rewriting
- [ ] audit logging behavior in `src/logging/audit.rs`
- [ ] failure logging for deny and auth-failed paths

## Timestamp And Persist

Review:

- [ ] timestamp directory validation in `src/persist/timestamp.rs`
- [ ] symlink rejection and ownership/mode checks
- [ ] deauth behavior in `src/persist/deauth.rs`

## Review Outcome

For each completed review pass, record:

- reviewer
- date
- scope
- findings
- unresolved risks
