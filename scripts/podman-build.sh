#!/bin/bash

set -ex

command -v podman &> /dev/null || { echo "You need to have podman installed to run this script" >&2; exit 1; }

ROOT=$(dirname $(dirname "${BASH_SOURCE[0]}"))
CC="cargo"
XC="podman run --name shortcut-builder --rm -v $ROOT:/usr/src/app -w /usr/src/app shortcut-builder $CC"

(
    cd $ROOT
    $XC build --release --bin shortcut-gui --target=x86_64-unknown-linux-gnu
    $XC build --release --bin shortcut-daemon --target=x86_64-unknown-linux-musl
)
