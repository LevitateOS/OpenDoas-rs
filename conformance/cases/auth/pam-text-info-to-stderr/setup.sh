#!/bin/sh
set -eu
cat > /etc/pam-echo.txt <<'EOF'
hello from pam
EOF
cat > /etc/pam.d/doas <<'EOF'
#%PAM-1.0
auth       optional     pam_echo.so file=/etc/pam-echo.txt
auth       sufficient   pam_permit.so
account    sufficient   pam_permit.so
session    sufficient   pam_permit.so
EOF
