#!/bin/sh
set -eu
cp /conformance/fixtures/pam/doas-permit /etc/pam.d/doas
cat > /tmp/linger.sh <<'EOF'
#!/bin/sh
trap '' TERM
while :; do
    sleep 1
done
EOF
chmod 0755 /tmp/linger.sh
