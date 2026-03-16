#!/bin/sh
set -eu

/usr/bin/python3 - <<'PY'
import os

os.execve(
    b"/usr/bin/doas",
    [b"doas", b"/usr/bin/id", b"-u"],
    {
        b"PATH": b"/usr/bin",
        b"BAD": b"\xff",
    },
)
PY
