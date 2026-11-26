#!/usr/bin/env bash
# One-command local stack: Postgres DB, Solana test validator, wallets, app.
#
#   scripts/dev.sh            # localnet (free, offline SOL)
#   CLUSTER=devnet scripts/dev.sh
set -euo pipefail
cd "$(dirname "$0")/.."
CLUSTER=${CLUSTER:-localnet}
DB=${DATABASE_URL:-postgres://$(whoami)@localhost:5432/blinker}

command -v cargo-leptos >/dev/null || cargo install cargo-leptos --locked
rustup target add wasm32-unknown-unknown >/dev/null

psql -lqt 2>/dev/null | cut -d'|' -f1 | grep -qw blinker || createdb blinker
[[ -f .env ]] || cargo run -q -p blinker-cli -- setup --cluster "$CLUSTER" --database-url "$DB"

mkdir -p data
if [[ "$CLUSTER" == "localnet" ]] && ! solana cluster-version --url http://127.0.0.1:8899 >/dev/null 2>&1; then
  echo "starting solana-test-validator (logs: data/validator.log)"
  solana-test-validator --quiet --ledger data/ledger > data/validator.log 2>&1 &
  until solana cluster-version --url http://127.0.0.1:8899 >/dev/null 2>&1; do sleep 0.5; done
fi

exec cargo leptos watch
