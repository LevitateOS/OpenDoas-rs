#!/bin/sh
set -eu

[ $# -eq 3 ] || {
    printf 'usage: container-run.sh <impl> <case-dir> <results-dir>\n' >&2
    exit 1
}

impl="$1"
case_dir="$2"
results_dir="$3"

# shellcheck disable=SC1090
. "$case_dir/case.env"

: "${ACTOR:=}"
[ -n "$ACTOR" ] && : "${RUN_AS:=$ACTOR}"
: "${RUN_AS:=alice}"
: "${TTY:=0}"
: "${CAPTURE_SYSLOG:=0}"

mkdir -p "$results_dir"
: > "$results_dir/stdout"
: > "$results_dir/stderr"
: > "$results_dir/tty"
: > "$results_dir/syslog"

/bin/sh /conformance/fixtures/users/basic-users.sh

if [ -f "$case_dir/doas.conf" ]; then
    cp "$case_dir/doas.conf" /etc/doas.conf
    chown root:root /etc/doas.conf
    chmod "${DOAS_CONF_MODE:-0400}" /etc/doas.conf
fi

if [ -f "$case_dir/pam.doas" ]; then
    mkdir -p /etc/pam.d
    cp "$case_dir/pam.doas" /etc/pam.d/doas
elif [ -n "${PAM_PROFILE:-}" ]; then
    mkdir -p /etc/pam.d
    cp "/conformance/fixtures/pam/doas-$PAM_PROFILE" /etc/pam.d/doas
fi

if [ -f "$case_dir/setup.sh" ]; then
    CASE_IMPL="$impl" CASE_DIR="$case_dir" RESULTS_DIR="$results_dir" /bin/sh "$case_dir/setup.sh"
fi

syslog_pid=
if [ "$CAPTURE_SYSLOG" = "1" ]; then
    rm -f /dev/log
    python3 /conformance/fixtures/files/syslog_capture.py /dev/log "$results_dir/syslog" &
    syslog_pid=$!
    sleep 0.2
fi

wrapper=/tmp/conformance-case-wrapper.sh
cat > "$wrapper" <<'EOF'
#!/bin/sh
set -eu
exec /bin/sh "$CASE_DIR/invoke.sh"
EOF
chmod 755 "$wrapper"

stdin_file=/dev/null
if [ -f "$case_dir/stdin.txt" ]; then
    stdin_file="$case_dir/stdin.txt"
fi

set +e
if [ "$RUN_AS" = "root" ]; then
    if [ "$TTY" = "1" ]; then
        CASE_IMPL="$impl" CASE_DIR="$case_dir" RESULTS_DIR="$results_dir" \
            python3 /conformance/runner/bin/pty_capture.py "$results_dir/tty" "$stdin_file" "$wrapper"
        exit_code=$?
    else
        CASE_IMPL="$impl" CASE_DIR="$case_dir" RESULTS_DIR="$results_dir" \
            /bin/sh "$wrapper" < "$stdin_file" > "$results_dir/stdout" 2> "$results_dir/stderr"
        exit_code=$?
    fi
else
    if [ "$TTY" = "1" ]; then
        CASE_IMPL="$impl" CASE_DIR="$case_dir" RESULTS_DIR="$results_dir" \
            python3 /conformance/runner/bin/pty_capture.py "$results_dir/tty" "$stdin_file" \
            su -s /bin/sh "$RUN_AS" -c "CASE_IMPL='$impl' CASE_DIR='$case_dir' RESULTS_DIR='$results_dir' $wrapper"
        exit_code=$?
    else
        su -s /bin/sh "$RUN_AS" -c "CASE_IMPL='$impl' CASE_DIR='$case_dir' RESULTS_DIR='$results_dir' $wrapper" \
            < "$stdin_file" > "$results_dir/stdout" 2> "$results_dir/stderr"
        exit_code=$?
    fi
fi
set -e

printf '%s\n' "$exit_code" > "$results_dir/exit_code"

if [ -n "$syslog_pid" ]; then
    kill "$syslog_pid" >/dev/null 2>&1 || true
    wait "$syslog_pid" >/dev/null 2>&1 || true
fi

exit 0
