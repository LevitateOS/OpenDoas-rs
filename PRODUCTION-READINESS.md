# Production Readiness

This document defines what `OpenDoas-rs` must satisfy before it should be
described as production-ready.

Current status:

- `OpenDoas-rs` is green against the current Python-runner conformance sweep.
- That sweep currently executes `113` TOML-backed cases against upstream
  `OpenDoas`.
- First-pass investigation phases have since found real high-severity product
  and harness issues that keep the project well short of production readiness.
- The full gap register is tracked in
  [GAP-REGISTER.md](/home/vince/Projects/rsudoas/docs/GAP-REGISTER.md).
- The investigation order for closing those gaps is tracked in
  [INVESTIGATION-PHASES.md](/home/vince/Projects/rsudoas/docs/INVESTIGATION-PHASES.md).

## Production-Ready Means

For this project, production-ready means all of the following are true:

- core behavior matches the chosen oracle closely enough that no known parity
  gaps remain in the supported surface
- the release process can prevent regressions, not just detect them manually
- the privilege boundary has been reviewed with security in mind, not only with
  parity in mind
- real deployment environments are covered, not just the current containerized
  oracle setup
- operators have enough installation, configuration, and recovery guidance to
  use the tool safely

## Current Assessment

Current assessment: `not production-ready`

Reasons:

- the first-pass investigations found open high-severity issues in parser,
  auth/session, logging, and harness coverage
- CI is present, but the current green sweep does not cover the full checked-in
  case corpus
- the current evidence is still narrow across real deployment environments
- no investigation phase that matters for a production claim has signed off yet

## Required Gates

These are the minimum gates before a production-ready claim should be made.

### 1. Conformance Gate

- [x] Shared oracle-driven conformance harness exists in
  [conformance](/home/vince/Projects/rsudoas/conformance)
- [x] Current suite is green end to end
- [x] Current upstream `OpenDoas` backlog is modeled
- [x] Full suite runs automatically in CI on every change
- [ ] CI failure blocks release
- [x] Release builds are tested from a clean environment, not only a warm local
  machine

### 2. Environment Matrix Gate

- [ ] Validate on at least two real Linux distributions, not only the current
  Alpine-based oracle path
- [ ] Validate both supported auth backends in those environments where
  relevant
- [ ] Validate setuid install and runtime behavior outside the conformance
  runner
- [ ] Validate a real package/install path, not only container-local builds

Suggested minimum matrix:

- Alpine
- Arch or Debian/Ubuntu
- `auth-plain`
- `auth-pam`
- timestamp `on`
- timestamp `off`

### 3. Security Review Gate

- [ ] Manual review of command execution and privilege transitions in
  [src/exec](/home/vince/Projects/rsudoas/src/exec)
- [ ] Manual review of config parsing and rule matching in
  [src/config](/home/vince/Projects/rsudoas/src/config) and
  [src/policy](/home/vince/Projects/rsudoas/src/policy)
- [ ] Manual review of auth flows in [src/auth](/home/vince/Projects/rsudoas/src/auth)
- [ ] Manual review of timestamp handling in
  [src/persist](/home/vince/Projects/rsudoas/src/persist)
- [ ] Review for unsafe environment inheritance and path handling
- [ ] Review for file descriptor leakage and child-process handling
- [ ] Review of logging and failure behavior for security-relevant audit trails

### 4. Negative Testing Gate

- [x] Parser fuzzing or equivalent malformed-input stress testing
- [ ] Additional fault-injection around auth, persist, and runtime paths
- [ ] Regression tests for any bugs found during real-world trial use
- [ ] At least one review pass focused specifically on harness blind spots

### 5. Operational Gate

- [x] Installation documentation is complete for each supported auth mode
- [x] PAM setup guidance is correct for supported distributions
- [x] Upgrade and rollback guidance exists
- [x] Failure modes are documented clearly enough for operators
- [x] Logging and audit expectations are documented
- [x] Safe default examples are provided for configuration

### 6. Release Gate

- [x] A reproducible release process exists
- [ ] Release artifacts are built from a clean tree
- [x] Versioning and changelog policy are defined
- [ ] A release checklist exists and is followed
- [ ] At least one release-candidate soak period completes without critical
  regressions

## Recommended Path To Production

The next work should happen in this order:

1. Add CI for the full conformance suite.
2. Run the suite across a small real distro/auth matrix.
3. Perform a focused manual security review of the privilege boundary.
4. Add parser and runtime stress testing.
5. Write installation and operational documentation.
6. Run at least one release-candidate soak period.

## Release Candidate Criteria

`OpenDoas-rs` can reasonably be called a release candidate when:

- the full conformance suite is green in CI
- there are no known parity gaps in the supported feature set
- at least one additional distro/backend matrix has passed
- the security review has not found unresolved critical issues
- installation and rollback documentation is present

## Production Claim Criteria

`OpenDoas-rs` should only be called production-ready when:

- all required gates in this document are complete
- no unresolved critical or high-severity security issues are open
- the current supported feature set is stated clearly in the project docs
- releases are reproducible and regression-gated

## Current Verdict

Verdict today: `not production-ready`, and not yet ready for a release-candidate
claim.

This is still a privilege-escalation tool. The current investigations have
already shown that “the current tests pass” was not a strong enough standard.
