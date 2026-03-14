#!/bin/sh
set -eu
doas -u root /usr/bin/id -u
printf '%s\n' '---'
doas -n -u root /usr/bin/id -u

