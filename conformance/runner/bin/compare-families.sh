#!/bin/sh
set -eu

SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)

REBUILD=
if [ "${1:-}" = "--rebuild" ]; then
    REBUILD=--rebuild
    shift
fi

if [ $# -lt 1 ]; then
    set -- "${SCRIPT_DIR}/../../cases"/*
fi

status=0
for path in "$@"; do
    if [ -f "${path}/case.env" ]; then
        if ! "${SCRIPT_DIR}/compare-case.sh" ${REBUILD} "${path}"; then
            status=1
        fi
        continue
    fi

    for case_dir in $(find "${path}" -mindepth 1 -maxdepth 1 -type d | sort); do
        if ! "${SCRIPT_DIR}/compare-case.sh" ${REBUILD} "${case_dir}"; then
            status=1
        fi
    done
done

exit "${status}"
