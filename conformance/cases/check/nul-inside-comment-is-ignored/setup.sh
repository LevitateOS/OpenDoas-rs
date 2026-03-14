#!/bin/sh
set -eu
python3 - <<'PY'
from pathlib import Path

Path("/etc/doas.conf").write_bytes(
    b"# comment before NUL\x00this should stay in the comment\n"
    b"permit nopass alice as root cmd /usr/bin/id args -u\n"
)
PY
chmod 0400 /etc/doas.conf
