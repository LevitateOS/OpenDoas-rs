#!/bin/sh
set -eu
python3 - <<'EOF'
from pathlib import Path
Path('/etc/doas.conf').write_bytes(b'permit nopass alice as root cmd /usr/bin/id\\xff\\n')
EOF
chmod 0400 /etc/doas.conf
chown root:root /etc/doas.conf

