#!/bin/sh
set -eu
doas -u root /usr/bin/id -u
doas -u root /bin/chown bob /run/doas
printf '%s\n' '---'
doas -u root /usr/bin/id -u
