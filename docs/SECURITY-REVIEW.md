# Security Review Checklist

This checklist is for manual review of the privilege boundary.

Current status:

- no formal sign-off recorded yet
- first-pass phase investigations now exist under
  [docs/investigations](/home/vince/Projects/rsudoas/docs/investigations/README.md)
- multiple open findings remain, including high-severity product issues in
  authentication, logging, and parser behavior

## Recorded Investigation Passes

- [Phase 0: Claim Boundary And Evidence Lock](/home/vince/Projects/rsudoas/docs/investigations/00-phase-0-claim-boundary.md)
- [Phase 1: Privilege Boundary And Execution Audit](/home/vince/Projects/rsudoas/docs/investigations/01-phase-1-privilege-boundary.md)
- [Phase 2: Config And Policy Audit](/home/vince/Projects/rsudoas/docs/investigations/02-phase-2-config-policy.md)
- [Phase 3: Auth, TTY, And Session Audit](/home/vince/Projects/rsudoas/docs/investigations/03-phase-3-auth-tty-session.md)
- [Phase 4: Persist And Logging Audit](/home/vince/Projects/rsudoas/docs/investigations/04-phase-4-persist-logging.md)
- [Phase 5: Harness And Negative-Testing Hardening](/home/vince/Projects/rsudoas/docs/investigations/05-phase-5-harness-negative-testing.md)
- [Phase 6: Environment Validation](/home/vince/Projects/rsudoas/docs/investigations/06-phase-6-environment-validation.md)
- [Phase 7: Release Gate, Soak, And Operational Validation](/home/vince/Projects/rsudoas/docs/investigations/07-phase-7-release-soak.md)

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

The phase reports above are the current record. This checklist should not be
considered signed off until the open findings in those reports are either fixed
or explicitly accepted with rationale.
