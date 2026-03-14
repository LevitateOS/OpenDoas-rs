#!/bin/sh
set -eu

SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)

rebuild=
if [ "${1:-}" = "--rebuild-image" ]; then
    rebuild="--rebuild-image"
    shift
fi

[ $# -eq 1 ] || {
    printf 'usage: run-all.sh [--rebuild-image] <opendoas|opendoas-rs>\n' >&2
    exit 1
}

impl="$1"

find "$SCRIPT_DIR/../../cases" -mindepth 2 -maxdepth 2 -type d | sort | while IFS= read -r case_dir; do
    printf '==> %s %s\n' "$impl" "$(basename "$case_dir")"
    "$SCRIPT_DIR/run-case.sh" ${rebuild:+$rebuild} "$impl" "$case_dir"
done

