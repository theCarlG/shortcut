#!/bin/bash
set -eux

cargo test --workspace --doc
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings -W clippy::all
cargo fmt --all -- --check
cargo doc --no-deps
cargo deny check
