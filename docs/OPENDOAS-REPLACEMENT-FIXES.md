# OpenDoas Replacement Fix List

This document captures the concrete fixes required before `OpenDoas-rs` can be
treated as a credible replacement for upstream `OpenDoas`.

Current assessment: `not replacement-ready`

This list is based on:

- manual code review of the privilege, auth, environment, logging, and policy
  paths
- direct local verification of several reproduced behaviors
- a full `conformance.py run-suite` execution on `2026-03-16`

## Priority Model

- `P0`
  Security or correctness blocker. Must be fixed before any replacement claim.
- `P1`
  Important parity or deployment gap. Should be fixed before real-world use.
- `P2`
  Hardening, cleanup, or evidence work that should follow the above.

## P0 Fixes

### 1. Remove the syslog format-string vulnerability

Status:

- open

Why it matters:

- attacker-controlled command lines and cwd values currently flow into
  `syslog(3)` as the C format string
- this is unacceptable in a setuid privilege-escalation tool

Current implementation:

- `src/logging/audit.rs`
- dependency call path:
  `syslog-c-0.1.3/src/lib.rs`

Required fix:

- stop using the message text as the format string
- either replace `syslog-c` entirely or wrap `libc::syslog()` with a constant
  format string such as `"%s"`
- re-check deny logging, permit logging, auth-failure logging, and tty-required
  logging

Required regression coverage:

- deny-path log with `%n` in argv
- permit-path log with `%n` in argv
- cwd containing `%` characters
- syslog capture cases that prove no crash and correct output

### 2. Remove privileged panics on non-UTF-8 environment values

Status:

- open

Why it matters:

- the setuid path currently reads the environment through
  `std::env::vars()`
- invalid Unicode in an environment value can panic the root process before
  auth or policy completes

Current implementation:

- `src/main.rs`

Required fix:

- replace `std::env::vars()` with `std::env::vars_os()` or a raw-byte
  environment collection path
- ensure all later env-processing code can handle non-UTF-8 safely
- keep behavior aligned with OpenDoas, which reads the raw environment

Required regression coverage:

- a runtime case that invokes `doas` with a non-UTF-8 env value and proves the
  binary exits cleanly instead of panicking

### 3. Make `keepenv` reject invalid and overlong variable names like OpenDoas

Status:

- open

Why it matters:

- OpenDoas ignores invalid or overlong inherited names
- the Rust implementation currently copies them under `keepenv`
- this is already visible in the checked-in failing case

Current implementation:

- `src/exec/env.rs`
- OpenDoas reference behavior:
  `.reference/OpenDoas/env.c`

Required fix:

- validate inherited env names before inserting them into the target env
- reject:
  - empty names
  - names without `=`
  - names longer than the OpenDoas limit
  - otherwise invalid env keys

Required regression coverage:

- `conformance/cases/env/keepenv-overlong-name-ignored`
- an invalid-name keepenv case if one does not already exist

### 4. Keep the project out of “replacement-ready” status until the full suite is green

Status:

- open

Why it matters:

- the current full conformance suite is not green
- a replacement claim is not credible while checked-in oracle cases still fail

Current failing cases from the `2026-03-16` full suite:

- `env/keepenv-display`
- `env/keepenv-overlong-name-ignored`
- `logging/permit-long-cmdline-truncation`
- `logging/permit-syslog`
- `runtime/path-component-too-long`
- `runtime/path-restored-when-cmd-omitted`

Required fix:

- close all current suite failures
- only revisit replacement claims after the suite is green again

## P1 Fixes

### 5. Restore OpenDoas PATH behavior when the rule omits `cmd`

Status:

- open

Why it matters:

- upstream OpenDoas restores the caller PATH when the matched rule does not pin
  a command
- the Rust implementation currently forces the safe PATH regardless
- this breaks real command lookup parity and is already visible in the suite

Current implementation:

- `src/exec/run.rs`
- `src/exec/env.rs`
- OpenDoas reference:
  `.reference/OpenDoas/doas.c`

Required fix:

- distinguish between:
  - command-restricted rules
  - unrestricted-command rules
- preserve and restore the former PATH for unrestricted-command execution
- keep the safe PATH for the restricted-command path

Required regression coverage:

- `conformance/cases/runtime/path-restored-when-cmd-omitted`

### 6. Preserve `ENAMETOOLONG` and related lookup failures during PATH search

Status:

- open

Why it matters:

- the current fallback search can collapse path-search errors into
  `command not found`
- OpenDoas preserves `path too long` in the relevant path-search case

Current implementation:

- `src/exec/spawn.rs`

Required fix:

- make PATH search preserve the strongest relevant lookup error
- do not silently demote `ENAMETOOLONG` into a generic not-found result

Required regression coverage:

- `conformance/cases/runtime/path-component-too-long`

### 7. Match OpenDoas permit-log wording and command-line truncation behavior

Status:

- open

Why it matters:

- current syslog permit messages do not match upstream wording
- current logged command lines are unbounded
- both are already causing checked-in suite failures

Current implementation:

- `src/logging/audit.rs`
- `src/policy/command.rs`
- OpenDoas reference:
  `.reference/OpenDoas/doas.c`

Required fix:

- change permit log text to match upstream expectations
- cap logged command-line length compatibly with OpenDoas behavior
- keep runtime execution separate from audit formatting

Required regression coverage:

- `conformance/cases/logging/permit-syslog`
- `conformance/cases/logging/permit-long-cmdline-truncation`

### 8. Normalize or control harness-only environment differences in env baselines

Status:

- open

Why it matters:

- `env/keepenv-display` currently differs due to environment noise that is not
  central to the feature under test
- if the harness baseline is too loose or too noisy, real parity signals get
  mixed with container-specific artifacts

Current implementation:

- `conformance/runner/bin/conformance.py`
- `conformance/cases/env/keepenv-display`

Required fix:

- choose one of:
  - normalize image-specific env entries
  - tighten the case to assert only the intended vars
  - install a stricter env sandbox for the invoke path

Required regression coverage:

- rerun `env/keepenv-display` until it compares only the intended behavior

### 9. Make the default auth backend, docs, and release process agree

Status:

- open

Why it matters:

- the code defaults to plain auth
- the docs say the default build is PAM
- CI and release-preflight do not build the PAM backend at all
- that creates a real operator risk for a setuid tool

Current implementation:

- `Cargo.toml`
- `build.rs`
- `.github/workflows/ci.yml`
- `scripts/release-preflight.sh`
- `README.md`
- `docs/INSTALL.md`

Required fix:

- decide what the actual supported default should be
- make:
  - feature defaults
  - build-time `AUTH_MODE`
  - installation docs
  - CI
  - release preflight
  all match that decision

Required regression coverage:

- successful CI build for the chosen default
- explicit host or container build for PAM
- explicit host or container build for plain auth

### 10. Make PAM service naming consistent across code, harness, and docs

Status:

- open

Why it matters:

- the code uses `doas`
- the docs tell operators to configure `opendoas-rs`
- the harness installs `/etc/pam.d/doas`
- following the docs today can configure the wrong PAM file

Current implementation:

- `src/auth/pam.rs`
- `conformance/runner/bin/conformance.py`
- `README.md`
- `docs/INSTALL.md`

Required fix:

- choose one PAM service name
- use it consistently in:
  - code
  - harness
  - install docs
  - operations docs

Required regression coverage:

- a documented clean install flow that works without hidden harness conventions

### 11. Make the PAM conversation flow more compatible with real PAM stacks

Status:

- open

Why it matters:

- the current implementation blindly answers every echo-on prompt with the
  source username
- that is narrower and less robust than upstream behavior

Current implementation:

- `src/auth/pam.rs`
- OpenDoas reference:
  `.reference/OpenDoas/pam.c`

Required fix:

- review whether the initial PAM user should be set at context creation
- handle echo-on prompts in a way that does not assume they are always asking
  for the username
- verify behavior against at least one real PAM stack outside the current
  narrow harness path

Required regression coverage:

- retain the existing prompt-rewrite test
- add at least one case covering a non-password echo-on conversation path if it
  can be modeled

### 12. Refresh PAM persist tickets only after the session supervision path is established

Status:

- open

Why it matters:

- the current PAM path refreshes the ticket before the fork/session-watch path
  fully succeeds
- upstream OpenDoas refreshes later

Current implementation:

- `src/main.rs`
- OpenDoas reference:
  `.reference/OpenDoas/pam.c`

Required fix:

- move timestamp refresh to the post-fork success path that matches OpenDoas
  semantics more closely

Required regression coverage:

- a targeted fault-injection case if practical
- otherwise at least a focused unit or integration test around the refresh point

### 13. Remove `expect()`-driven aborts from auth setup and prompting paths

Status:

- open

Why it matters:

- prompt or PAM initialization failures should degrade into controlled auth
  errors, not panics, in a setuid utility

Current implementation examples:

- `src/auth/plain.rs`
- `src/auth/pam.rs`

Required fix:

- replace `expect()` paths with error returns
- handle non-UTF-8 hostnames and PAM context startup failures gracefully

Required regression coverage:

- unit coverage for graceful error handling where practical

## P2 Fixes

### 14. Refresh the checked-in investigation and readiness docs

Status:

- open

Why it matters:

- some earlier investigation documents now describe bugs that have since been
  fixed
- other live blockers are more important than those stale findings

Current affected docs include:

- `docs/investigations/02-phase-2-config-policy.md`
- `docs/investigations/03-phase-3-auth-tty-session.md`
- `docs/PRODUCTION-READINESS.md`
- `docs/GAP-REGISTER.md`

Required fix:

- update stale findings that no longer match current code
- keep the live blockers in sync with the current implementation state

### 15. Expand CI and release preflight to cover the real supported matrix

Status:

- open

Why it matters:

- current CI does not prove the PAM path builds
- current release preflight mirrors that same narrow coverage

Current implementation:

- `.github/workflows/ci.yml`
- `scripts/release-preflight.sh`

Required fix:

- add PAM build coverage
- keep plain and none coverage if those remain supported
- include the full conformance suite and parser stress in the required release
  gate

### 16. Validate on real host environments, not only the current harness path

Status:

- open

Why it matters:

- replacement-readiness is not proven by Alpine/container-only evidence
- PAM, logging, install layout, and packaging behavior vary across hosts

Required fix:

- validate on at least two real Linux distributions
- validate setuid install, PAM setup, logging visibility, and rollback
- record the outcome in the docs

## Completion Criteria

This document should only be considered closed when all of the following are
true:

- all `P0` items are fixed
- the full checked-in conformance suite is green
- the supported auth story is internally consistent across code, CI, docs, and
  release scripts
- PAM and plain-auth builds are both intentionally supported or one is
  intentionally removed
- the project documentation no longer overstates replacement readiness

Until then, `OpenDoas-rs` should not be described as an OpenDoas replacement.
