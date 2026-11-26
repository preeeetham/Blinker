#!/usr/bin/env bash
# Runs each fuzz target for N seconds (default 60). Needs nightly + cargo-fuzz.
set -euo pipefail
cd "$(dirname "$0")/../fuzz"
SECS=${1:-60}
for t in tx_decode event_decode log_recovery; do
  echo "== $t ($SECS s)"
  cargo +nightly fuzz run "$t" -- -max_total_time="$SECS" -max_len=1232
done
