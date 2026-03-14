#!/bin/sh
set -eu
cd /tmp/gone
rmdir /tmp/gone
exec doas -u root /usr/bin/id -u
