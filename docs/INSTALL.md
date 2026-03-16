# Installation

This document covers direct installation of `OpenDoas-rs` from source.

## Supported Auth Modes

- `auth-pam`
  First-class target backend for Linux OpenDoas parity. Requires PAM
  development headers and a distro-appropriate PAM service file.
- `auth-plain`
  Shadow/password-file based authentication. This is the current primary
  verified host build path in this project.
- `auth-none`
  Test-only or tightly controlled environments. Not suitable for general
  privileged use.

## Build Requirements

Minimum source build requirements:

- Rust toolchain
- C toolchain
- `pkg-config`
- for `auth-pam`:
  `linux-pam` development headers plus a working `bindgen`/libclang toolchain
- for `auth-plain`:
  libc/shadow support available on the target system

On Alpine, the practical `auth-pam` host prerequisites are:

```sh
doas apk add linux-pam linux-pam-dev clang21 clang21-libclang llvm21-dev pkgconf
```

and, when invoking Cargo directly:

```sh
export LLVM_CONFIG_PATH=/usr/bin/llvm-config-21
export LIBCLANG_PATH=/usr/lib/llvm21/lib
```

## Build Commands

Current default source build:

```sh
cargo build --release --locked
```

At the moment this follows the Cargo default feature set, which is
`auth-plain`, not `auth-pam`.

Explicit PAM build:

```sh
AUTH_MODE=pam cargo build --release --locked --no-default-features --features auth-pam
```

Explicit plain-auth build:

```sh
AUTH_MODE=plain cargo build --release --locked --no-default-features --features auth-plain
```

Explicit no-auth build:

```sh
AUTH_MODE=none cargo build --release --locked --no-default-features --features auth-none
```

## Install

Install the built binary as `/usr/bin/doas` with the setuid bit:

```sh
doas install -Dm4755 target/release/OpenDoas-rs /usr/bin/doas
```

Then create `/etc/doas.conf` owned by `root:root` and mode `0400`.

Example:

```conf
permit persist :wheel
permit nopass root as root
```

## PAM Configuration

The current code uses the PAM service name `doas`.

If you build the PAM backend yourself, you must create `/etc/pam.d/doas`
yourself. Start from your
distribution's `doas` or `sudo` PAM file and adapt as needed.

Arch-style starting point:

```pam
#%PAM-1.0
auth            include         system-auth
account         include         system-auth
session         include         system-auth
```

Do not ship a guessed PAM file from another distribution unchanged.

Note:

- this project aims to treat PAM as a first-class backend for Linux OpenDoas
  parity
- that does not mean PAM is the only supported auth path
- today, the plain/shadow backend is still the more consistently verified host
  build path

## First Validation

After installation:

1. Validate the config without executing a command.
2. Validate one explicit permit rule.
3. Validate one deny path.
4. Confirm logging/audit behavior on your target system.

Suggested checks:

```sh
doas -C /etc/doas.conf
doas -C /etc/doas.conf -u root /usr/bin/id
doas -n /usr/bin/true
```

## Distro Notes

- Alpine:
  prefer comparing your PAM config and defaults with the packaged `opendoas`
  reference
- Debian/Ubuntu:
  start from `/etc/pam.d/sudo` or packaged `doas` if available
- Arch:
  start from the `opendoas` package PAM config

## Unsafe Configurations

Do not consider the installation complete if any of these are true:

- `/usr/bin/doas` is not setuid root
- `/etc/doas.conf` is writable by group or world
- `/etc/doas.conf` is not owned by root
- the PAM service file was copied from another distro without review
