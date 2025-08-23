#!/bin/bash
# Test runner script for embedded Rust project
# This is necessary because the project targets a microcontroller by default

echo "Running unit tests on host target..."
cargo test --target x86_64-unknown-linux-gnu --lib "$@"