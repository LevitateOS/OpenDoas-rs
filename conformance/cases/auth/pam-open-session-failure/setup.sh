#!/bin/sh
set -eu
cat > /etc/pam.d/doas <<'EOF'
#%PAM-1.0
auth       sufficient   pam_permit.so
account    sufficient   pam_permit.so
session    required     pam_deny.so
EOF
