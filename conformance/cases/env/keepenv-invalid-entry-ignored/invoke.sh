#!/bin/sh
exec /usr/bin/python3 - <<'PY'
import os
env = {
    'PATH': os.environ.get('PATH', '/usr/bin'),
    'DISPLAY': ':1',
    '': 'ignored',
}
os.execve('/usr/bin/doas', ['doas', '-u', 'root', '/usr/bin/env'], env)
PY
