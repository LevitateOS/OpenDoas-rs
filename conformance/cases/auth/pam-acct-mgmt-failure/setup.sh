#!/bin/sh
set -eu
cat > /etc/pam.d/doas <<'EOF'
#%PAM-1.0
auth       sufficient   pam_permit.so
account    required     pam_deny.so
session    sufficient   pam_permit.so
EOF
