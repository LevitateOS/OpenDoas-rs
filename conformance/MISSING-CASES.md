# Missing Conformance Cases

This is the current source-audited backlog of parity cases that are still not
represented in the recovered conformance tree.

Scope:
- the existing suite already covers many happy-path and first-wave edge cases
- this list is only for additional cases that are still missing
- references point at the current OpenDoas oracle source in
  `.reference/OpenDoas/`

## Priority 0

These are the highest-value missing cases because they exercise explicit
OpenDoas branches with user-visible behavior.

### CLI and Identity

- `cli/check-plus-shell-usage`
  `doas -C /etc/doas.conf -s` must fail with usage.
  Source: `.reference/OpenDoas/doas.c`

- `cli/shell-plus-command-usage`
  `doas -s /bin/echo hi` must fail with usage.
  Source: `.reference/OpenDoas/doas.c`

- `cli/no-passwd-entry-for-self`
  If the invoking uid has no passwd entry, `doas` must fail before check/runtime
  execution with `no passwd entry for self`.
  Source: `.reference/OpenDoas/doas.c`

- `runtime/no-passwd-entry-for-target`
  A numeric `-u` target with no passwd entry must fail with `no passwd entry for target`.
  Source: `.reference/OpenDoas/doas.c`

- `persist/deauth-no-ticket-succeeds`
  `doas -L` must succeed when no ticket exists.
  Source: `.reference/OpenDoas/doas.c`, `.reference/OpenDoas/timestamp.c`

### Parser and Lexer

- `check/too-long-line`
  Overlong tokens must report `too long line`.
  Source: `.reference/OpenDoas/parse.y`

- `check/unterminated-escape`
  A trailing backslash at EOF must report `unterminated escape`.
  Source: `.reference/OpenDoas/parse.y`

- `check/backslash-newline-continuation`
  Backslash-newline inside a token must continue onto the next line.
  Source: `.reference/OpenDoas/parse.y`

- `check/quoted-keyword-is-string`
  Quoted keywords like `"as"` or `"cmd"` must be treated as plain strings.
  Source: `.reference/OpenDoas/parse.y`

- `check/escaped-keyword-is-string`
  Escaped keywords like `a\\s` must be treated as plain strings.
  Source: `.reference/OpenDoas/parse.y`

- `check/empty-setenv-section`
  `setenv {}` must parse successfully.
  Source: `.reference/OpenDoas/parse.y`

- `match/empty-arg-exact-match`
  `args ""` must match exactly one empty argument and reject other argv shapes.
  Source: `.reference/OpenDoas/parse.y`, `.reference/OpenDoas/doas.c`

- `check/nul-inside-comment-is-ignored`
  NUL bytes inside a `#` comment should not trigger the token-level NUL error.
  Source: `.reference/OpenDoas/parse.y`

### Matching and Policy

- `match/unrestricted-command-rule`
  A rule without `cmd` must permit any command for the matched identity/target.
  Source: `.reference/OpenDoas/doas.c`

- `match/default-root-target`
  A rule without `as` must match the default target uid `0`.
  Source: `.reference/OpenDoas/doas.c`

- `match/numeric-gid-group-match`
  Group rules like `:3000` must match by numeric gid as well as by name.
  Source: `.reference/OpenDoas/doas.c`

- `match/relative-vs-absolute-command`
  `cmd /usr/bin/id` must not match `id`, and vice versa.
  Source: `.reference/OpenDoas/doas.c`

### Runtime and Environment

- `runtime/path-poisoning-ignored-when-cmd-present`
  When a rule has `cmd`, command lookup must use the safe path, not caller PATH.
  Source: `.reference/OpenDoas/doas.c`

- `runtime/path-restored-when-cmd-omitted`
  When a rule omits `cmd`, execution should use the caller/restored PATH.
  Source: `.reference/OpenDoas/doas.c`, `.reference/OpenDoas/env.c`

- `logging/permit-log-cwd-failed`
  If `getcwd()` fails, the permit log must use `(failed)` for the cwd field.
  Source: `.reference/OpenDoas/doas.c`

- `config/unreadable-explicit-config`
  `doas -C /path/to/conf` must surface explicit-config read failures.
  Source: `.reference/OpenDoas/doas.c`

- `config/config-path-is-directory`
  An explicit config path that is a directory must fail through the open/fopen path.
  Source: `.reference/OpenDoas/doas.c`

- `env/keepenv-does-not-override-target-core-vars`
  `keepenv` must not override `DOAS_USER`, `HOME`, `LOGNAME`, `USER`, `SHELL`, or
  the default `PATH` prepared for the target.
  Source: `.reference/OpenDoas/env.c`

- `env/setenv-last-wins`
  Repeated entries in one `setenv { ... }` block must be order-sensitive, with later
  entries overriding or deleting earlier ones.
  Source: `.reference/OpenDoas/env.c`

- `env/setenv-missing-source-drops-var`
  `setenv { FOO=$MISSING }` must leave `FOO` unset.
  Source: `.reference/OpenDoas/env.c`

- `env/setenv-path-with-no-incoming-path`
  `setenv { PATH }` must use the saved former PATH behavior even when the caller PATH
  is absent.
  Source: `.reference/OpenDoas/env.c`, `.reference/OpenDoas/doas.c`

### Auth, PAM, and TTY

- `auth/shadow-tty-required-log`
  No-TTY shadow auth must emit the exact `tty required for <user>` syslog entry and
  fail with `a tty is required`.
  Source: `.reference/OpenDoas/shadow.c`

- `auth/shadow-nonshadowed-passwd-rejected`
  A passwd entry with a non-`x`, non-`*` hash must fail with `Authentication failed`.
  Source: `.reference/OpenDoas/shadow.c`

- `auth/shadow-missing-shadow-entry`
  `pw_passwd == "x"` but no shadow entry must fail with `Authentication failed`.
  Source: `.reference/OpenDoas/shadow.c`

- `auth/pam-acct-mgmt-failure`
  PAM account-management denial must log `failed auth for <user>` and fail with
  `Authentication failed`.
  Source: `.reference/OpenDoas/pam.c`

- `auth/pam-text-info-to-stderr`
  PAM `PAM_TEXT_INFO` and `PAM_ERROR_MSG` conversation messages must go to `stderr`
  with trailing newlines.
  Source: `.reference/OpenDoas/pam.c`

- `auth/pam-open-session-failure`
  PAM session-open failure must surface `pam_open_session: ...`.
  Source: `.reference/OpenDoas/pam.c`

- `auth/pam-child-signaled`
  Under PAM, a child terminated by signal must print the signal text and exit with
  `128 + signal`.
  Source: `.reference/OpenDoas/pam.c`

- `auth/plain-piped-stdin-with-controlling-tty`
  Shadow auth behavior should be checked when stdin is piped but a controlling tty is
  still available through `/dev/tty`.
  Source: `.reference/OpenDoas/shadow.c`, `.reference/OpenDoas/libopenbsd/readpassphrase.c`

### Persist and Timestamp Hardening

- `persist/timestamp-dir-wrong-owner`
  Wrong owner on the timestamp directory must disable reuse safely.
  Source: `.reference/OpenDoas/timestamp.c`

- `persist/timestamp-file-wrong-owner`
  Wrong owner on an existing timestamp file must fail hard with `timestamp uid, gid or mode wrong`.
  Source: `.reference/OpenDoas/timestamp.c`

- `persist/timestamp-file-wrong-gid`
  Wrong gid on an existing timestamp file must fail hard with `timestamp uid, gid or mode wrong`.
  Source: `.reference/OpenDoas/timestamp.c`

- `persist/timestamp-file-wrong-mode`
  Wrong mode on an existing timestamp file must fail hard with `timestamp uid, gid or mode wrong`.
  Source: `.reference/OpenDoas/timestamp.c`

- `persist/timestamp-never-set-is-invalid-not-fatal`
  A timestamp file with zeroed times must be treated as invalid but not as an error.
  Source: `.reference/OpenDoas/timestamp.c`

- `persist/timestamp-too-old-reprompts`
  An expired ticket must be ignored without a special warning.
  Source: `.reference/OpenDoas/timestamp.c`

- `persist/timestamp-too-far-future-warning`
  A future ticket must warn `timestamp too far in the future` and reprompt.
  Source: `.reference/OpenDoas/timestamp.c`

## Priority 1

These are still source-backed, but they are lower-priority than the explicit
P0 branches above.

### Logging

- `logging/nolog-does-not-suppress-auth-failure`
  `nolog` must not suppress failed-auth syslog entries.
  Source: `.reference/OpenDoas/pam.c`, `.reference/OpenDoas/shadow.c`

- `logging/command-not-permitted-full-cmdline`
  Deny logging should preserve the observable command-line formatting behavior.
  Source: `.reference/OpenDoas/doas.c`

### Environment and Parser Corner Cases

- `env/keepenv-path-does-not-beat-target-default`
  Caller PATH imported by `keepenv` should not override the target/default path node
  inserted first.
  Source: `.reference/OpenDoas/env.c`

- `check/tab-and-comment-normalization`
  Tabs and trailing comments should behave the same as spaces and comments in the lexer.
  Source: `.reference/OpenDoas/parse.y`

## Priority 2

Only add these after the higher-value product behaviors are covered.

### Alpine-Packaged OpenDoas Addendum

These apply only if the oracle is changed from upstream `OpenDoas` to the Alpine
packaged variant in `.reference/Alpine-doas-3.23-stable/`.

- `config/confdir-ignores-legacy-doas-conf`
  With Alpine's confdir patch enabled, `/etc/doas.d` is authoritative and
  `/etc/doas.conf` is ignored.
  Source: `.reference/Alpine-doas-3.23-stable/configuration-directory.patch`

- `config/confdir-alphasort-order`
  `.conf` snippets in `/etc/doas.d` must be loaded in `alphasort()` order.
  Source: `.reference/Alpine-doas-3.23-stable/configuration-directory.patch`

- `config/confdir-no-matching-files`
  An existing config directory with no matching `*.conf` files must fail.
  Source: `.reference/Alpine-doas-3.23-stable/configuration-directory.patch`

- `config/check-mode-confdir-path`
  `doas -C /etc/doas.d` should parse the directory rather than a file when the
  confdir feature is compiled in.
  Source: `.reference/Alpine-doas-3.23-stable/configuration-directory.patch`

- `env/alpine-safe-path-order`
  Alpine's patched safe PATH order should be verified if the oracle image is switched
  to the packaged build.
  Source: `.reference/Alpine-doas-3.23-stable/change-PATH.patch`
