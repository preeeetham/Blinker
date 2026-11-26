#!/usr/bin/env bash
# Runs every criterion benchmark (HTML reports land in target/criterion).
set -euo pipefail
cd "$(dirname "$0")/.."
cargo bench -p blinker-solana -p blinker-eventlog -p blinker-analytics "$@"
echo "reports: target/criterion/report/index.html"
