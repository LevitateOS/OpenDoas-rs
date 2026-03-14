#!/bin/sh
set -eu
python3 - <<'PY'
from pathlib import Path

line = "permit nopass alice as root cmd /usr/bin/" + ("x" * 1100) + "\n"
Path("/etc/doas.conf").write_text(line)
PY
chmod 0400 /etc/doas.conf
