#!/bin/bash

set -ex

command -v podman &> /dev/null || { echo "You need to have podman installed to run this script" >&2; exit 1; }

ROOT=$(dirname $(dirname "${BASH_SOURCE[0]}"))
(
    cd $ROOT
    podman build -t shortcut-builder:latest $ROOT
)
