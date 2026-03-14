# Operations

This document covers day-2 operation of `OpenDoas-rs`.

## Logging Expectations

Operators should expect security-relevant events for:

- denied command attempts
- failed authentication attempts
- permitted command execution, where configured

`nolog` should only suppress permit logging where the implementation and policy
 allow it. It should not hide deny or failed-auth events.

Validate your actual sink:

- syslog
- journald forwarding
- log rotation path

## Upgrade

Before upgrade:

1. Keep a copy of the current binary.
2. Keep a copy of `/etc/doas.conf`.
3. Keep a copy of your PAM service file if using PAM.
4. Run the release preflight on the candidate build if you built it yourself.

Upgrade steps:

1. install the new binary with mode `4755`
2. verify ownership and mode on `/usr/bin/doas`
3. run `doas -C /etc/doas.conf`
4. validate one known-good command and one known-denied command

## Rollback

Rollback should be immediate:

1. replace `/usr/bin/doas` with the previously known-good binary
2. restore the previous `/etc/doas.conf` if it changed
3. restore the previous PAM service file if it changed
4. rerun the same validation checks used during upgrade

## Failure Modes

Common operator-visible failures:

- `Authentication required`
  usually policy requires auth and the invocation was non-interactive
- TTY-required failures
  expected for interactive auth paths without a controlling terminal
- `command not found`
  command lookup failed under the target execution environment
- config ownership/mode rejection
  `/etc/doas.conf` is unsafe at runtime
- setuid-required failure
  the installed binary lost root ownership or the setuid bit

## Safe Default Config Example

Conservative starting point:

```conf
permit persist :wheel
permit nopass root as root
```

More explicit command scoping:

```conf
permit :wheel as root cmd /usr/bin/systemctl
permit :wheel as root cmd /usr/bin/journalctl
deny :wheel as root cmd /usr/bin/sh
```

## Operator Checklist

Before putting a host in service:

1. confirm binary path, owner, and setuid mode
2. confirm `/etc/doas.conf` owner and mode
3. confirm PAM service presence and correctness if enabled
4. confirm deny logging and failed-auth logging
5. confirm rollback binary and config are retained
