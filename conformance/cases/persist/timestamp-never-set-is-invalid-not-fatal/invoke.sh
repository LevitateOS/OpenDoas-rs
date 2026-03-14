#!/bin/sh
set -eu
doas -u root /usr/bin/id -u
ts=$(find /run/doas -maxdepth 1 -type f | head -n 1)
doas -u root /usr/bin/python3 - "$ts" <<'PY'
import os, sys
os.utime(sys.argv[1], ns=(0, 0))
PY
printf '%s\n' '---'
doas -u root /usr/bin/id -u
