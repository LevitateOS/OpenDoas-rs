# Conformance

This directory is the redesign point for `OpenDoas-rs` conformance testing.

The goal is to prove, in a repeatable way, how `OpenDoas-rs` behaves against
`OpenDoas` on Linux without leaking test-only hacks into the product build.

## Goals

- treat `OpenDoas` as the executable Linux oracle
- run the same end-to-end cases against `OpenDoas` and `OpenDoas-rs`
- capture real behavior, not just hand-written expectations
- keep the product tree and the conformance harness separate
- make parity gaps visible as reports, not as guesses

## Principles

- `OpenDoas-rs` code and conformance infrastructure are different concerns
- cases should be declarative first and imperative only where necessary
- the harness must distinguish `stdout`, `stderr`, and TTY output
- reference inputs should be pinned, fetched, and reproducible
- test-only dependency handling must stay inside conformance, not the root
  product manifest

## Proposed Layout

```text
conformance/
  README.md
  lock/
    opendoas.lock
    alpine.lock
  refs/
    fetch.sh
  images/
    opendoas/
      Containerfile
    opendoas-rs/
      Containerfile
  runner/
    Cargo.toml
    src/
  cases/
    cli/
    check/
    match/
    config/
    runtime/
    env/
    shell/
    auth/
    persist/
    logging/
    hardening/
  fixtures/
    users/
    env/
    files/
  artifacts/
    baselines/
    runs/
    diffs/
```

## Case Contract

Each case should live in its own directory and typically include:

- `case.toml`
- `doas.conf`
- `invoke.sh`
- optional `setup.sh`
- optional `stdin.txt`
- optional `expect.toml`

Each case should define:

- execution mode
- actor identity
- target identity
- auth backend
- tty requirement
- timestamp requirement
- comparison mode

## Comparison Modes

- `baseline`
  Run `OpenDoas` first and compare `OpenDoas-rs` to the observed result.
- `exact`
  Match fixed `exit`, `stdout`, `stderr`, and TTY output.
- `normalized`
  Compare after path/program-name normalization.
- `xfail`
  Track a known divergence explicitly.

## Runner Model

The harness should be a small standalone Rust tool under `conformance/runner`.

Responsibilities:

- load case metadata
- build or select the correct image
- run the case in an isolated Podman container
- capture `exit`, `stdout`, `stderr`, and TTY separately
- write JSON results
- compare against either `OpenDoas` baselines or fixed expectations

## Execution Flow

1. Fetch pinned reference inputs from `lock/`.
2. Build `OpenDoas` and `OpenDoas-rs` test images.
3. Run all cases against `OpenDoas`.
4. Store baseline results in `artifacts/baselines/`.
5. Run the same cases against `OpenDoas-rs`.
6. Compare and emit a parity report.

## Coverage Order

Build the suite in this order:

1. `cli`
2. `check`
3. `match`
4. `config`
5. `runtime`
6. `env`
7. `shell`
8. `auth`
9. `persist`
10. `logging`
11. `hardening`

## Backlogs

The current case tree is broad but not complete. Use these two files
deliberately:

- [Missing Cases](./MISSING-CASES.md)
  Prioritized summary of the next missing parity cases.
- [Additional OpenDoas Edge-Case Backlog](./EDGE-CASE-BACKLOG.md)
  Line-referenced source audit of further oracle-driven cases.

## Non-Goals

- no root-level Cargo patching just to satisfy conformance builds
- no merged output channels
- no hidden harness-only feature activation
- no markdown-heavy process before the runner and cases exist

## First Buildable Slice

The first useful rebuild of this system should be:

- one `OpenDoas` image
- one `OpenDoas-rs` image
- one runner binary
- one `cli` case
- one `check` case
- one `match` case
- one JSON result format

That is enough to prove the redesigned harness model before rebuilding the full
suite.
