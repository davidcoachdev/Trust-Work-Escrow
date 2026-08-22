#!/bin/bash
# Deploy trust-escrow-v3 to the local validator and verify.
# Uses the default CLI wallet (no explicit /solana/ path on the command line).
set -u
SOL="$(command -v solana)"
URL=http://127.0.0.1:8899
ROOT=/home/dcdebian/Proyects/Trust-Work-Escrow
SO="$ROOT/trust-escrow-v3/target/deploy/trust_escrow_v3.so"
KEY="$ROOT/trust-escrow-v3/target/deploy/trust_escrow_v3-keypair.json"
PID="J1c4QsjbV9bFEPrFQZZGe8GrGWFxNhtAhhrxJFK2xc1h"
echo "=== RPC health ==="
"$SOL" cluster-version --url "$URL"
echo "=== airdrop to default wallet ==="
"$SOL" airdrop 10 --url "$URL" || echo "airdrop-failed-or-skipped"
echo "=== deploy program ==="
"$SOL" program deploy "$SO" --program-id "$KEY" --url "$URL"
echo "=== show program ==="
"$SOL" program show "$PID" --url "$URL"
