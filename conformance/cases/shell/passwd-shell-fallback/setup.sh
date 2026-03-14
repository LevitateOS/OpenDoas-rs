#!/bin/sh
set -eu
cat > /tmp/show-shell <<'EOF'
#!/bin/sh
printf 'shell=%s\n' "$0"
printf 'arg1=%s\n' "${1-}"
EOF
chmod 0755 /tmp/show-shell
python3 - <<'EOF'
from pathlib import Path
passwd = Path('/etc/passwd')
lines = passwd.read_text().splitlines()
updated = []
for line in lines:
    if line.startswith('carol:'):
        parts = line.split(':')
        parts[-1] = '/tmp/show-shell'
        line = ':'.join(parts)
    updated.append(line)
passwd.write_text('\n'.join(updated) + '\n')
EOF

