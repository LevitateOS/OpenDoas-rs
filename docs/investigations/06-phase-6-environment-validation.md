# Phase 6: Environment Validation

## Objective

Audit the current repository evidence for environment and deployment readiness, with emphasis on what is and is not proven about real installs, distro coverage, host validation, and operator readiness. The goal is to decide whether Phase 6 can exit based on the current docs, CI workflow, and conformance harness.

## Scope

Reviewed [PRODUCTION-READINESS.md](/home/vince/Projects/rsudoas/PRODUCTION-READINESS.md#L1), [docs/INSTALL.md](/home/vince/Projects/rsudoas/docs/INSTALL.md#L1), [docs/OPERATIONS.md](/home/vince/Projects/rsudoas/docs/OPERATIONS.md#L1), [docs/RELEASE-CHECKLIST.md](/home/vince/Projects/rsudoas/docs/RELEASE-CHECKLIST.md#L1), [ci.yml](/home/vince/Projects/rsudoas/.github/workflows/ci.yml#L1), and the active conformance image/runner implementation in [conformance/images/opendoas/Containerfile](/home/vince/Projects/rsudoas/conformance/images/opendoas/Containerfile#L1), [conformance/images/opendoas-rs/Containerfile](/home/vince/Projects/rsudoas/conformance/images/opendoas-rs/Containerfile#L1), and [conformance/runner/bin/conformance.py](/home/vince/Projects/rsudoas/conformance/runner/bin/conformance.py#L1).

This review is limited to environment-validation evidence. It does not attempt a privilege-boundary security review or a new conformance audit.

## Commands Run

```sh
git status --short
find . -path '*/.github/workflows/*.yml' -o -path '*/.github/workflows/*.yaml' -o -name 'ci.yml' -o -name 'ci.yaml'
nl -ba PRODUCTION-READINESS.md | sed -n '1,220p'
nl -ba docs/INSTALL.md | sed -n '1,220p'
nl -ba docs/OPERATIONS.md | sed -n '1,220p'
nl -ba docs/RELEASE-CHECKLIST.md | sed -n '1,220p'
nl -ba .github/workflows/ci.yml | sed -n '1,220p'
find conformance -maxdepth 4 \( -name 'Containerfile' -o -name 'Dockerfile' -o -name '*.Dockerfile' \) | sort
nl -ba conformance/images/opendoas/Containerfile | sed -n '1,240p'
nl -ba conformance/images/opendoas-rs/Containerfile | sed -n '1,260p'
nl -ba conformance/runner/bin/conformance.py | sed -n '1,520p'
python3 - <<'PY'
from pathlib import Path
import tomllib
from collections import Counter
root=Path('conformance/cases')
oracle=Counter(); subject=Counter(); both=Counter()
for p in root.rglob('case.toml'):
    data=tomllib.loads(p.read_text())['case']
    oracle[data.get('oracle_variant','shadow-off')] += 1
    subject[data.get('subject_variant','plain-off')] += 1
    both[(data.get('oracle_variant','shadow-off'), data.get('subject_variant','plain-off'))] += 1
print('oracle')
for k,v in sorted(oracle.items()):
    print(f'{k} {v}')
print('subject')
for k,v in sorted(subject.items()):
    print(f'{k} {v}')
print('pairs')
for k,v in sorted(both.items()):
    print(f'{k[0]} -> {k[1]} {v}')
PY
```

Key command result: the case census found `113` total `case.toml` files, split as `91` `shadow-off -> plain-off`, `14` `shadow-on -> plain-on`, and `8` `pam-off -> pam-off`. No `pam-on` cases were present.

## Findings

1. **High: no real-host install or deployment path is evidenced yet.**

   The readiness document still leaves every environment-matrix gate open, including validation on two real Linux distributions, validation of setuid behavior outside the runner, and validation of a real package or install path. The installation guide only documents a direct source build plus `install -Dm4755`, followed by suggested manual checks; it does not record any completed install on a target host. The release checklist also leaves real-host install and rollback validation unchecked. What is currently proven is narrower: CI runs builds on `ubuntu-latest`, then the conformance runner exports container images into a temporary rootfs and executes them via `unshare` and `chroot`. That is useful evidence for container-local runtime parity, but it is not evidence of a native distro install, packaged deployment, or host-integrated setuid path.

   Refs: [PRODUCTION-READINESS.md](/home/vince/Projects/rsudoas/PRODUCTION-READINESS.md#L61), [PRODUCTION-READINESS.md](/home/vince/Projects/rsudoas/PRODUCTION-READINESS.md#L67), [PRODUCTION-READINESS.md](/home/vince/Projects/rsudoas/PRODUCTION-READINESS.md#L69), [docs/INSTALL.md](/home/vince/Projects/rsudoas/docs/INSTALL.md#L47), [docs/INSTALL.md](/home/vince/Projects/rsudoas/docs/INSTALL.md#L82), [docs/RELEASE-CHECKLIST.md](/home/vince/Projects/rsudoas/docs/RELEASE-CHECKLIST.md#L15), [docs/RELEASE-CHECKLIST.md](/home/vince/Projects/rsudoas/docs/RELEASE-CHECKLIST.md#L16), [ci.yml](/home/vince/Projects/rsudoas/.github/workflows/ci.yml#L40), [conformance/runner/bin/conformance.py](/home/vince/Projects/rsudoas/conformance/runner/bin/conformance.py#L128), [conformance/runner/bin/conformance.py](/home/vince/Projects/rsudoas/conformance/runner/bin/conformance.py#L157), [conformance/runner/bin/conformance.py](/home/vince/Projects/rsudoas/conformance/runner/bin/conformance.py#L178), [conformance/runner/bin/conformance.py](/home/vince/Projects/rsudoas/conformance/runner/bin/conformance.py#L391)

2. **High: the current environment matrix is still effectively Alpine-centric and incomplete.**

   The target gate calls for at least two real Linux distributions plus both supported auth backends where relevant. The actual automated setup remains narrower: the CI jobs run on a single `ubuntu-latest` host, and both conformance implementations are built from `alpine:3.23` images. The case census shows that most cases are still the default `shadow-off -> plain-off` path, only `14` cases exercise timestamp-enabled variants, only `8` cases exercise PAM, and no case requests `pam-on`. On top of that, the runner can silently downgrade a requested `*-on` variant to `*-off` if the build fails, which weakens timestamp-on proof unless the resulting behavior differs enough to trip a parity mismatch. The repository therefore has meaningful Alpine-based parity evidence, but it does not yet prove a cross-distro deployment matrix.

   Refs: [PRODUCTION-READINESS.md](/home/vince/Projects/rsudoas/PRODUCTION-READINESS.md#L63), [PRODUCTION-READINESS.md](/home/vince/Projects/rsudoas/PRODUCTION-READINESS.md#L65), [PRODUCTION-READINESS.md](/home/vince/Projects/rsudoas/PRODUCTION-READINESS.md#L71), [ci.yml](/home/vince/Projects/rsudoas/.github/workflows/ci.yml#L42), [conformance/images/opendoas/Containerfile](/home/vince/Projects/rsudoas/conformance/images/opendoas/Containerfile#L1), [conformance/images/opendoas-rs/Containerfile](/home/vince/Projects/rsudoas/conformance/images/opendoas-rs/Containerfile#L1), [conformance/runner/bin/conformance.py](/home/vince/Projects/rsudoas/conformance/runner/bin/conformance.py#L92), [conformance/runner/bin/conformance.py](/home/vince/Projects/rsudoas/conformance/runner/bin/conformance.py#L117), [conformance/runner/bin/conformance.py](/home/vince/Projects/rsudoas/conformance/runner/bin/conformance.py#L499), [docs/INSTALL.md](/home/vince/Projects/rsudoas/docs/INSTALL.md#L99)

3. **Medium: operator readiness is documented, but not operator-validated, and the readiness checklist overstates that proof.**

   The readiness document explicitly says the release and operational guidance still needs real operator validation, but the Operational Gate is marked complete. The install guide asks operators to create their own PAM file, adapt from distro references, and avoid copying a guessed PAM file unchanged. The operations guide likewise tells operators to validate their actual logging sink, preserve rollback artifacts, and perform upgrade and rollback checks manually. Those instructions are useful, but they are not evidence that an operator has successfully followed them on Alpine, Debian/Ubuntu, Arch, or any other target environment. The release checklist confirms the gap by leaving manual host validation and soak work unchecked.

   Refs: [PRODUCTION-READINESS.md](/home/vince/Projects/rsudoas/PRODUCTION-READINESS.md#L43), [PRODUCTION-READINESS.md](/home/vince/Projects/rsudoas/PRODUCTION-READINESS.md#L101), [docs/INSTALL.md](/home/vince/Projects/rsudoas/docs/INSTALL.md#L64), [docs/INSTALL.md](/home/vince/Projects/rsudoas/docs/INSTALL.md#L80), [docs/INSTALL.md](/home/vince/Projects/rsudoas/docs/INSTALL.md#L99), [docs/OPERATIONS.md](/home/vince/Projects/rsudoas/docs/OPERATIONS.md#L16), [docs/OPERATIONS.md](/home/vince/Projects/rsudoas/docs/OPERATIONS.md#L22), [docs/OPERATIONS.md](/home/vince/Projects/rsudoas/docs/OPERATIONS.md#L38), [docs/OPERATIONS.md](/home/vince/Projects/rsudoas/docs/OPERATIONS.md#L79), [docs/RELEASE-CHECKLIST.md](/home/vince/Projects/rsudoas/docs/RELEASE-CHECKLIST.md#L15), [docs/RELEASE-CHECKLIST.md](/home/vince/Projects/rsudoas/docs/RELEASE-CHECKLIST.md#L35)

4. **Medium: release preflight does not add deployment evidence beyond current CI and conformance automation.**

   The release checklist requires `scripts/release-preflight.sh`, but the script only rebuilds three feature combinations and reruns the conformance suite plus parser stress. It does not install onto a real host, verify rollback, exercise journald or syslog integration on a target system, or validate a package-managed install path. That means passing preflight is necessary but not sufficient for closing the environment-validation gate.

   Refs: [docs/RELEASE-CHECKLIST.md](/home/vince/Projects/rsudoas/docs/RELEASE-CHECKLIST.md#L14), [scripts/release-preflight.sh](/home/vince/Projects/rsudoas/scripts/release-preflight.sh#L1), [scripts/release-preflight.sh](/home/vince/Projects/rsudoas/scripts/release-preflight.sh#L4), [scripts/release-preflight.sh](/home/vince/Projects/rsudoas/scripts/release-preflight.sh#L7)

## Remaining Risks

- The first native install on a real distro may fail on PAM integration, filesystem ownership or mode assumptions, or distro packaging expectations that are invisible inside the Alpine-based harness.
- Native setuid behavior remains unproven under real host constraints such as filesystem mounts, service managers, LSM policy, and distro-specific account tooling.
- Logging and audit behavior is only partially proven. The docs tell operators to validate the actual sink, but no recorded evidence shows journald, syslog forwarding, or rotation working as intended on a supported host.
- PAM coverage is thin relative to the broader suite, and there is no recorded `pam-on` matrix coverage.
- Upgrade, rollback, and short soak-period behavior remain documented process requirements rather than demonstrated deployment evidence.

## Exit Decision

`Do not exit Phase 6 yet.`

Current evidence is strong enough to claim containerized Alpine-based parity coverage, basic setuid install behavior inside the harness, and the presence of operator-facing documentation. It is not strong enough to claim environment validation is complete, because the repository still lacks recorded real-host installs, a multi-distro validation matrix, host-native setuid and logging validation, and operator-validated upgrade or rollback evidence.
