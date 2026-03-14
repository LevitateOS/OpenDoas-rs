#!/bin/sh
set -eu

ensure_group() {
    name="$1"
    gid="$2"
    if ! getent group "$name" >/dev/null 2>&1; then
        addgroup -g "$gid" "$name" >/dev/null
    fi
}

ensure_user() {
    name="$1"
    uid="$2"
    gid_name="$3"
    shell_path="$4"
    password="$5"
    if ! getent passwd "$name" >/dev/null 2>&1; then
        adduser -D -u "$uid" -G "$gid_name" -s "$shell_path" "$name" >/dev/null
    fi
    printf '%s:%s\n' "$name" "$password" | chpasswd
}

ensure_group alice 2000
ensure_group bob 2001
ensure_group carol 2002
ensure_group wheel 3000
ensure_group doastest 4000
ensure_group builders 4001

ensure_user alice 2000 alice /bin/sh alicepass
ensure_user bob 2001 bob /bin/sh bobpass
ensure_user carol 2002 carol /bin/bash carolpass

addgroup alice wheel >/dev/null 2>&1 || true
addgroup alice doastest >/dev/null 2>&1 || true
addgroup bob wheel >/dev/null 2>&1 || true
addgroup bob builders >/dev/null 2>&1 || true

printf 'root:rootpass\n' | chpasswd
