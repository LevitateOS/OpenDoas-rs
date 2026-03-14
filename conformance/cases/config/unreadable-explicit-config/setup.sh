#!/bin/sh
set -eu
cat > /tmp/unreadable.conf <<'EOF'
permit nopass alice as root cmd /usr/bin/id args -u
EOF
chmod 000 /tmp/unreadable.conf
