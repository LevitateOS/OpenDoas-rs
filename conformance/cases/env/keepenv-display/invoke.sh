#!/bin/sh
exec env DISPLAY=:99 FOO=bar doas -u root /usr/bin/env | sort

