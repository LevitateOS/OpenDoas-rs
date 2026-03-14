#!/bin/sh

set -eu

CONFORMANCE_ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)
ARTIFACT_ROOT="${CONFORMANCE_ROOT}/artifacts"

impl_image_tag() {
    impl=$1
    auth=$2
    timestamp=$3
    printf 'localhost/opendoas-rs-conformance-%s:%s-%s\n' "$impl" "$auth" "$timestamp"
}

load_case() {
    CASE_DIR=$(CDPATH= cd -- "$1" && pwd)
    CASE_NAME=$(printf '%s\n' "${CASE_DIR#${CONFORMANCE_ROOT}/cases/}" | tr '/' '_')

    AUTH=plain
    TIMESTAMP=off
    ACTOR=alice
    EXPECT_EXIT=0
    TTY=0
    EXPECT_STDOUT_MODE=
    EXPECT_STDERR_MODE=
    EXPECT_TTY_MODE=
    EXPECT_LOG_MODE=

    # shellcheck disable=SC1090
    . "${CASE_DIR}/case.env"

    STDOUT_FILE="${CASE_DIR}/stdout"
    STDERR_FILE="${CASE_DIR}/stderr"
    TTY_FILE="${CASE_DIR}/tty"
    LOG_FILE="${CASE_DIR}/log"
    STDIN_FILE="${CASE_DIR}/stdin.txt"
    SETUP_FILE="${CASE_DIR}/setup.sh"
    INVOKE_FILE="${CASE_DIR}/invoke.sh"
    CONF_FILE="${CASE_DIR}/doas.conf"

    [ -f "${INVOKE_FILE}" ] || {
        echo "missing invoke.sh in ${CASE_DIR}" >&2
        exit 1
    }

    [ -n "${EXPECT_STDOUT_MODE}" ] || {
        if [ -f "${STDOUT_FILE}" ]; then EXPECT_STDOUT_MODE=exact; else EXPECT_STDOUT_MODE=empty; fi
    }
    [ -n "${EXPECT_STDERR_MODE}" ] || {
        if [ -f "${STDERR_FILE}" ]; then EXPECT_STDERR_MODE=exact; else EXPECT_STDERR_MODE=empty; fi
    }
    [ -n "${EXPECT_TTY_MODE}" ] || {
        if [ -f "${TTY_FILE}" ]; then EXPECT_TTY_MODE=exact; else EXPECT_TTY_MODE=ignore; fi
    }
    [ -n "${EXPECT_LOG_MODE}" ] || {
        if [ -f "${LOG_FILE}" ]; then EXPECT_LOG_MODE=exact; else EXPECT_LOG_MODE=ignore; fi
    }
}

normalize_file() {
    file=$1
    tr -d '\r' < "${file}"
}

assert_channel() {
    label=$1
    mode=$2
    expect_path=$3
    actual_path=$4

    case "${mode}" in
        ignore)
            return 0
            ;;
        empty)
            if [ -s "${actual_path}" ]; then
                echo "${label} expected empty, got:" >&2
                sed -n '1,120p' "${actual_path}" >&2
                return 1
            fi
            ;;
        exact)
            if ! diff -u "${expect_path}" "${actual_path}" >/dev/null 2>&1; then
                echo "${label} mismatch" >&2
                diff -u "${expect_path}" "${actual_path}" >&2 || true
                return 1
            fi
            ;;
        contains)
            needle=$(normalize_file "${expect_path}")
            haystack=$(normalize_file "${actual_path}")
            case "${haystack}" in
                *"${needle}"*) ;;
                *)
                    echo "${label} missing expected text:" >&2
                    printf '%s\n' "${needle}" >&2
                    echo "--- actual ---" >&2
                    printf '%s\n' "${haystack}" >&2
                    return 1
                    ;;
            esac
            ;;
        *)
            echo "unsupported ${label} mode: ${mode}" >&2
            return 1
            ;;
    esac
}

prepare_case_container() {
    container=$1

    podman exec "${container}" /bin/sh -eu -c '
        rm -f /etc/doas.conf
        if [ -f /case/doas.conf ]; then
            cp /case/doas.conf /etc/doas.conf
            chown root:root /etc/doas.conf
            chmod 0400 /etc/doas.conf
        fi
        : > /var/log/messages
    '

    if [ -f "${SETUP_FILE}" ]; then
        podman exec "${container}" /bin/sh -eu /case/setup.sh
    fi
}
