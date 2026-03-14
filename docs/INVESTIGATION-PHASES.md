# Investigation Phases

This document turns the flat gap inventory in
[GAP-REGISTER.md](/home/vince/Projects/rsudoas/docs/GAP-REGISTER.md) into an
ordered investigation plan.

The intent is simple:

- keep the full gap register as the source of truth for open risk
- break the work into phases that can be executed, reviewed, and closed
- force each phase to produce evidence, not just opinions

## How To Use This Document

Each phase defines:

- the objective
- the primary code scope
- the questions that must be answered
- the evidence that must be produced
- the exit condition for closing the phase
- the gap-register items it closes or materially advances

The phases are ordered. Later phases can start early if useful, but no
production-ready claim should be made until all `P0`-relevant phases are closed.

## Phase 0: Claim Boundary And Evidence Lock

Objective:

- lock what the project can honestly claim today
- prevent “tests are green” from being misread as “security-equivalent”

Primary scope:

- [README.md](/home/vince/Projects/rsudoas/README.md)
- [PRODUCTION-READINESS.md](/home/vince/Projects/rsudoas/PRODUCTION-READINESS.md)
- [GAP-REGISTER.md](/home/vince/Projects/rsudoas/docs/GAP-REGISTER.md)
- [SECURITY-REVIEW.md](/home/vince/Projects/rsudoas/docs/SECURITY-REVIEW.md)

Questions:

- What is actually proven today?
- What must the project explicitly not claim yet?
- What evidence counts toward production readiness?

Required evidence:

- production-readiness language aligned across docs
- a flat gap register
- this phased investigation plan

Exit condition:

- the claim boundary is documented clearly enough that parity and security are
  not conflated in project docs

Advances gap items:

- foundation for all items in
  [GAP-REGISTER.md](/home/vince/Projects/rsudoas/docs/GAP-REGISTER.md)

## Phase 1: Privilege Boundary And Execution Audit

Objective:

- review the highest-risk execution paths where `OpenDoas-rs` changes process
  identity, prepares the child environment, and executes commands

Primary scope:

- [src/exec/privilege.rs](/home/vince/Projects/rsudoas/src/exec/privilege.rs)
- [src/exec/spawn.rs](/home/vince/Projects/rsudoas/src/exec/spawn.rs)
- [src/exec/run.rs](/home/vince/Projects/rsudoas/src/exec/run.rs)
- [src/exec/path.rs](/home/vince/Projects/rsudoas/src/exec/path.rs)
- [src/platform/passwd.rs](/home/vince/Projects/rsudoas/src/platform/passwd.rs)
- [src/platform/groups.rs](/home/vince/Projects/rsudoas/src/platform/groups.rs)
- [src/platform/tty.rs](/home/vince/Projects/rsudoas/src/platform/tty.rs)
- [src/app/execute.rs](/home/vince/Projects/rsudoas/src/app/execute.rs)

Questions:

- Are uid/gid transitions ordered correctly?
- Are supplementary groups handled safely and completely?
- Are all file descriptors above `stderr` closed or intentionally preserved?
- Are signal, `exec`, and child-exit paths safe and complete?
- Is path resolution correct under hostile or unusual environments?

Required evidence:

- written review findings recorded in
  [SECURITY-REVIEW.md](/home/vince/Projects/rsudoas/docs/SECURITY-REVIEW.md)
- concrete findings or “no issue found” notes per reviewed file
- additional targeted tests for any execution-path blind spots discovered

Exit condition:

- privilege-boundary and execution findings are recorded and no unresolved
  critical issues remain in this scope

Closes or advances:

- `P0.1` No Formal Privilege-Boundary Review
- `P0.2` No Formal Security Findings Register
- part of `P0.5` No Additional Fault-Injection Around Auth / Persist / Runtime

## Phase 2: Config And Policy Audit

Objective:

- verify that rule parsing, validation, and matching are safe and semantically
  aligned with the intended oracle behavior

Primary scope:

- [src/config/parser.rs](/home/vince/Projects/rsudoas/src/config/parser.rs)
- [src/config/validate.rs](/home/vince/Projects/rsudoas/src/config/validate.rs)
- [src/config/lexer.rs](/home/vince/Projects/rsudoas/src/config/lexer.rs)
- [src/policy/matcher.rs](/home/vince/Projects/rsudoas/src/policy/matcher.rs)
- [src/policy/identity.rs](/home/vince/Projects/rsudoas/src/policy/identity.rs)
- [src/policy/command.rs](/home/vince/Projects/rsudoas/src/policy/command.rs)
- [src/policy/rule.rs](/home/vince/Projects/rsudoas/src/policy/rule.rs)

Questions:

- Are parsing and validation failures safe and deterministic?
- Are rule-order and “last match wins” semantics implemented safely?
- Are user, group, target, and command identities resolved safely?
- Are hostile configs rejected in every relevant runtime path?

Required evidence:

- review notes in
  [SECURITY-REVIEW.md](/home/vince/Projects/rsudoas/docs/SECURITY-REVIEW.md)
- targeted tests for any parser, validation, or matcher gaps discovered
- explicit sign-off that config safety and policy semantics were reviewed

Exit condition:

- config and policy review is recorded with no unresolved critical issues in
  this scope

Closes or advances:

- `P0.1` No Formal Privilege-Boundary Review
- `P0.2` No Formal Security Findings Register
- part of `P0.5` No Additional Fault-Injection Around Auth / Persist / Runtime

## Phase 3: Auth, TTY, And Session Audit

Objective:

- review the highest-risk authentication and session-management behavior,
  especially where PAM, TTY handling, and failure modes can diverge subtly

Primary scope:

- [src/auth/plain.rs](/home/vince/Projects/rsudoas/src/auth/plain.rs)
- [src/auth/pam.rs](/home/vince/Projects/rsudoas/src/auth/pam.rs)
- [src/auth/none.rs](/home/vince/Projects/rsudoas/src/auth/none.rs)
- [src/platform/tty.rs](/home/vince/Projects/rsudoas/src/platform/tty.rs)

Questions:

- Are prompt, noninteractive, and no-TTY paths safe and correct?
- Are PAM session and teardown paths complete?
- Are auth failures logged and surfaced correctly?
- Are partial PAM or TTY failures handled safely?

Required evidence:

- review notes and findings in
  [SECURITY-REVIEW.md](/home/vince/Projects/rsudoas/docs/SECURITY-REVIEW.md)
- targeted auth fault-injection cases where useful
- explicit sign-off on PAM and plain auth behavior in supported modes

Exit condition:

- auth and session behavior has recorded review coverage and no unresolved
  critical issues remain in this scope

Closes or advances:

- `P0.1` No Formal Privilege-Boundary Review
- `P0.2` No Formal Security Findings Register
- part of `P0.5` No Additional Fault-Injection Around Auth / Persist / Runtime

## Phase 4: Persist And Logging Audit

Objective:

- verify timestamp safety and security-relevant audit/logging behavior under
  hostile filesystem and failure conditions

Primary scope:

- [src/persist/timestamp.rs](/home/vince/Projects/rsudoas/src/persist/timestamp.rs)
- [src/persist/deauth.rs](/home/vince/Projects/rsudoas/src/persist/deauth.rs)
- [src/logging/audit.rs](/home/vince/Projects/rsudoas/src/logging/audit.rs)
- [src/main.rs](/home/vince/Projects/rsudoas/src/main.rs)

Questions:

- Are timestamp directories and files validated safely?
- Are symlink, owner, mode, and time-based checks safe?
- Do logging and audit paths preserve the information operators need?
- Are deny and failed-auth cases visible under real failure conditions?

Required evidence:

- review notes in
  [SECURITY-REVIEW.md](/home/vince/Projects/rsudoas/docs/SECURITY-REVIEW.md)
- targeted persist/logging fault-injection tests
- explicit sign-off for timestamp and audit behavior

Exit condition:

- persist and logging review is recorded and no unresolved critical issues
  remain in this scope

Closes or advances:

- `P0.1` No Formal Privilege-Boundary Review
- `P0.2` No Formal Security Findings Register
- remaining part of `P0.5` No Additional Fault-Injection Around Auth / Persist / Runtime

## Phase 5: Harness And Negative-Testing Hardening

Objective:

- close the gap between “the suite is broad” and “the suite is not masking the
  product’s faults”

Primary scope:

- [conformance](/home/vince/Projects/rsudoas/conformance)
- [conformance/runner/bin/conformance.py](/home/vince/Projects/rsudoas/conformance/runner/bin/conformance.py)
- [conformance/runner/bin/parser_stress.py](/home/vince/Projects/rsudoas/conformance/runner/bin/parser_stress.py)
- [conformance/MISSING-CASES.md](/home/vince/Projects/rsudoas/conformance/MISSING-CASES.md)
- [conformance/EDGE-CASE-BACKLOG.md](/home/vince/Projects/rsudoas/conformance/EDGE-CASE-BACKLOG.md)

Questions:

- Can the harness still hide product faults in any important area?
- Do fault-injection and mutation-style checks fail when they should?
- Are current oracle assumptions narrow or misleading?

Required evidence:

- documented harness blind-spot review
- targeted negative tests for auth, persist, and runtime failures
- any newly found blind spots added back into the case corpus or backlog

Exit condition:

- harness blind-spot review is explicitly closed and no known high-risk masking
  issue remains open

Closes or advances:

- `P0.5` No Additional Fault-Injection Around Auth / Persist / Runtime
- `P2.11` Harness Blind-Spot Review Is Not Formally Closed

## Phase 6: Real Environment Validation

Objective:

- validate that the project behaves safely outside the controlled conformance
  containers

Primary scope:

- packaging/install paths
- real host setuid install paths
- real PAM configuration paths
- distro differences

Minimum matrix:

- Alpine
- Debian or Ubuntu
- Arch
- `auth-plain`
- `auth-pam`
- timestamp `off`
- timestamp `on`

Questions:

- Does a real install behave safely on supported environments?
- Are package, PAM, libc, and filesystem differences handled safely?
- Are install, upgrade, rollback, and logging paths acceptable on hosts?

Required evidence:

- matrix run records
- install-path validation notes
- rollback/upgrade notes
- any distro-specific caveats documented in
  [INSTALL.md](/home/vince/Projects/rsudoas/docs/INSTALL.md) and
  [OPERATIONS.md](/home/vince/Projects/rsudoas/docs/OPERATIONS.md)

Exit condition:

- at least the minimum environment matrix has passed with recorded evidence and
  no unresolved critical issues remain

Closes or advances:

- `P0.3` No Multi-Distro Runtime Validation
- `P0.4` No Real Install-Path Validation Outside Conformance
- part of `P1.9` Operator Docs Exist But Are Not Yet Operator-Validated
- part of `P2.12` Oracle Scope Is Still Upstream `OpenDoas`, Not Alpine-Packaged `doas`

## Phase 7: Release Gate, Soak, And Operational Validation

Objective:

- prove that the project can be released and used with an operational discipline
  appropriate for a privilege-escalation tool

Primary scope:

- [scripts/release-preflight.sh](/home/vince/Projects/rsudoas/scripts/release-preflight.sh)
- [docs/RELEASE-CHECKLIST.md](/home/vince/Projects/rsudoas/docs/RELEASE-CHECKLIST.md)
- [docs/VERSIONING.md](/home/vince/Projects/rsudoas/docs/VERSIONING.md)
- [CHANGELOG.md](/home/vince/Projects/rsudoas/CHANGELOG.md)
- CI/release policy

Questions:

- Does CI truly block release?
- Can a release be performed from a clean checkout and reproduced?
- Can operators install, upgrade, roll back, and observe the tool safely?
- Does a release-candidate soak period uncover operational defects?

Required evidence:

- one exercised release from a clean tree
- CI policy documented and enforced
- operator validation feedback or tested walkthroughs
- at least one release-candidate soak period with recorded findings

Exit condition:

- release process has been exercised end-to-end, CI is a real gate, and the
  soak period has completed without unresolved critical issues

Closes or advances:

- `P0.6` No Release Soak Period
- `P1.7` CI Is Green But Not Yet a Proven Release Gate
- `P1.8` Release Process Exists But Has Not Been Exercised End-To-End
- `P1.9` Operator Docs Exist But Are Not Yet Operator-Validated
- `P1.10` No Bug-Backfill Policy Has Been Exercised In Practice
- `P2.13` No Long-Run Operational Metrics Yet

## Recommended Execution Order

1. Phase 0
2. Phase 1
3. Phase 2
4. Phase 3
5. Phase 4
6. Phase 5
7. Phase 6
8. Phase 7

The production-readiness blind spot is not closed until:

- phases `1` through `5` are complete
- the minimum environment matrix in phase `6` is complete
- the release and soak work in phase `7` is complete

At that point, the project can be reassessed against
[PRODUCTION-READINESS.md](/home/vince/Projects/rsudoas/PRODUCTION-READINESS.md)
with evidence instead of optimism.
