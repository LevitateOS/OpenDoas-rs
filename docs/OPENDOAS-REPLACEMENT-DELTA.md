# OpenDoas Replacement Delta

This document captures the delta between the current `OpenDoas-rs` state and
an honest "good enough OpenDoas replacement" claim.

Review date: `2026-03-16`

Current assessment: `not good enough yet`

Related project docs:

- [Production Readiness](./../PRODUCTION-READINESS.md)
- [Gap Register](./GAP-REGISTER.md)
- [Replacement Fix List](./OPENDOAS-REPLACEMENT-FIXES.md)
- [Security Review Checklist](./SECURITY-REVIEW.md)

## What "Good Enough" Means

For a tool in this class, "good enough OpenDoas replacement" means all of the
following are true:

- no known privilege-boundary hardening gaps remain in the supported path
- behavior matches upstream OpenDoas closely enough that no known replacement-
  blocking parity gaps remain
- packaging, PAM, configuration, and install guidance are internally
  consistent
- the verification evidence is strong enough to trust the result outside the
  current container harness

Passing a partial conformance corpus is not enough on its own.

For the product gaps below, the "How OpenDoas handles it" notes point at the
vendored upstream reference under [../.reference/OpenDoas](../.reference/OpenDoas).

## Current Verdict

The project is serious and already has real parity work behind it, but it is
still short of replacement-ready. The remaining delta is not mostly about new
features. It is about:

- byte-exact Unix behavior
- privilege-boundary hardening
- execution-model parity
- clean packaging and PAM behavior
- stronger release evidence

## Product Delta

These are the product changes still needed before a replacement claim is
credible.

### 1. Make Config And Argv Handling Byte-Safe

Priority: `P0`

Why this blocks replacement:

- OpenDoas operates on Unix byte strings
- `OpenDoas-rs` still treats config and argv as UTF-8 text in multiple core
  paths
- that makes it impossible to be a byte-exact OpenDoas replacement

Current implementation:

- config load uses `read_to_string()` in [src/app/execute.rs](../src/app/execute.rs)
- parser tokens and rules use `String` in [src/config/parser.rs](../src/config/parser.rs) and
  [src/policy/rule.rs](../src/policy/rule.rs)
- argv is lossily converted with `to_string_lossy()` in
  [src/cli/args.rs](../src/cli/args.rs)

Required change:

- move config parsing, rule storage, matching, and execution boundaries to
  byte-safe or `OsStr`-safe representations
- stop lossy conversion at the CLI boundary
- make config parsing accept the same byte surface that upstream accepts

How OpenDoas handles it:

- the parser reads raw bytes incrementally with `getc()` in
  [.reference/OpenDoas/parse.y](../.reference/OpenDoas/parse.y)
- argv stays as raw `char **argv` through matching and `execvpe()` in
  [.reference/OpenDoas/doas.c](../.reference/OpenDoas/doas.c)

Minimum regression coverage:

- non-UTF-8 config token accepted by upstream OpenDoas
- non-UTF-8 command path and non-UTF-8 argument round-trip cases
- explicit comparison against upstream behavior for those cases

### 2. Close Inherited File Descriptors Before Privileged Work

Priority: `P0`

Why this blocks replacement:

- the current binary keeps inherited descriptors open through config loading,
  auth setup, and session work
- upstream OpenDoas closes everything above `stderr` at process start
- the current fd test only proves the executed child is clean, not the
  privileged parent path

Current implementation:

- privileged flow begins in [src/main.rs](../src/main.rs)
- descriptor closing happens only in the final spawn path in
  [src/exec/spawn.rs](../src/exec/spawn.rs)
- upstream closes descriptors immediately in
  [.reference/OpenDoas/doas.c](../.reference/OpenDoas/doas.c)

Required change:

- perform close-from behavior before config/auth/session work begins
- keep only explicitly needed descriptors open
- add targeted tests for auth/session paths, not only the executed child

How OpenDoas handles it:

- it calls `closefrom(STDERR_FILENO + 1)` immediately on startup in
  [.reference/OpenDoas/doas.c](../.reference/OpenDoas/doas.c)
- config parsing, auth, and execution all happen after that early close

### 3. Fix Parser Parity For Escaping

Priority: `P0`

Why this blocks replacement:

- valid OpenDoas configs using backslash escaping outside quotes are rejected
- that is a direct parser compatibility break for a drop-in replacement claim

Current implementation:

- tokenizer escape handling in [src/config/parser.rs](../src/config/parser.rs)
- upstream reference behavior in [.reference/OpenDoas/parse.y](../.reference/OpenDoas/parse.y)

Required change:

- implement upstream backslash behavior, not only backslash-newline
  continuation
- verify exact behavior around spaces, keywords, comments, and quoted strings

How OpenDoas handles it:

- the lexer in [.reference/OpenDoas/parse.y](../.reference/OpenDoas/parse.y)
  toggles escape state on backslash and accepts escaped characters generally
  outside comments
- newline after backslash is a special continuation case, not the only
  supported escape form

Minimum regression coverage:

- escaped space in `args`
- escaped keyword text
- parser-stress cases covering both accepted and rejected escape forms

### 4. Remove Mandatory Reverse Group Lookup From Authorization

Priority: `P1`

Why this matters:

- the current implementation aborts if any supplementary GID lacks a
  resolvable group name
- OpenDoas matches numerically from `getgroups()` and does not require every
  group to reverse-resolve
- this creates a real compatibility problem on systems with NSS drift or
  partial group state

Current implementation:

- [src/platform/groups.rs](../src/platform/groups.rs)
- caller path in [src/main.rs](../src/main.rs)

Required change:

- treat numeric group IDs as authoritative for matching
- resolve names only when useful, not as a hard prerequisite for every command

How OpenDoas handles it:

- it reads supplementary groups with `getgroups()` in
  [.reference/OpenDoas/doas.c](../.reference/OpenDoas/doas.c)
- group matching in the `match()` function compares numeric gids directly after
  parsing the rule group in the same file

Minimum regression coverage:

- a case where one supplementary GID has no reverse name entry
- numeric group rule still matches correctly

### 5. Decide And Lock The Exec Model

Priority: `P1`

Why this matters:

- non-PAM execution currently uses a supervising wrapper that `posix_spawn`s
  and waits
- upstream OpenDoas `exec`s into the authorized command
- this changes signal and process-lifetime behavior

Current implementation:

- [src/exec/run.rs](../src/exec/run.rs)
- [src/exec/spawn.rs](../src/exec/spawn.rs)
- upstream reference:
  [.reference/OpenDoas/doas.c](../.reference/OpenDoas/doas.c)

Required change:

- either move closer to upstream `exec` semantics or explicitly scope and test
  the divergence
- add coverage for wrapper death, signal handling, and orphaned child behavior

How OpenDoas handles it:

- the normal execution path ends in direct `execvpe(cmd, argv, envp)` in
  [.reference/OpenDoas/doas.c](../.reference/OpenDoas/doas.c)
- the PAM path is the only place where upstream deliberately forks a parent
  watcher, in [.reference/OpenDoas/pam.c](../.reference/OpenDoas/pam.c)

### 6. Remove Panic Paths From Privileged Auth Flow

Priority: `P1`

Why this matters:

- privileged auth paths still abort on `expect()` for hostname and PAM context
  assumptions
- even when the impact is "only" denial-of-service, that is poor engineering
  for a setuid replacement

Current implementation:

- [src/auth/plain.rs](../src/auth/plain.rs)
- [src/auth/pam.rs](../src/auth/pam.rs)

Required change:

- convert all privileged auth assumptions into normal error returns
- ensure failure messages stay consistent with upstream expectations where
  relevant

How OpenDoas handles it:

- the shadow path in [.reference/OpenDoas/shadow.c](../.reference/OpenDoas/shadow.c)
  falls back to `?` for hostname and returns explicit auth failures instead of
  assuming valid UTF-8 host data
- the PAM path in [.reference/OpenDoas/pam.c](../.reference/OpenDoas/pam.c)
  follows the same pattern and has explicit cleanup on auth and session
  failures

### 7. Make Build, PAM, And Docs Tell The Same Story

Priority: `P1`

Why this matters:

- the default auth mode, PAM service name, and build knobs are still
  inconsistent across code and docs
- operators should not have to guess whether the correct service is `doas` or
  `opendoas-rs`

Current implementation:

- default feature selection in [Cargo.toml](../Cargo.toml)
- PAM service name in [src/auth/pam.rs](../src/auth/pam.rs)
- install guidance in [README.md](../README.md) and [docs/INSTALL.md](./INSTALL.md)
- build env knobs in [build.rs](../build.rs)

Required change:

- choose one supported default auth story
- choose one PAM service name
- remove dead build knobs or wire them up correctly
- make code, install docs, CI, and container images consistent

How OpenDoas handles it:

- upstream uses one PAM service constant, `doas`, in
  [.reference/OpenDoas/pam.c](../.reference/OpenDoas/pam.c)
- the config path is compiled in as `DOAS_CONF` and used consistently from
  [.reference/OpenDoas/doas.c](../.reference/OpenDoas/doas.c)
- distro-level customization is handled in packaging, not by conflicting
  stories inside the binary and docs; see
  [../.reference/Alpine-doas-3.23-stable](../.reference/Alpine-doas-3.23-stable)

## Evidence Delta

Even after the code issues above are fixed, the project still needs better
evidence before a replacement claim is honest.

### 8. Make The Conformance Harness Clean And Hermetic

Priority: `P0`

Why this matters:

- the current harness is not yet strong enough to serve as final release
  evidence
- local verification exposed environment contamination in the subject image
- clean results matter more here than a large but noisy suite

Current implementation:

- runner: [conformance/runner/bin/conformance.py](../conformance/runner/bin/conformance.py)
- parser stress: [conformance/runner/bin/parser_stress.py](../conformance/runner/bin/parser_stress.py)
- subject image: [conformance/images/opendoas-rs/Containerfile](../conformance/images/opendoas-rs/Containerfile)

Required change:

- eliminate image-specific env leakage from runtime env tests
- make parser stress clean against the oracle
- ensure the runner exercises the intended checked-in corpus consistently
- make CI failure block replacement or release claims

How OpenDoas helps here:

- upstream source does not solve hermetic CI by itself
- the practical use of OpenDoas here is as the oracle executable and source
  reference that the harness compares against

### 9. Add Missing Negative Tests Around Privileged Paths

Priority: `P0`

Why this matters:

- the highest-value missing tests are around privilege-boundary failure paths,
  not only parser fuzz
- current checked-in coverage does not prove the parent/auth/session path is as
  clean as the executed child

Required coverage additions:

- early inherited-fd closure
- reverse-group-lookup failure handling
- non-UTF-8 config and argv parity
- wrapper-vs-exec runtime semantics
- PAM and persist failure injection

How OpenDoas helps here:

- the upstream source identifies exactly which behaviors to test, especially in
  [.reference/OpenDoas/doas.c](../.reference/OpenDoas/doas.c),
  [.reference/OpenDoas/env.c](../.reference/OpenDoas/env.c),
  [.reference/OpenDoas/pam.c](../.reference/OpenDoas/pam.c), and
  [.reference/OpenDoas/timestamp.c](../.reference/OpenDoas/timestamp.c)

### 10. Validate Real Install Paths On Real Systems

Priority: `P0`

Why this matters:

- container conformance is useful but not enough for a setuid replacement
- the project still needs real install-path validation outside the harness

Minimum matrix:

- Alpine
- Debian or Ubuntu
- Arch
- `auth-plain`
- `auth-pam`
- timestamp `off`
- timestamp `on`

Required validations:

- setuid install
- PAM configuration
- runtime behavior
- logging visibility
- upgrade and rollback safety

How OpenDoas helps here:

- this is mostly outside what upstream source code can prove
- the best local references are the vendored upstream source plus distro
  packaging examples such as
  [../.reference/Alpine-doas-3.23-stable](../.reference/Alpine-doas-3.23-stable)

### 11. Complete A Real Release Gate

Priority: `P1`

Why this matters:

- a privilege-escalation tool should not be called replacement-ready without a
  clean release gate

Required evidence:

- release artifacts built from a clean tree
- CI gates that block release on failures
- at least one release-candidate soak period without critical regressions

How OpenDoas helps here:

- upstream code does not solve release discipline
- this is replacement-program work that must be supplied by this project

## Shortest Credible Path

If the goal is the shortest honest path to "good enough OpenDoas replacement,"
the work should happen in this order:

1. Fix byte handling and parser parity.
2. Close descriptors early and remove panic paths from privileged code.
3. Resolve group-matching and exec-model parity gaps.
4. Align build defaults, PAM service naming, and install docs.
5. Make the conformance harness clean and CI-blocking.
6. Run a real distro and install matrix.
7. Complete a release-candidate soak period.

## Practical Summary

The remaining delta is not mostly missing features. It is trustworthiness work
at the privilege boundary plus the evidence needed to justify a replacement
claim.

Until the `P0` items above are closed, the strongest honest claim remains:

- `OpenDoas-rs` has meaningful parity work and useful conformance evidence, but
  it is not yet good enough to be presented as a replacement for OpenDoas.
