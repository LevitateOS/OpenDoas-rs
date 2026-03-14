#!/bin/sh
set -eu
doas -u root /tmp/linger.sh &
pid=$!
sleep 1
kill -TERM "$pid"
wait "$pid"
