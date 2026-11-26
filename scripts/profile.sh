#!/usr/bin/env bash
# CPU + memory profiling for the headless API under load.
#
#   scripts/profile.sh <blink-id> [seconds]
#
# Linux: uses `perf` via cargo-flamegraph; macOS: uses `samply` if present.
# Also captures peak RSS and context switches with /usr/bin/time.
set -euo pipefail
cd "$(dirname "$0")/.."
BLINK=${1:?usage: scripts/profile.sh <blink-id> [seconds]}
SECS=${2:-20}
mkdir -p profiles

cargo build --profile profiling -p blinker-server --bin blinker-api
BIN=target/profiling/blinker-api

if [[ "$(uname)" == "Linux" ]]; then
  TIME="/usr/bin/time -v"
  if command -v perf >/dev/null; then
    perf record -F 999 -g -o profiles/perf.data -- "$BIN" & PID=$!
  else
    $TIME "$BIN" 2> profiles/time.txt & PID=$!
  fi
else
  TIME="/usr/bin/time -l"
  if command -v samply >/dev/null; then
    samply record --save-only -o profiles/samply.json -- "$BIN" & PID=$!
  else
    $TIME "$BIN" 2> profiles/time.txt & PID=$!
  fi
fi

until curl -sf localhost:3000/healthz >/dev/null; do sleep 0.3; done
python3 scripts/loadtest.py --blink "$BLINK" --seconds "$SECS" --conns 32 --post-ratio 0.2 | tee profiles/load.txt
kill -INT $PID; wait $PID || true

if [[ -f profiles/perf.data ]] && command -v inferno-flamegraph >/dev/null; then
  perf script -i profiles/perf.data | inferno-collapse-perf | inferno-flamegraph > profiles/flamegraph.svg
  echo "flamegraph: profiles/flamegraph.svg"
fi
[[ -f profiles/samply.json ]] && echo "open with: samply load profiles/samply.json"
[[ -f profiles/time.txt ]] && grep -Ei "maximum resident|context switches|elapsed" profiles/time.txt || true
