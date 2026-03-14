#!/bin/sh
set -eu

addgroup -g 3000 wheel 2>/dev/null || true

adduser -D -u 2000 -s /bin/sh alice 2>/dev/null || true
adduser -D -u 2001 -s /bin/sh bob 2>/dev/null || true
adduser -D -u 2002 -s /bin/ash carol 2>/dev/null || true

addgroup alice wheel 2>/dev/null || true

printf '%s\n' \
  'root:rootpass' \
  'alice:secret' \
  'bob:secret' \
  'carol:secret' | chpasswd
