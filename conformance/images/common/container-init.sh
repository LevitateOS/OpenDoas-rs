#!/bin/sh
set -eu

mkdir -p /run /var/log /etc/pam.d
: > /var/log/messages

syslogd -O /var/log/messages

exec sleep infinity
