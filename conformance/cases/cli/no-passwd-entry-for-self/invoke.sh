#!/bin/sh
exec /sbin/su-exec 2000:2000 doas -u root /usr/bin/id -u
