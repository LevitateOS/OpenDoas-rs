#!/bin/sh
set -eu

cargo build --locked
AUTH_MODE=plain cargo build --locked --no-default-features --features auth-plain
AUTH_MODE=none cargo build --locked --no-default-features --features auth-none
conformance/runner/bin/run-suite.sh
conformance/runner/bin/run-parser-stress.sh --count 20
