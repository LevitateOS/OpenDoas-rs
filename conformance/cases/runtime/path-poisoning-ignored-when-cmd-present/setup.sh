#!/bin/sh
set -eu
mkdir -p /tmp/bin
cat > /tmp/bin/id <<'EOF'
#!/bin/sh
printf '%s\n' poison
EOF
chmod 0755 /tmp/bin/id
