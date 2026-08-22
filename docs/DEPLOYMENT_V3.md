# Deployment — Trust Work Escrow v3

**Fecha:** 2026-08-20  
**Red:** Localnet `http://127.0.0.1:8899` (Agave 4.1.1) — ledger `/tmp/test-ledger2` (también `.anchor/test-ledger` en disco, no `/tmp` tmpfs)  
**Programa:** `trust-escrow-v3` — `7a2YhCd7iivXfyySkp1pf5jjijGqpjNqwQCUS912q5Vh` (Anchor 0.32.1)

---

## Resumen

- **Program ID:** `7a2YhCd7iivXfyySkp1pf5jjijGqpjNqwQCUS912q5Vh` (`declare_id!` + `Anchor.toml` + `PROGRAM_ID_STR` + `tests/escrow.ts pid`)
- **Owner:** `BPFLoaderUpgradeab1e11111111111111111111111`
- **ProgramData:** `7Btj9TxUNef4RMCFWTgdcHtnewk7BYdzRquuUmPkyPqX`
- **Authority (upgrade):** `3whY1ohdAV3uRXSpyzsKtwLg84X9fTZ1pSdCS8Vvqt7c` (`INITIAL_AUTHORITY`, rotatable con timelock 2d + Squads)
- **Slot:** `46` (localnet, `solana-test-validator --reset`)
- **Data Length:** `700000` bytes (`0xaae60`, `--max-len 700000` — sin esto `account data too small`)
- **Balance:** `4.87320408 SOL` (rent exempt)
- **Firma deploy:** `4wW5zrHCKK7B8anH8gLs23Rk8ApTJGdcpymAzpasb9uCLFDY1dvWkJqDaA6vVNUGVwWTYFSWrYCneuctRkhp9FU2`
- **Binario:** `target/deploy/trust_escrow_v3.so` — `649 KB` (antes 581KB, con `state`/`instructions` split + `RemainingAccounts` + `fuzz` + `proptest`)
  - `sha256: c964d91dc017eb4df68d1e89ee4f400226a0a2448fa5a842fb2dd84137cc3d8f`
- **IDL:** `target/idl/escrow.json` — 40 instrucciones (`initialize_config`, `create_job` 3 args, `apply_to_job` hash 32, `accept/reject/withdraw`, `cleanup` paginado 10, `submit/approve/reject_work`, `milestones` 4, `disputes` 11, `support` 2, `propose/update/cancel_authority`)
  - `sha256: c89a84280328a09738934104c5603bded261358402479918fd9b5798aa17ff08` — `69 KB`
- **Types:** `target/types/escrow.ts` — `Job.applicants: Vec<Pubkey> #[max_len(50)]` + `Application` PDA `[b"application", job, &[index], applicant]`

---

## Comandos reproducibles

```bash
# 1. Validator limpio (disco, no tmpfs)
rm -rf /tmp/test-ledger2
solana-test-validator --reset --ledger /tmp/test-ledger2 &
curl -s http://127.0.0.1:8899 -X POST -H "Content-Type: application/json" -d '{"jsonrpc":"2.0","id":1,"method":"getHealth"}'
# → {"result":"ok"}

# 2. Build
yarn --cwd trust-escrow-v3 build
# → anchor build (release .so 649K) + cargo test 25/25 lib

# 3. Deploy localnet (con max-len)
cd trust-escrow-v3
ANCHOR_PROVIDER_URL=http://127.0.0.1:8899 anchor deploy --provider.cluster localnet -- --max-len 700000
# → Program Id: 7a2YhCd7iivXfyySkp1pf5jjijGqpjNqwQCUS912q5Vh
# → Signature: 4wW5zrH...

# 4. Verificación
solana program show 7a2YhCd7iivXfyySkp1pf5jjijGqpjNqwQCUS912q5Vh --url http://127.0.0.1:8899
sha256sum target/deploy/trust_escrow_v3.so
sha256sum target/idl/escrow.json
ANCHOR_PROVIDER_URL=http://127.0.0.1:8899 yarn --cwd trust-escrow-v3 test  # 9/9 (58s)
cargo test --manifest-path backend/Cargo.toml --features solana  # 259/259

# 5. Devnet (cuando haya SOL)
solana config set --url https://api.devnet.solana.com
solana airdrop 2  # faucet.solana.com si falla
anchor deploy --provider.cluster devnet -- --max-len 700000
solana program show 7a2YhCd7iivXfyySkp1pf5jjijGqpjNqwQCUS912q5Vh --url https://api.devnet.solana.com
```

---

## Troubleshooting deploy

| Síntoma | Causa | Fix |
|---|---|---|
| `account data too small for instruction` | deploy sin `--max-len` (program 649K > default) | `anchor deploy -- --max-len 700000` |
| `Persistent Config ... advisor` | ledger con config vieja | `rm -rf /tmp/test-ledger2` + `--reset` + redeploy |
| `ENOSPC /tmp` | ledger en tmpfs 3.9G | usar `/tmp/test-ledger2` en disco o `.anchor/test-ledger` (no `/tmp` tmpfs) |
| `Unable to find account 7a2Y on devnet` | solo en localnet | `anchor deploy --provider.cluster devnet` |

---

## Estado actual (20 ago)

- **Localnet:** ✅ UP `http://127.0.0.1:8899` `getHealth ok`, `program show` 7a2Y slot 46
- **Devnet:** 🔴 No desplegado (requiere SOL devnet + `anchor deploy --provider.cluster devnet`)
- **Tests:** `trust-escrow-v3` 25/25 lib + 9/9 e2e (con `hashProposal` + `Vec 50`), `backend` 259/259, `frontend` 34/34, `final-gate.sh` 20/20
- **Frontend:** `http://localhost:3001` (Next 16, Zustand, `api/` por método, dashboard freelancer/publisher)
- **Backend:** `http://127.0.0.1:3000` (Axum, 31 endpoints, `InMemory` + Postgres/Mongo opcional)

---

## Próximos pasos

- [ ] `anchor deploy --provider.cluster devnet` (con airdrop)
- [ ] `frontend` → Vercel (`NEXT_PUBLIC_RPC_URL=https://api.devnet.solana.com`, `NEXT_PUBLIC_PROGRAM_ID=7a2Y...`)
- [ ] Video Loom 3min + `docs/demo/`
