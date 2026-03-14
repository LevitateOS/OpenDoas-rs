#!/bin/sh
set -eu
cat > /tmp/caller-shell <<'EOF'
#!/bin/sh
printf 'shell=%s\n' "$0"
printf 'arg1=%s\n' "${1-}"
EOF
chmod 0755 /tmp/caller-shell
cat > /tmp/bob-shell <<'EOF'
#!/bin/sh
printf 'bob-shell=%s\n' "$0"
EOF
chmod 0755 /tmp/bob-shell
python3 - <<'EOF'
from pathlib import Path
passwd = Path('/etc/passwd')
lines = passwd.read_text().splitlines()
updated = []
for line in lines:
    if line.startswith('bob:'):
        parts = line.split(':')
        parts[-1] = '/tmp/bob-shell'
        line = ':'.join(parts)
    updated.append(line)
passwd.write_text('\n'.join(updated) + '\n')
EOF

