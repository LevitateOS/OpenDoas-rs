#!/bin/sh
set -eu
cp /conformance/fixtures/pam/doas-permit /etc/pam.d/doas
cat > /tmp/sigterm.sh <<'EOF'
#!/bin/sh
kill -TERM $$
EOF
chmod 0755 /tmp/sigterm.sh
