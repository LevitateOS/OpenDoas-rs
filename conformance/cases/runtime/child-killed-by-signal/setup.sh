#!/bin/sh
set -eu
cat > /tmp/sigterm.sh <<'EOF'
#!/bin/sh
kill -TERM $$
EOF
chmod 0755 /tmp/sigterm.sh

