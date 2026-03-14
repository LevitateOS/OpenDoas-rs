#!/bin/sh
set -eu
python3 - <<'PY'
from pathlib import Path

shadow = Path("/etc/shadow")
lines = [line for line in shadow.read_text().splitlines() if not line.startswith("alice:")]
shadow.write_text("\n".join(lines) + "\n")
PY
