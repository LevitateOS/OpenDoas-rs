#!/bin/sh
set -eu
long=$(/usr/bin/python3 - <<'PY'
print("/" + ("x" * 5000))
PY
)
PATH="$long:/usr/bin" exec doas -u root id -u
