#!/bin/sh
set -eu

SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)

if [ "${1:-}" = "--rebuild-image" ]; then
    shift
    exec python3 "$SCRIPT_DIR/conformance.py" run-case --rebuild "$@"
fi

exec python3 "$SCRIPT_DIR/conformance.py" run-case "$@"
