#!/bin/sh
set -eu
cat > /tmp/no-shebang <<'EOF'
printf 'fallback-ok\n'
EOF
chmod 0755 /tmp/no-shebang

