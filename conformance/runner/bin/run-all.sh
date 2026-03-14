#!/bin/sh
set -eu

SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)

rebuild=
if [ "${1:-}" = "--rebuild-image" ]; then
    rebuild="--rebuild"
    shift
fi

exec python3 "$SCRIPT_DIR/conformance.py" run-suite ${rebuild:+$rebuild} "$@"
