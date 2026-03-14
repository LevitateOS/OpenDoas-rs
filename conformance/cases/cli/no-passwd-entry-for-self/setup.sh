#!/bin/sh
set -eu
python3 - <<'PY'
from pathlib import Path

for path in (Path("/etc/passwd"), Path("/etc/shadow")):
    lines = [line for line in path.read_text().splitlines() if not line.startswith("alice:")]
    path.write_text("\n".join(lines) + "\n")
PY
