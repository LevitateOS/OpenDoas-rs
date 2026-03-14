#!/bin/sh
exec env DISPLAY=:99 doas -u root /usr/bin/env | sort

