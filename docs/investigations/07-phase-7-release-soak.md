# Phase 7: Release Gate, Soak, And Operational Validation

## Objective

- prove that `OpenDoas-rs` can be released and operated with the discipline
  expected for a privilege-escalation tool

## Scope

- [scripts/release-preflight.sh](/home/vince/Projects/rsudoas/scripts/release-preflight.sh)
- [docs/RELEASE-CHECKLIST.md](/home/vince/Projects/rsudoas/docs/RELEASE-CHECKLIST.md)
- [docs/VERSIONING.md](/home/vince/Projects/rsudoas/docs/VERSIONING.md)
- [CHANGELOG.md](/home/vince/Projects/rsudoas/CHANGELOG.md)
- CI and release policy

## Commands Run

Not started yet.

## Findings

- Not investigated yet.

## Remaining Risks

- No exercised release from a clean tree is recorded yet.
- CI is green, but release gating policy and branch protection are not recorded
  as evidence in this phase yet.
- No release-candidate soak period is recorded yet.
- No operator-validation feedback is attached to the current install and
  rollback documentation.

## Exit Decision

Open.

This phase should stay open until at least one full release rehearsal and one
documented soak period have completed without unresolved critical findings.
