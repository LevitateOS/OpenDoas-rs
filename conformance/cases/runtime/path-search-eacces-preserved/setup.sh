#!/bin/sh
set -eu
mkdir -p /tmp/bin
cat > /tmp/bin/hello <<'EOF'
#!/bin/sh
printf '%s\n' should-not-run
EOF
chmod 0644 /tmp/bin/hello
