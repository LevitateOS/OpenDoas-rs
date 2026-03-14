#!/bin/sh
set -eu
mkdir -p /tmp/bin
cat > /tmp/bin/hello <<'EOF'
#!/bin/sh
printf '%s\n' restored-path
EOF
chmod 0755 /tmp/bin/hello
