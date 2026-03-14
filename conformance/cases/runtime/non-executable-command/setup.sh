#!/bin/sh
set -eu
printf '#!/bin/sh\nexit 0\n' > /tmp/not-exec
chmod 0644 /tmp/not-exec

