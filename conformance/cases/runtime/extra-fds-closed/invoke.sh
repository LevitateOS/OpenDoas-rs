#!/bin/sh
exec 9>/tmp/parent-fd
exec doas -u root /tmp/check-fd.sh

