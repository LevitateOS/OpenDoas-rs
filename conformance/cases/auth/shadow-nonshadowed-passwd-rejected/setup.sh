#!/bin/sh
set -eu
python3 - <<'PY'
from pathlib import Path

passwd = Path("/etc/passwd")
lines = []
for line in passwd.read_text().splitlines():
    if line.startswith("alice:"):
        parts = line.split(":")
        parts[1] = "!"
        line = ":".join(parts)
    lines.append(line)
passwd.write_text("\n".join(lines) + "\n")
PY
