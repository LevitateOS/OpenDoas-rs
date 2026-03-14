# Phase 2: Config And Policy Audit

## Objective

- verify that config parsing, validation, and policy matching are safe and aligned with the current OpenDoas oracle behavior
- determine whether the Phase 2 surface is ready to sign off

## Scope

- [src/config/lexer.rs](/home/vince/Projects/rsudoas/src/config/lexer.rs#L1)
- [src/config/parser.rs](/home/vince/Projects/rsudoas/src/config/parser.rs#L1)
- [src/config/validate.rs](/home/vince/Projects/rsudoas/src/config/validate.rs#L1)
- [src/policy/matcher.rs](/home/vince/Projects/rsudoas/src/policy/matcher.rs#L1)
- [src/policy/identity.rs](/home/vince/Projects/rsudoas/src/policy/identity.rs#L1)
- [src/policy/command.rs](/home/vince/Projects/rsudoas/src/policy/command.rs#L1)
- [src/policy/rule.rs](/home/vince/Projects/rsudoas/src/policy/rule.rs#L1)
- Supporting flow reads used to confirm runtime effect and oracle intent:
  [src/app/execute.rs](/home/vince/Projects/rsudoas/src/app/execute.rs#L1),
  [src/policy/decision.rs](/home/vince/Projects/rsudoas/src/policy/decision.rs#L1),
  [src/exec/env.rs](/home/vince/Projects/rsudoas/src/exec/env.rs#L1),
  [.reference/OpenDoas/parse.y](/home/vince/Projects/rsudoas/.reference/OpenDoas/parse.y#L1),
  [.reference/OpenDoas/doas.conf.5](/home/vince/Projects/rsudoas/.reference/OpenDoas/doas.conf.5#L1),
  [.reference/OpenDoas/doas.c](/home/vince/Projects/rsudoas/.reference/OpenDoas/doas.c#L150)

## Commands Run

```sh
wc -l src/config/lexer.rs src/config/parser.rs src/config/validate.rs src/policy/matcher.rs src/policy/identity.rs src/policy/command.rs src/policy/rule.rs
git status --short -- docs/investigations/02-phase-2-config-policy.md src/config/lexer.rs src/config/parser.rs src/config/validate.rs src/policy/matcher.rs src/policy/identity.rs src/policy/command.rs src/policy/rule.rs
nl -ba src/config/lexer.rs
nl -ba src/config/parser.rs
nl -ba src/config/validate.rs
nl -ba src/policy/matcher.rs
nl -ba src/policy/identity.rs
nl -ba src/policy/command.rs
nl -ba src/policy/rule.rs
rg -n "validate_runtime_config_metadata|try_from\\(|Rules::|permit_opts|setenv|keepenv|nopass|persist|nolog|matches_target|get_cmdline|command:|args:" src
nl -ba src/policy/decision.rs
nl -ba src/app/execute.rs
nl -ba src/exec/env.rs
cargo test --quiet
nl -ba .reference/OpenDoas/parse.y | sed -n '1,360p'
nl -ba .reference/OpenDoas/doas.conf.5 | sed -n '1,260p'
nl -ba .reference/OpenDoas/doas.c | sed -n '150,190p'
cargo run --quiet -- -C conformance/cases/check/backslash-newline-continuation/doas.conf -u root /usr/bin/printf hello
id -u
tmp=$(mktemp); printf 'permit nopass vince args -u\n' > "$tmp"; cargo run --quiet -- -C "$tmp" /usr/bin/id -u; rm -f "$tmp"
tmp=$(mktemp); printf 'permit nopass vince args -u\n' > "$tmp"; cargo run --quiet -- -C "$tmp" /bin/echo -u; rm -f "$tmp"
tmp=$(mktemp); printf 'permit nopass vince cmd /usr/bin/id as root\n' > "$tmp"; cargo run --quiet -- -C "$tmp" -u root /usr/bin/id; rm -f "$tmp"
tmp=$(mktemp); printf 'permit nopass vince cmd /bin/echo cmd /usr/bin/id\n' > "$tmp"; cargo run --quiet -- -C "$tmp" /usr/bin/id; rm -f "$tmp"
tmp=$(mktemp); printf 'permit nopass vince as root as 1000 cmd /usr/bin/id\n' > "$tmp"; cargo run --quiet -- -C "$tmp" -u 1000 /usr/bin/id; rm -f "$tmp"
tmp=$(mktemp); python3 - "$tmp" <<'PY'
from pathlib import Path
import sys
Path(sys.argv[1]).write_bytes(b'permit nopass v\\x00ince as root cmd /usr/bin/id\\n')
PY
cargo run --quiet -- -C "$tmp"
rm -f "$tmp"
tmp=$(mktemp); python3 - "$tmp" <<'PY'
from pathlib import Path
import sys
Path(sys.argv[1]).write_bytes(b'permit nopass setenv { FOO=bar\\x00baz } vince as root cmd /usr/bin/id\\n')
PY
cargo run --quiet -- -C "$tmp"
rm -f "$tmp"
tmp=$(mktemp); python3 - "$tmp" <<'PY'
from pathlib import Path
import sys
line = 'permit nopass vince as root cmd /usr/bin/' + ('x' * 1100) + '\\n'
Path(sys.argv[1]).write_text(line)
PY
cargo run --quiet -- -C "$tmp"
rm -f "$tmp"
rg -n "Tokenizer|lexer" src
nl -ba src/config/mod.rs
```

## Findings

1. High: the structural clause parser is too permissive and can silently widen or rewrite a rule.

   [`parse_rule_line()`](/home/vince/Projects/rsudoas/src/config/parser.rs#L34) treats `as`, `cmd`, and `args` as a free-form loop instead of the oracle grammar. That has two concrete effects. First, `args` is accepted without any preceding `cmd`, which leaves [`rule.command`](/home/vince/Projects/rsudoas/src/policy/matcher.rs#L54) unset and turns the rule into "any command whose argv exactly matches this list". Second, reordered or repeated `as` and `cmd` clauses are accepted and later occurrences silently overwrite earlier ones.

   Reproductions from this audit:
   `permit nopass vince args -u` returned `permit nopass` for both `/usr/bin/id -u` and `/bin/echo -u`.
   `permit nopass vince cmd /bin/echo cmd /usr/bin/id` returned `permit nopass` for `/usr/bin/id`.
   `permit nopass vince cmd /usr/bin/id as root` returned `permit nopass` even though `cmd` appeared before `as`.

   Refs: [src/config/parser.rs](/home/vince/Projects/rsudoas/src/config/parser.rs#L96), [src/config/parser.rs](/home/vince/Projects/rsudoas/src/config/parser.rs#L104), [src/config/parser.rs](/home/vince/Projects/rsudoas/src/config/parser.rs#L110), [src/policy/matcher.rs](/home/vince/Projects/rsudoas/src/policy/matcher.rs#L54), [src/policy/matcher.rs](/home/vince/Projects/rsudoas/src/policy/matcher.rs#L59), [.reference/OpenDoas/parse.y](/home/vince/Projects/rsudoas/.reference/OpenDoas/parse.y#L175), [.reference/OpenDoas/parse.y](/home/vince/Projects/rsudoas/.reference/OpenDoas/parse.y#L181), [.reference/OpenDoas/parse.y](/home/vince/Projects/rsudoas/.reference/OpenDoas/parse.y#L189)

2. Medium: escaped newline continuation is missing, so valid OpenDoas configs are rejected.

   [`Rules::try_from()`](/home/vince/Projects/rsudoas/src/config/parser.rs#L16) splits the whole file with `config.lines()` before tokenization. Once that happens, a trailing backslash can only become [`unterminated escape`](/home/vince/Projects/rsudoas/src/config/parser.rs#L203); the tokenizer never sees the following line as part of the same logical token stream. Upstream OpenDoas explicitly treats backslash-newline as continuation outside comments.

   Reproduction from this audit:
   `cargo run --quiet -- -C conformance/cases/check/backslash-newline-continuation/doas.conf -u root /usr/bin/printf hello` failed with `Error parsing config: unterminated escape at line 1`.

   Refs: [src/config/parser.rs](/home/vince/Projects/rsudoas/src/config/parser.rs#L19), [src/config/parser.rs](/home/vince/Projects/rsudoas/src/config/parser.rs#L21), [src/config/parser.rs](/home/vince/Projects/rsudoas/src/config/parser.rs#L153), [src/config/parser.rs](/home/vince/Projects/rsudoas/src/config/parser.rs#L187), [src/config/parser.rs](/home/vince/Projects/rsudoas/src/config/parser.rs#L203), [.reference/OpenDoas/parse.y](/home/vince/Projects/rsudoas/.reference/OpenDoas/parse.y#L270), [.reference/OpenDoas/parse.y](/home/vince/Projects/rsudoas/.reference/OpenDoas/parse.y#L275), [.reference/OpenDoas/parse.y](/home/vince/Projects/rsudoas/.reference/OpenDoas/parse.y#L279)

3. Medium: NUL bytes inside words are accepted instead of being rejected deterministically.

   The load path reads config data into a Rust [`String`](/home/vince/Projects/rsudoas/src/app/execute.rs#L49), and [`tokenize_line()`](/home/vince/Projects/rsudoas/src/config/parser.rs#L153) has no NUL rejection branch. A `\0` byte outside comments therefore becomes ordinary token content via the default [`current.push(chr)`](/home/vince/Projects/rsudoas/src/config/parser.rs#L199) path. OpenDoas rejects NUL in words at lex time.

   Reproductions from this audit:
   a config containing `v\x00ince` in the identity field exited `0` under `doas -C`.
   a config containing `setenv { FOO=bar\x00baz }` also exited `0` under `doas -C`.

   Refs: [src/app/execute.rs](/home/vince/Projects/rsudoas/src/app/execute.rs#L49), [src/config/parser.rs](/home/vince/Projects/rsudoas/src/config/parser.rs#L173), [src/config/parser.rs](/home/vince/Projects/rsudoas/src/config/parser.rs#L180), [src/config/parser.rs](/home/vince/Projects/rsudoas/src/config/parser.rs#L199), [.reference/OpenDoas/parse.y](/home/vince/Projects/rsudoas/.reference/OpenDoas/parse.y#L265)

4. Low: the parser has no line-length bound and accepts overlong logical lines that the oracle rejects.

   [`tokenize_line()`](/home/vince/Projects/rsudoas/src/config/parser.rs#L153) appends characters into an unbounded [`String`](/home/vince/Projects/rsudoas/src/config/parser.rs#L155) and never enforces a maximum line length. OpenDoas uses a fixed 1024-byte lexer buffer and emits `too long line` when that bound is exceeded. The Rust implementation therefore removes a simple parser resource bound and accepts configs that the oracle treats as invalid.

   Reproduction from this audit:
   a temporary config with `cmd /usr/bin/` followed by `1100` `x` bytes exited `0` under `doas -C`.

   Refs: [src/config/parser.rs](/home/vince/Projects/rsudoas/src/config/parser.rs#L153), [src/config/parser.rs](/home/vince/Projects/rsudoas/src/config/parser.rs#L155), [src/config/parser.rs](/home/vince/Projects/rsudoas/src/config/parser.rs#L173), [src/config/parser.rs](/home/vince/Projects/rsudoas/src/config/parser.rs#L199), [.reference/OpenDoas/parse.y](/home/vince/Projects/rsudoas/.reference/OpenDoas/parse.y#L229), [.reference/OpenDoas/parse.y](/home/vince/Projects/rsudoas/.reference/OpenDoas/parse.y#L315)

No independent correctness defects were found in [src/config/validate.rs](/home/vince/Projects/rsudoas/src/config/validate.rs#L1), [src/policy/identity.rs](/home/vince/Projects/rsudoas/src/policy/identity.rs#L1), [src/policy/command.rs](/home/vince/Projects/rsudoas/src/policy/command.rs#L1), or [src/policy/rule.rs](/home/vince/Projects/rsudoas/src/policy/rule.rs#L1) during this pass. The blocking issues in this phase are parser-driven.

## Remaining Risks

- [src/config/lexer.rs](/home/vince/Projects/rsudoas/src/config/lexer.rs#L1) is currently not used by the live parser path, and it already diverges materially from [`src/config/parser.rs`](/home/vince/Projects/rsudoas/src/config/parser.rs#L153) on tokenization details such as tab handling, brace tokens, and quote or escape failure behavior. That dead-code split is a maintenance risk even though it was not the active source of the findings above.
- `cargo test --quiet` reported `0` tests. The parser and matcher edge cases above currently rely on manual `-C` reproductions rather than regression coverage.
- This was not a full harness run or a setuid runtime audit. The Phase 2 conclusions are based on static review plus direct `doas -C` style reproductions.

## Exit Decision

Phase 2 is not ready to sign off.

The current parser accepts multiple non-oracle rule forms, and one of them can materially widen authorization by turning `args` into a wildcard-command permit. Config and policy validation should stay open until the parser grammar is tightened, the malformed-input cases above are rejected deterministically, and regression coverage exists for the reproduced cases.
