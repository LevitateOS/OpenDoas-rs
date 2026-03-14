# Gap Register

This document is the full register of the gaps between:

- `OpenDoas-rs` being conformance-green against upstream `OpenDoas`
- `OpenDoas-rs` being honestly claimable as production-ready and
  security-comparable in the real world

The purpose of this document is to prevent one specific mistake:

- passing the current oracle-driven behavior suite
  does **not** automatically mean
- the implementation is security-equivalent to `OpenDoas`

This document is the flat inventory.

The ordered execution plan lives in
[INVESTIGATION-PHASES.md](/home/vince/Projects/rsudoas/docs/INVESTIGATION-PHASES.md).

## Current Proven State

What is currently proven:

- the oracle-driven conformance suite is green against upstream `OpenDoas`
- `OpenDoas-rs` matches that current suite
- the Python runner currently executes `113` TOML-backed end-to-end cases across:
  - `cli`
  - `check`
  - `match`
  - `config`
  - `runtime`
  - `env`
  - `shell`
  - `auth`
  - `persist`
  - `logging`
- deterministic parser stress testing is wired into CI
- GitHub Actions CI is green for the current workflow

What is now known to be **not** proven cleanly:

- the full checked-in conformance corpus is not fully exercised by the current
  Python runner
- the current green suite does not rule out shared regressions in fixture-based
  negative cases
- several high-severity product defects still exist outside what the previous
  status language implied

What is **not** proven:

- security equivalence to `OpenDoas`
- implementation equivalence to `OpenDoas`
- absence of privilege-boundary bugs outside the currently modeled behavior
- real-world safety across multiple distributions and package/install paths
- operator safety under upgrade, rollback, and incident conditions

## Non-Claims

The project must **not** currently claim any of the following:

- “security-equivalent to `OpenDoas`”
- “production-ready”
- “fully audited”
- “proven safe on Linux in general”
- “all edge cases are exhausted”

The strongest honest claim right now is:

- `OpenDoas-rs` has meaningful behavioral parity evidence against the current
  TOML-backed upstream-`OpenDoas` oracle surface, but the first-pass
  investigation has already found serious product and harness gaps

## First-Pass Investigation Findings

The first-pass investigation phases recorded in
[docs/investigations](/home/vince/Projects/rsudoas/docs/investigations/README.md)
found concrete blockers that materially change the risk picture:

- High: the parser currently accepts non-oracle rule forms that can widen
  authorization, including `args` without `cmd`.
  Evidence:
  [02-phase-2-config-policy.md](/home/vince/Projects/rsudoas/docs/investigations/02-phase-2-config-policy.md)
- High: the plain auth backend contains unsafe behavior and PAM sessions are
  skipped on `nopass` or persisted execution paths.
  Evidence:
  [03-phase-3-auth-tty-session.md](/home/vince/Projects/rsudoas/docs/investigations/03-phase-3-auth-tty-session.md)
- High: audit logging currently passes attacker-controlled text into C
  `syslog(3)` as a raw format string.
  Evidence:
  [04-phase-4-persist-logging.md](/home/vince/Projects/rsudoas/docs/investigations/04-phase-4-persist-logging.md)
- High: the executable harness does not currently cover the full checked-in
  case corpus and can miss shared regressions.
  Evidence:
  [05-phase-5-harness-negative-testing.md](/home/vince/Projects/rsudoas/docs/investigations/05-phase-5-harness-negative-testing.md)
- High: environment validation is still container-heavy and does not prove
  real-host or multi-distro safety.
  Evidence:
  [06-phase-6-environment-validation.md](/home/vince/Projects/rsudoas/docs/investigations/06-phase-6-environment-validation.md)

These findings mean the project is not merely “not production-ready yet”; it
still has open correctness and security blockers that must be fixed before a
release-candidate claim is credible.

## Gap Classes

The remaining gaps fall into six classes:

1. security review gaps
2. negative-testing gaps
3. environment/deployment gaps
4. release/process gaps
5. harness-evidence gaps
6. oracle-scope gaps

## Priority Levels

- `P0`
  Must be closed before any production-ready claim.
- `P1`
  Should be closed before broad real-world rollout.
- `P2`
  Valuable hardening or scope expansion after the above.

## Execution Model

The gaps in this document should be worked through in the phase order defined
in [INVESTIGATION-PHASES.md](/home/vince/Projects/rsudoas/docs/INVESTIGATION-PHASES.md).

In practice that means:

1. lock the claim boundary
2. audit privilege boundary and execution
3. audit config and policy
4. audit auth, TTY, and session behavior
5. audit persist and logging
6. harden the harness and negative testing
7. validate on real environments
8. exercise release and soak gates

## P0 Gaps

### 1. No Formal Privilege-Boundary Review

Status:

- open

Why it matters:

- behavior parity does not prove the implementation is safe at the privilege
  boundary

Required review targets:

- [src/exec/privilege.rs](/home/vince/Projects/rsudoas/src/exec/privilege.rs)
- [src/exec/spawn.rs](/home/vince/Projects/rsudoas/src/exec/spawn.rs)
- [src/exec/run.rs](/home/vince/Projects/rsudoas/src/exec/run.rs)
- [src/exec/path.rs](/home/vince/Projects/rsudoas/src/exec/path.rs)
- [src/auth/plain.rs](/home/vince/Projects/rsudoas/src/auth/plain.rs)
- [src/auth/pam.rs](/home/vince/Projects/rsudoas/src/auth/pam.rs)
- [src/persist/timestamp.rs](/home/vince/Projects/rsudoas/src/persist/timestamp.rs)
- [src/config/parser.rs](/home/vince/Projects/rsudoas/src/config/parser.rs)
- [src/config/validate.rs](/home/vince/Projects/rsudoas/src/config/validate.rs)
- [src/policy/matcher.rs](/home/vince/Projects/rsudoas/src/policy/matcher.rs)

Specific questions that remain open:

- Are all uid/gid transitions ordered correctly?
- Are all supplementary groups handled safely?
- Is every exec path free of descriptor leakage?
- Are error paths and child-signal paths safe and complete?
- Are timestamp validation and reuse semantics safe under hostile filesystem
  conditions?
- Are auth failures and session teardown paths correct under partial failure?

### 2. No Formal Security Findings Register

Status:

- open

Why it matters:

- a security review without recorded findings and sign-off does not create
  release evidence

Missing artifact:

- completed review entries in
  [SECURITY-REVIEW.md](/home/vince/Projects/rsudoas/docs/SECURITY-REVIEW.md)

### 3. No Multi-Distro Runtime Validation

Status:

- open

Why it matters:

- current evidence is strongly Alpine/container oriented
- behavior and safety can change with PAM stacks, packaging, libc, and runtime
  environment

Minimum missing matrix:

- Alpine
- Debian or Ubuntu
- Arch
- `auth-plain`
- `auth-pam`
- timestamp `off`
- timestamp `on`

### 4. No Real Install-Path Validation Outside Conformance

Status:

- open

Why it matters:

- the conformance harness builds and installs inside controlled containers
- that does not prove a real package or manual install is safe on a host

Missing validations:

- setuid install on a real host
- PAM setup on a real host
- upgrade and rollback on a real host
- logging visibility on a real host

### 5. No Additional Fault-Injection Around Auth / Persist / Runtime

Status:

- open

Why it matters:

- parser stress now exists, but the highest-value failure paths are elsewhere

Still missing:

- auth fault injection
- persist fault injection
- runtime fault injection
- failure-path mutation checks around logging and audit behavior

### 6. No Release Soak Period

Status:

- open

Why it matters:

- root tools often look correct in tests and then fail under real operator use

Missing evidence:

- one release-candidate build used long enough to flush out operational defects

## P1 Gaps

### 7. CI Is Green But Not Yet a Proven Release Gate

Status:

- partially closed

What is true:

- CI exists
- the workflow is green

What is missing:

- branch protection or equivalent enforcement
- explicit rule that a failed CI blocks release

### 8. Release Process Exists But Has Not Been Exercised End-To-End

Status:

- partially closed

Artifacts now present:

- [release-preflight.sh](/home/vince/Projects/rsudoas/scripts/release-preflight.sh)
- [RELEASE-CHECKLIST.md](/home/vince/Projects/rsudoas/docs/RELEASE-CHECKLIST.md)
- [VERSIONING.md](/home/vince/Projects/rsudoas/docs/VERSIONING.md)
- [CHANGELOG.md](/home/vince/Projects/rsudoas/CHANGELOG.md)

What is still missing:

- a release performed from a clean checkout using this exact process
- a recorded artifact build from that process

### 9. Operator Docs Exist But Are Not Yet Operator-Validated

Status:

- partially closed

Artifacts now present:

- [INSTALL.md](/home/vince/Projects/rsudoas/docs/INSTALL.md)
- [OPERATIONS.md](/home/vince/Projects/rsudoas/docs/OPERATIONS.md)

What is still missing:

- proof that the documented steps work on at least two real target
  distributions
- operator review of rollback and incident response paths

### 10. No Bug-Backfill Policy Has Been Exercised In Practice

Status:

- open

Why it matters:

- production maturity depends on turning every real bug into a regression test

Current gap:

- the policy is described, but no real-world post-release bug loop exists yet

## P2 Gaps

### 11. Harness Blind-Spot Review Is Not Formally Closed

Status:

- open

What improved:

- TTY separation exists
- parser stress exists

What still needs explicit review:

- remaining assertion permissiveness
- mutation testing for false positives
- proof that the harness fails on deliberately wrong channel/output behavior

### 12. Oracle Scope Is Still Upstream `OpenDoas`, Not Alpine-Packaged `doas`

Status:

- open only if Alpine package parity matters

Current oracle:

- upstream `OpenDoas`

Potential future oracle expansion:

- [`.reference/Alpine-doas-3.23-stable`](/home/vince/Projects/rsudoas/.reference/Alpine-doas-3.23-stable)

Tracked missing Alpine-specific cases:

- `config/confdir-ignores-legacy-doas-conf`
- `config/confdir-alphasort-order`
- `config/confdir-no-matching-files`
- `config/check-mode-confdir-path`
- `env/alpine-safe-path-order`

### 13. No Long-Run Operational Metrics Yet

Status:

- open

Missing:

- evidence from repeated real usage
- evidence of log behavior under routine operations
- evidence of supportability during upgrades and failures

## Module-by-Module Risk Map

These are not claims of existing bugs. They are the parts of the product that
still need explicit trust-building work.

### Highest-Risk Product Areas

- [src/exec](/home/vince/Projects/rsudoas/src/exec)
  privilege transitions, spawn semantics, fd handling, shell fallback
- [src/auth](/home/vince/Projects/rsudoas/src/auth)
  PAM/session flow, shadow/plain auth behavior, prompt and TTY behavior
- [src/persist](/home/vince/Projects/rsudoas/src/persist)
  timestamp safety, symlink and ownership handling, deauth behavior

### Medium-Risk Product Areas

- [src/config](/home/vince/Projects/rsudoas/src/config)
  parser correctness, malformed-input handling, validation separation
- [src/policy](/home/vince/Projects/rsudoas/src/policy)
  last-match-wins, identity resolution, command/argv matching
- [src/logging](/home/vince/Projects/rsudoas/src/logging)
  security-relevant audit completeness and wording

### Lower-Risk But Still Relevant Areas

- [src/platform](/home/vince/Projects/rsudoas/src/platform)
  platform assumptions around passwd/groups/tty
- [src/app](/home/vince/Projects/rsudoas/src/app)
  orchestration correctness and error propagation

## Immediate Next Actions

The fastest responsible path forward is:

1. perform and record the manual security review in
   [SECURITY-REVIEW.md](/home/vince/Projects/rsudoas/docs/SECURITY-REVIEW.md)
2. add fault-injection coverage for auth, persist, and runtime
3. validate one real install path on Alpine and one on Debian/Ubuntu or Arch
4. exercise the documented release process from a clean checkout
5. run a release-candidate soak period

## Exit Condition

This gap register is not closed until:

- all `P0` gaps are closed
- all release-blocking `P1` gaps are closed
- the remaining `P2` gaps are either closed or explicitly accepted as
  out-of-scope with rationale

Until then, the correct position is:

- `OpenDoas-rs` has strong behavioral parity evidence
- `OpenDoas-rs` does **not yet** have complete security/process parity evidence
