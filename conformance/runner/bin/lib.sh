#!/bin/sh
set -eu

ROOT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)
CONFORMANCE_DIR="$ROOT_DIR/conformance"
ARTIFACTS_DIR="$CONFORMANCE_DIR/artifacts"

mkdir -p "$ARTIFACTS_DIR"

die() {
    printf '%s\n' "$*" >&2
    exit 1
}

abs_path() {
    python3 -c 'import os,sys; print(os.path.abspath(sys.argv[1]))' "$1"
}

case_slug() {
    family=$(basename "$(dirname "$1")")
    name=$(basename "$1")
    printf '%s__%s' "$family" "$name"
}

variant_of() {
    awk -F= '$1=="VARIANT"{print $2}' "$1/case.env" 2>/dev/null | tail -n1 | tr -d '"' || true
}

image_tag() {
    impl="$1"
    variant="$2"
    case "${impl}:${variant}" in
        opendoas:plain-off) printf 'localhost/rsudoas-conformance-opendoas:latest' ;;
        opendoas:plain-on) printf 'localhost/rsudoas-conformance-opendoas:shadow-on' ;;
        opendoas:pam-off) printf 'localhost/rsudoas-conformance-opendoas:pam-off' ;;
        opendoas:pam-on) printf 'localhost/rsudoas-conformance-opendoas:pam-on' ;;
        opendoas-rs:plain-off) printf 'localhost/rsudoas-conformance-rsudoas:latest' ;;
        opendoas-rs:plain-on) printf 'localhost/rsudoas-conformance-rsudoas:plain-on' ;;
        opendoas-rs:pam-off) printf 'localhost/rsudoas-conformance-rsudoas:pam' ;;
        opendoas-rs:pam-on) printf 'localhost/rsudoas-conformance-rsudoas:pam-on' ;;
        *) printf 'localhost/opendoas-rs-conformance-%s:%s' "$impl" "$variant" ;;
    esac
}

load_case_env() {
    case_dir="$1"
    [ -f "$case_dir/case.env" ] || die "missing case.env in $case_dir"
    unset VARIANT RUN_AS ACTOR AUTH TIMESTAMP TTY CAPTURE_SYSLOG
    unset EXPECT_EXIT EXPECT_STDOUT_MODE EXPECT_STDERR_MODE EXPECT_TTY_MODE EXPECT_LOG_MODE
    unset COMPARE_STDOUT COMPARE_STDERR COMPARE_TTY COMPARE_SYSLOG
    # shellcheck disable=SC1090
    . "$case_dir/case.env"

    if [ -n "${ACTOR:-}" ] && [ -z "${RUN_AS:-}" ]; then
        RUN_AS="$ACTOR"
    fi
    : "${RUN_AS:=alice}"
    : "${TTY:=0}"

    if [ -z "${VARIANT:-}" ]; then
        auth="${AUTH:-plain}"
        timestamp="${TIMESTAMP:-off}"
        VARIANT="${auth}-${timestamp}"
    fi

    if [ -n "${EXPECT_STDOUT_MODE:-}" ]; then COMPARE_STDOUT="$EXPECT_STDOUT_MODE"; fi
    if [ -n "${EXPECT_STDERR_MODE:-}" ]; then COMPARE_STDERR="$EXPECT_STDERR_MODE"; fi
    if [ -n "${EXPECT_TTY_MODE:-}" ]; then COMPARE_TTY="$EXPECT_TTY_MODE"; fi
    if [ -n "${EXPECT_LOG_MODE:-}" ]; then COMPARE_SYSLOG="$EXPECT_LOG_MODE"; fi

    [ -n "${COMPARE_STDOUT:-}" ] || {
        if [ -f "$case_dir/expect.stdout" ] || [ -f "$case_dir/stdout" ]; then COMPARE_STDOUT=exact; else COMPARE_STDOUT=empty; fi
    }
    [ -n "${COMPARE_STDERR:-}" ] || {
        if [ -f "$case_dir/expect.stderr" ] || [ -f "$case_dir/stderr" ]; then COMPARE_STDERR=exact; else COMPARE_STDERR=empty; fi
    }
    [ -n "${COMPARE_TTY:-}" ] || {
        if [ -f "$case_dir/expect.tty" ] || [ -f "$case_dir/tty" ]; then COMPARE_TTY=exact; else COMPARE_TTY=ignore; fi
    }
    [ -n "${COMPARE_SYSLOG:-}" ] || {
        if [ -f "$case_dir/expect.syslog" ] || [ -f "$case_dir/log" ]; then COMPARE_SYSLOG=exact; else COMPARE_SYSLOG=ignore; fi
    }
    if [ -z "${CAPTURE_SYSLOG:-}" ]; then
        if [ "$COMPARE_SYSLOG" = "ignore" ]; then CAPTURE_SYSLOG=0; else CAPTURE_SYSLOG=1; fi
    fi
}

expect_file_for() {
    case_dir="$1"
    stream="$2"

    if [ -f "$case_dir/expect.$stream" ]; then
        printf '%s/expect.%s\n' "$case_dir" "$stream"
        return 0
    fi

    case "$stream" in
        syslog)
            if [ -f "$case_dir/log" ]; then
                printf '%s/log\n' "$case_dir"
                return 0
            fi
            ;;
        *)
            if [ -f "$case_dir/$stream" ]; then
                printf '%s/%s\n' "$case_dir" "$stream"
                return 0
            fi
            ;;
    esac
    return 1
}

compare_mode() {
    mode="$1"
    left="$2"
    right="$3"
    label="$4"

    case "$mode" in
        ignore)
            return 0
            ;;
        exact)
            cmp -s "$left" "$right" || {
                printf 'mismatch in %s\n' "$label" >&2
                diff -u "$left" "$right" >&2 || true
                return 1
            }
            ;;
        contains)
            needle=$(cat "$left")
            grep -F -- "$needle" "$right" >/dev/null || {
                printf 'missing expected content in %s\n' "$label" >&2
                printf 'expected substring:\n%s\n' "$needle" >&2
                printf 'actual:\n' >&2
                cat "$right" >&2
                return 1
            }
            ;;
        empty)
            [ ! -s "$right" ] || {
                printf 'expected empty %s\n' "$label" >&2
                cat "$right" >&2
                return 1
            }
            ;;
        *)
            die "unknown compare mode: $mode"
            ;;
    esac
}

assert_impl_result() {
    impl="$1"
    case_dir="$2"
    result_dir="$3"
    load_case_env "$case_dir"

    if [ -f "$case_dir/expect.exit" ]; then
        expected_exit=$(tr -d '\n' < "$case_dir/expect.exit")
        actual_exit=$(tr -d '\n' < "$result_dir/exit_code")
        [ "$expected_exit" = "$actual_exit" ] || die "$impl exit mismatch for $(case_slug "$case_dir"): expected $expected_exit got $actual_exit"
    elif [ -n "${EXPECT_EXIT:-}" ]; then
        expected_exit="$EXPECT_EXIT"
        actual_exit=$(tr -d '\n' < "$result_dir/exit_code")
        [ "$expected_exit" = "$actual_exit" ] || die "$impl exit mismatch for $(case_slug "$case_dir"): expected $expected_exit got $actual_exit"
    fi

    for stream in stdout stderr tty syslog; do
        actual_file="$result_dir/$stream"
        mode_var="COMPARE_$(printf '%s' "$stream" | tr '[:lower:]' '[:upper:]')"
        eval mode=\${$mode_var}
        if expect_file=$(expect_file_for "$case_dir" "$stream"); then
            compare_mode "$mode" "$expect_file" "$actual_file" "$impl:$stream"
        elif [ "$mode" = "empty" ]; then
            compare_mode "$mode" /dev/null "$actual_file" "$impl:$stream"
        fi
    done
}

compare_impl_results() {
    case_dir="$1"
    left_dir="$2"
    right_dir="$3"
    load_case_env "$case_dir"

    if [ -f "$case_dir/expect.exit" ]; then
        assert_impl_result opendoas "$case_dir" "$left_dir"
        assert_impl_result opendoas-rs "$case_dir" "$right_dir"
    else
        left_exit=$(tr -d '\n' < "$left_dir/exit_code")
        right_exit=$(tr -d '\n' < "$right_dir/exit_code")
        [ "$left_exit" = "$right_exit" ] || die "exit mismatch for $(case_slug "$case_dir"): OpenDoas=$left_exit OpenDoas-rs=$right_exit"
    fi

    for stream in stdout stderr tty syslog; do
        mode_var="COMPARE_$(printf '%s' "$stream" | tr '[:lower:]' '[:upper:]')"
        eval mode=\${$mode_var}
        compare_mode "$mode" "$left_dir/$stream" "$right_dir/$stream" "$(case_slug "$case_dir"):$stream"
    done
}
