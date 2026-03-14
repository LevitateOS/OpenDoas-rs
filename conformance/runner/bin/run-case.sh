#!/bin/sh
set -eu
exec python3 "$(dirname "$0")/conformance.py" run-case "$@"
