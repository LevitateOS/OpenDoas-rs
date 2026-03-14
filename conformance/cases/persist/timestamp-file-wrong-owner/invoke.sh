#!/bin/sh
set -eu
doas -u root /usr/bin/id -u
ts=$(find /run/doas -maxdepth 1 -type f | head -n 1)
doas -u root /bin/chown bob "$ts"
printf '%s\n' '---'
doas -u root /usr/bin/id -u
