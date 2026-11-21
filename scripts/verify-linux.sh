#!/usr/bin/env bash
# Linux release smoke checks.
#
# Run from the client checkout on an x86_64 Linux host with the sibling
# `../rune` and `../Runecord-backend` checkouts in place:
#
#   bash scripts/verify-linux.sh
#
# The GitHub workflow runs the same commands on ubuntu-22.04.
set -euo pipefail

cargo fmt --check
cargo test -p runecord
cargo build --release -p runecord --features embedded-server

echo
echo "Linux client built at target/release/runecord"
