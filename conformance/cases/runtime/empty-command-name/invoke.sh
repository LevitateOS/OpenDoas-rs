#!/bin/sh
exec /usr/bin/python3 - <<'PY'
import os
os.execv('/usr/bin/doas', ['doas', '-u', 'root', ''])
PY
