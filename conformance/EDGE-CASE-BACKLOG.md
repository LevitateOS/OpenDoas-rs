# Additional OpenDoas Edge-Case Backlog

This file tracks additional parity cases that are not represented in the
current `conformance/cases/` tree but are directly suggested by observable
`OpenDoas` behavior in [`.reference/OpenDoas`](../.reference/OpenDoas).

The goal here is not "more tests" in the abstract. Each item below names a
concrete externally visible behavior that should be pinned down with an oracle
run against `OpenDoas`.

## CLI And Identity

- [ ] `-s` with a command is a usage error. Reference:
  [`doas.c:298-302`](../.reference/OpenDoas/doas.c#L298).
- [ ] `-C` with `-s` is a usage error. Reference:
  [`doas.c:298-302`](../.reference/OpenDoas/doas.c#L298).
- [ ] invoking `doas` when the calling uid has no passwd entry fails with
  `no passwd entry for self`. Reference:
  [`doas.c:304-308`](../.reference/OpenDoas/doas.c#L304).
- [ ] `-u <numeric uid>` may parse successfully but still fail later with
  `no passwd entry for target` if that uid has no passwd entry. Reference:
  [`doas.c:280-282`](../.reference/OpenDoas/doas.c#L280),
  [`doas.c:376-380`](../.reference/OpenDoas/doas.c#L376).
- [ ] `-L` with no timestamp support exits success immediately. Reference:
  [`doas.c:274-279`](../.reference/OpenDoas/doas.c#L274).

## Parser And Lexer

- [ ] overly long config lines raise `too long line`. Reference:
  [`parse.y:315-317`](../.reference/OpenDoas/parse.y#L315).
- [ ] backslash-newline continues a token across lines instead of ending it.
  Reference:
  [`parse.y:275-285`](../.reference/OpenDoas/parse.y#L275).
- [ ] empty quoted args are accepted as real empty-string arguments
  (for example `args ""`). Reference:
  [`parse.y:326-335`](../.reference/OpenDoas/parse.y#L326).
- [ ] `deny` cannot take permit options such as `nopass`, `keepenv`, or
  `setenv`; that should fail at parse time. Reference:
  [`parse.y:113-156`](../.reference/OpenDoas/parse.y#L113).
- [ ] empty `setenv {}` parses successfully and has no runtime effect.
  Reference:
  [`parse.y:153-160`](../.reference/OpenDoas/parse.y#L153).
- [ ] quoted or escaped keywords become plain strings, not grammar tokens
  (examples: `"as"`, `a\\s`). Reference:
  [`parse.y:304-312`](../.reference/OpenDoas/parse.y#L304),
  [`parse.y:337-346`](../.reference/OpenDoas/parse.y#L337).
- [ ] comments never continue across a backslash; `#` consumes to newline.
  Reference:
  [`parse.y:250-257`](../.reference/OpenDoas/parse.y#L250).

## Matching

- [ ] a later `permit` must override an earlier matching `deny`
  (`last match wins` in both directions, not only deny-last). Reference:
  [`doas.c:139-152`](../.reference/OpenDoas/doas.c#L139).
- [ ] numeric group rules like `:3000` should match the caller's groups just
  like named group rules. Reference:
  [`doas.c:81-95`](../.reference/OpenDoas/doas.c#L81),
  [`doas.c:103-113`](../.reference/OpenDoas/doas.c#L103).
- [ ] command path matching is exact string equality, so `cmd /bin/echo`
  should not match invocation `echo`. Reference:
  [`doas.c:120-123`](../.reference/OpenDoas/doas.c#L120).
- [ ] empty-string argv elements participate in exact arg matching. Reference:
  [`doas.c:123-133`](../.reference/OpenDoas/doas.c#L123),
  [`parse.y:334-335`](../.reference/OpenDoas/parse.y#L334).

## Config And Runtime

- [ ] unreadable config file should be distinguished from missing config
  by the actual `fopen` error text. Reference:
  [`doas.c:162-165`](../.reference/OpenDoas/doas.c#L162).
- [ ] `-C` on an unreadable custom config should say
  `could not open config file ...`, not `doas is not enabled, ...`.
  Reference:
  [`doas.c:156-165`](../.reference/OpenDoas/doas.c#L156),
  [`doas.c:191-193`](../.reference/OpenDoas/doas.c#L191).
- [ ] target supplementary groups should come from `initgroups`, not only the
  primary gid. An execution test should inspect `id -G` or equivalent.
  Reference:
  [`doas.c:394-397`](../.reference/OpenDoas/doas.c#L394).
- [ ] logging should use cwd `(failed)` when `getcwd()` fails instead of a
  real path. Reference:
  [`doas.c:404-412`](../.reference/OpenDoas/doas.c#L404).
- [ ] an empty command name should still land in `command not found`.
  Reference:
  [`execvpe.c:57-63`](../.reference/OpenDoas/libopenbsd/execvpe.c#L57),
  [`doas.c:425-427`](../.reference/OpenDoas/doas.c#L425).
- [ ] PATH search should preserve `EACCES` if one matching entry exists but is
  not executable, even if later entries are missing. Reference:
  [`execvpe.c:147-155`](../.reference/OpenDoas/libopenbsd/execvpe.c#L147).
- [ ] oversized PATH components emit `execvp: <component>: path too long`
  to stderr and then continue searching. Reference:
  [`execvpe.c:96-111`](../.reference/OpenDoas/libopenbsd/execvpe.c#L96).

## Environment

- [ ] `keepenv` does not override fixed target values for `HOME`, `LOGNAME`,
  `PATH`, `SHELL`, `USER`, and `DOAS_USER`; earlier fixed inserts win.
  Reference:
  [`env.c:106-115`](../.reference/OpenDoas/env.c#L106),
  [`env.c:135-140`](../.reference/OpenDoas/env.c#L135).
- [ ] `TERM` is copied by default just like `DISPLAY`. Reference:
  [`env.c:93-95`](../.reference/OpenDoas/env.c#L93),
  [`env.c:113`](../.reference/OpenDoas/env.c#L113).
- [ ] invalid inherited environment entries without `=` or with an empty name
  are ignored under `keepenv`. Reference:
  [`env.c:124-133`](../.reference/OpenDoas/env.c#L124).
- [ ] overlong inherited environment names are ignored under `keepenv`.
  Reference:
  [`env.c:129-131`](../.reference/OpenDoas/env.c#L129).
- [ ] duplicate names inside one `setenv { ... }` list should be resolved by
  delete-and-reinsert, so the last one wins. Reference:
  [`env.c:191-223`](../.reference/OpenDoas/env.c#L191).
- [ ] `setenv { VAR=$MISSING }` should leave `VAR` unset instead of inserting
  an empty string. Reference:
  [`env.c:204-220`](../.reference/OpenDoas/env.c#L204).
- [ ] plain `setenv { PATH }` should restore `formerpath`, not the sanitized
  safe path. Reference:
  [`env.c:206-215`](../.reference/OpenDoas/env.c#L206).

## Auth

- [ ] shadow auth should fail immediately when passwd entry is neither shadowed
  (`x`) nor locked (`*`). Reference:
  [`shadow.c:67-75`](../.reference/OpenDoas/shadow.c#L67).
- [ ] shadow `tty required for <user>` is a separate path from ordinary auth
  failure and only happens on `ENOTTY`. Reference:
  [`shadow.c:84-91`](../.reference/OpenDoas/shadow.c#L84).
- [ ] PAM account-management failure after successful password entry should
  still log `failed auth for <user>` and print `Authentication failed`.
  Reference:
  [`pam.c:298-307`](../.reference/OpenDoas/pam.c#L298).
- [ ] PAM `PAM_ERROR_MSG` and `PAM_TEXT_INFO` messages go to stderr with a
  trailing newline. Reference:
  [`pam.c:105-109`](../.reference/OpenDoas/pam.c#L105).
- [ ] PAM prompt rewriting only applies to exact `Password:` or `Password: `
  prompts; all other prompts should pass through unchanged. Reference:
  [`pam.c:66-72`](../.reference/OpenDoas/pam.c#L66).
- [ ] PAM parent/session supervision on signal should print
  `Session terminated, killing shell` and later `...killed.` when it has to
  reap the child. Reference:
  [`pam.c:203-223`](../.reference/OpenDoas/pam.c#L203).

## Persist

- [ ] timestamp file wrong uid, wrong gid, and wrong mode should hard-fail with
  `timestamp uid, gid or mode wrong`. Reference:
  [`timestamp.c:222-225`](../.reference/OpenDoas/timestamp.c#L222).
- [ ] a timestamp file created but never set should be invalid without being an
  error, so authentication is retried normally. Reference:
  [`timestamp.c:227-229`](../.reference/OpenDoas/timestamp.c#L227).
- [ ] a timestamp that is too old should quietly be treated as invalid.
  Reference:
  [`timestamp.c:237-240`](../.reference/OpenDoas/timestamp.c#L237).
- [ ] a timestamp that is too far in the future should be treated as invalid
  and emit `timestamp too far in the future`. Reference:
  [`timestamp.c:242-248`](../.reference/OpenDoas/timestamp.c#L242).
- [ ] timestamp directory wrong owner should fail the persist path safety
  checks. Reference:
  [`timestamp.c:265-272`](../.reference/OpenDoas/timestamp.c#L265).
- [ ] timestamp reuse should fail across a different tty or different session
  leader start time, because the ticket path encodes parent pid, sid, tty, and
  session-leader start time. Reference:
  [`timestamp.c:181-196`](../.reference/OpenDoas/timestamp.c#L181).
- [ ] `-L` clear should succeed on `ENOENT` but fail when `timestamp_path()`
  itself fails. Reference:
  [`timestamp.c:306-314`](../.reference/OpenDoas/timestamp.c#L306).

## Logging

- [ ] successful-command logging should record cwd as `(failed)` when cwd
  lookup fails. Reference:
  [`doas.c:404-412`](../.reference/OpenDoas/doas.c#L404).
- [ ] auth logs should distinguish `failed auth for <user>` from
  `tty required for <user>`. Reference:
  [`shadow.c:85-99`](../.reference/OpenDoas/shadow.c#L85),
  [`pam.c:290-306`](../.reference/OpenDoas/pam.c#L290).
- [ ] PAM account-management failure should emit the same auth-failure log form
  as password failure. Reference:
  [`pam.c:298-307`](../.reference/OpenDoas/pam.c#L298).
- [ ] long command lines in deny and permit logs should be truncated by the
  internal fixed `LINE_MAX` command buffer rather than failing execution.
  Reference:
  [`doas.c:248`](../.reference/OpenDoas/doas.c#L248),
  [`doas.c:335-341`](../.reference/OpenDoas/doas.c#L335).
