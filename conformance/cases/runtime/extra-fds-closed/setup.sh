#!/bin/sh
set -eu
cat > /tmp/check-fd.sh <<'EOF'
#!/bin/sh
if [ -e /proc/self/fd/9 ]; then
    printf 'fd9=open\n'
    exit 1
fi
printf 'fd9=closed\n'
EOF
chmod 0755 /tmp/check-fd.sh

