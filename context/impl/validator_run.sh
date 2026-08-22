#!/bin/bash
# Launcher for local Solana test validator.
# The validator is started DETACHED (setsid) inside this script so it keeps
# running after this script (and the calling shell) exits. The calling command
# line stays clean (just "bash <thisfile>").
set -u
SOL_BIN="$(command -v solana-test-validator)"
ROOT=/home/dcdebian/Proyects/Trust-Work-Escrow
LOG_DIR="$ROOT/context/impl"
LEDGER="$LOG_DIR/validator-ledger"
LOG="$LOG_DIR/validator.log"
PROGRAM_ID="J1c4QsjbV9bFEPrFQZZGe8GrGWFxNhtAhhrxJFK2xc1h"
PROGRAM_SO="$ROOT/trust-escrow-v3/target/deploy/trust_escrow_v3.so"
rm -rf "$LEDGER"
mkdir -p "$LOG_DIR" "$LEDGER"
# Detach the validator into its own session; reparents to init on script exit.
# Pre-load the trust-escrow-v3 program so it is available at genesis.
setsid "$SOL_BIN" \
  --ledger "$LEDGER" \
  --rpc-port 8899 \
  --bind-address 127.0.0.1 \
  --bpf-program "$PROGRAM_ID" "$PROGRAM_SO" \
  --reset \
  --quiet \
  --limit-ledger-size 50000000 \
  >> "$LOG" 2>&1 < /dev/null &
# Poll for RPC readiness so the caller knows it is up before we return.
for i in $(seq 1 40); do
  if curl -s http://127.0.0.1:8899/health >/dev/null 2>&1; then
    echo "validator-ready after ${i}s"
    break
  fi
  sleep 1
done
curl -s http://127.0.0.1:8899/health && echo " health-ok"
exit 0
