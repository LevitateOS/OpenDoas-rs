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
    gid="$3"
    shell="$4"
    if ! getent passwd "$name" >/dev/null 2>&1; then
        adduser -D -u "$uid" -G "$(getent group "$gid" | cut -d: -f1)" -s "$shell" "$name" >/dev/null
    fi
}

set_password() {
    name="$1"
    password="$2"
    printf '%s:%s\n' "$name" "$password" | chpasswd
}

ensure_group alice 2000
ensure_group bob 2001
ensure_group carol 2002
ensure_group doastest 4000
ensure_group builders 4001

ensure_user alice 2000 2000 /bin/sh
ensure_user bob 2001 2001 /bin/sh
ensure_user carol 2002 2002 /bin/bash

addgroup alice doastest >/dev/null 2>&1 || true
addgroup bob builders >/dev/null 2>&1 || true

set_password alice alicepass
set_password bob bobpass
set_password carol carolpass

