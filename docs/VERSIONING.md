# Versioning And Changelog Policy

`OpenDoas-rs` uses a simple release policy:

- patch release:
  bug fixes, parity fixes, documentation fixes, no intended breaking config or
  interface change
- minor release:
  backwards-compatible feature additions or significant coverage expansion
- major release:
  intentional breaking change in supported behavior, install contract, or
  operator-facing interface

## Changelog Policy

Every release should summarize:

- parity fixes
- security-relevant fixes
- auth/backend changes
- install/operational changes
- known limitations remaining after the release

Use a single top-level `CHANGELOG.md` with an `Unreleased` section.
