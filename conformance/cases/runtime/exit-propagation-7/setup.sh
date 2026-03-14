#!/bin/sh
set -eu
cat > /tmp/exit7.sh <<'EOF'
#!/bin/sh
exit 7
EOF
chmod 0755 /tmp/exit7.sh

