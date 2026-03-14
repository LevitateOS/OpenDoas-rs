#!/bin/sh
set -eu
arg=$(/usr/bin/python3 - <<'PY'
print("x" * 5000)
PY
)
exec doas -u root /bin/echo "$arg"
