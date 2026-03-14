#!/bin/sh
set -eu
cat > /tmp/show-shell <<'EOF'
#!/bin/sh
printf 'shell=%s\n' "$0"
printf 'arg1=%s\n' "${1-}"
EOF
chmod 0755 /tmp/show-shell

