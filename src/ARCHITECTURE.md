# Target Source Architecture

This is the target `src/` layout for parity-driven work against the
conformance suite.

The goal is not to mirror test-family names mechanically. The goal is to map
stable behavior domains into stable Rust modules.

## Current State

The current live implementation is still concentrated in:

- `src/lib.rs`
- `src/main.rs`

That path remains authoritative until code is migrated into the new tree.

## Target Tree

```text
src/
  main.rs
  lib.rs
  ARCHITECTURE.md

  app/
    mod.rs
    check.rs
    execute.rs

  auth/
    mod.rs
    none.rs
    pam.rs
    plain.rs

  cli/
    mod.rs
    args.rs
    mode.rs
    usage.rs

  config/
    mod.rs
    ast.rs
    lexer.rs
    parser.rs
    validate.rs

  exec/
    mod.rs
    env.rs
    path.rs
    privilege.rs
    run.rs
    shell.rs
    spawn.rs

  logging/
    mod.rs
    audit.rs

  persist/
    mod.rs
    deauth.rs
    timestamp.rs

  platform/
    mod.rs
    groups.rs
    passwd.rs
    tty.rs

  policy/
    mod.rs
    command.rs
    decision.rs
    identity.rs
    matcher.rs
    rule.rs
```

## Ownership

- `cli/`: parse argv, usage errors, mode selection
- `config/`: tokenize, parse, and validate config text and config file safety
- `policy/`: ordered rules, identity matching, command matching, and final
  authorization decisions
- `auth/`: backend-specific authentication behavior
- `exec/`: runtime execution, environment construction, shell mode, privilege
  switching, and spawning
- `persist/`: timestamps and `-L`
- `logging/`: audit/log messages and logging policy
- `platform/`: passwd, groups, and tty lookups
- `app/`: high-level orchestration for `check` and `execute`

## Conformance Mapping

- `cli` cases map to `cli/`
- `check` cases map to `cli/`, `config/`, `policy/`, and `app/check.rs`
- `match` cases map to `policy/`
- `config` cases map to `config/validate.rs`
- `runtime`, `env`, and `shell` cases map to `exec/`
- `auth` cases map to `auth/`
- `persist` cases map to `persist/`
- `logging` cases map to `logging/`

## Migration Direction

The first migrations should be structural, not cosmetic:

1. move command-line mode parsing into `cli/`
2. replace split `allowed` and `denied` rule storage with one ordered rule list
   in `policy/`
3. make matching return a full decision, not only rule options
4. split config parsing from config file validation
5. move execution concerns out of `main.rs` into `exec/` and `app/`

That sequence matches the highest-value parity failures already exposed by the
conformance suite.
