#!/bin/sh
set -eu

SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
. "$SCRIPT_DIR/lib.sh"

[ $# -eq 2 ] || die "usage: build-image.sh <opendoas|opendoas-rs> <variant>"

impl="$1"
variant="$2"
tag=$(image_tag "$impl" "$variant")
context_dir=$(mktemp -d)
trap 'rm -rf "$context_dir"' EXIT INT TERM

case "$impl" in
    opendoas)
        cp "$CONFORMANCE_DIR/images/opendoas/Containerfile" "$context_dir/Containerfile"
        cp -R "$ROOT_DIR/.reference/OpenDoas" "$context_dir/OpenDoas"
        ;;
    opendoas-rs)
        cp "$CONFORMANCE_DIR/images/opendoas-rs/Containerfile" "$context_dir/Containerfile"
        cp "$ROOT_DIR/Cargo.toml" "$ROOT_DIR/Cargo.lock" "$ROOT_DIR/build.rs" "$context_dir/"
        cp -R "$ROOT_DIR/src" "$context_dir/src"
        ;;
    *)
        die "unknown implementation: $impl"
        ;;
esac

podman build \
    --network=host \
    --build-arg "VARIANT=$variant" \
    -f "$context_dir/Containerfile" \
    -t "$tag" \
    "$context_dir"
