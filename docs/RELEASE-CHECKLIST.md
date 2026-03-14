# Release Checklist

Use this before publishing a tagged release or calling a build release-ready.

## Preflight

- [ ] working tree is clean
- [ ] intended version is decided
- [ ] release notes are drafted
- [ ] security-relevant changes since the last release are reviewed

## Verification

- [ ] run `scripts/release-preflight.sh`
- [ ] validate one manual install on a real target host
- [ ] validate rollback on that same target host
- [ ] confirm current online CI is green

## Artifacts

- [ ] build from a clean checkout
- [ ] record exact commit id
- [ ] record exact Rust toolchain used
- [ ] record auth modes covered by the release

## Publication

- [ ] update changelog
- [ ] tag the release commit
- [ ] publish release notes
- [ ] keep rollback instructions with the release

## Post-Release

- [ ] perform a short soak period
- [ ] track regressions found in real use
- [ ] backfill conformance cases for any production bug found
