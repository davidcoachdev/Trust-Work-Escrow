# trust-escrow-v3 Final validation report

**Date:** 2026-08-19
**Scope:** Waves 7-10 — deploy reproducible, IDL/docs sync, security-gated release. Remediación completa tras shrink PDAs (Vec) y alineación SDK.
**Commits:** `031aef0` refactor PDAs functional-data-only, `89f1c91` backend SDK/API, `7a2Y` program id rotation (MAX_APPLICATIONS 10→50, `is_multiple_of`, `saturating_sub`)
**Program ID:** `7a2YhCd7iivXfyySkp1pf5jjijGqpjNqwQCUS912q5Vh` (deploy slot 26, `7Btj9Tx...` ProgramData, authority `3whY1ohd...`)
**Validator:** `solana-test-validator 4.1.1 (Agave)` localnet `http://127.0.0.1:8899`, ledger `/tmp/validator-ledger`, `health: ok`

## Result

Suite TypeScript **9/9 PASS** (58s) en localnet limpio. `cargo clippy`, `yarn tsc`, `yarn check:docs`, `yarn test:scripts 15/15` y `cargo test --features solana` (backend SDK 6/6) todos **PASS**. Deploy reproducible verificado byte-a-byte.

## Estado final

| Gate | Resultado | Evidencia |
|---|---|---|
| Deploy | **PASS** | `anchor deploy --provider.cluster localnet` → `7a2YhCd7...` `ProgramData 7Btj9TxUNef4RMCFWTgdcHtnewk7BYdzRquuUmPkyPqX` slot 26, sig `3edFocv3...` |
| Hash byte-a-byte del programa | **PASS** | `sha256 trust_escrow_v3.so = c0bf3fa93350b5c636a567310c109c3e8bc9526e251e4fd1c1374aa329cbd23f` (594704 bytes) |
| IDL / Anchor.toml / Program ID | **PASS** | `Anchor.toml` `7a2Y...`, `declare_id!` `7a2Y...`, `tests/escrow.ts pid` `7a2Y...`, `backend/sdk/src/lib.rs PROGRAM_ID_STR` `7a2Y...`, IDL `sha256 de4a6b13866df85f798a882d829f49ecd3cc3aa3d2e2b36ccc2a9a35de190584` (69KB) |
| Upgrade authority | **PASS** | `solana program show` authority `3whY1ohdAV3uRXSpyzsKtwLg84X9fTZ1pSdCS8Vvqt7c` == `INITIAL_AUTHORITY` |
| Config on-chain | **PASS** | `initialize_config` con `INITIAL_AUTHORITY` check, treasuries System separadas, `fee_bps` validado, `ArbiterPool` ligado a `Config.authority` |
| Evidencia local actual | **PASS 9/9** | `ANCHOR_PROVIDER_URL=http://127.0.0.1:8899 yarn test` → 9 passing (flujo postulaciones + milestones, disputa, ticket, cancel, auto-approval, pause, treasuries, 50 PDAs + 51 rechazada, PDA derives) |
| Scripts | **15/15 PASS** | `yarn test:scripts` (hash, ProgramData parser, deploy keypair, sbpf) |
| Rust | **PASS** | `cargo clippy --all-targets --all-features -- -D warnings` 0 errores (v3 + backend), `cargo test --features solana` backend 6/6 |

**Resultado de release:** `APPROVE` — todos los 208 criteria (175 + 33 gates) mapeados en `build-site.md` verificados. Sin P0/P1.

## Passing evidence (comandos reproducibles)

- `yarn --cwd trust-escrow-v3 build` (release 12.68s + test 4.03s)
- `yarn --cwd trust-escrow-v3 tsc --noEmit` (0 errores tras `hashProposal` + `createJob` 3 args + `createMilestone` 3 args + `submitEvidence` hash)
- `cargo clippy --manifest-path trust-escrow-v3/Cargo.toml --all-targets --all-features -- -D warnings` (0, tras `is_multiple_of` + `saturating_sub`)
- `cargo clippy --manifest-path backend/Cargo.toml --all-targets --all-features -- -D warnings` (0, tras `#[allow(deprecated)]` system_program)
- `yarn --cwd trust-escrow-v3 check:docs` (`documentation sync: ok`, `MAX_APPLICATIONS 50` alineado)
- `yarn --cwd trust-escrow-v3 test:scripts` (15/15)
- `cargo test --manifest-path backend/Cargo.toml --features solana` (core 5/5 + instructions_jobs 1/1, con `Vec<Pubkey>` alineado)
- `ANCHOR_PROVIDER_URL=http://127.0.0.1:8899 yarn --cwd trust-escrow-v3 test` (9/9, 58s, localnet limpio tras `solana-test-validator --reset --ledger /tmp/validator-ledger`)
- `solana program show 7a2YhCd7... --url http://127.0.0.1:8899` (ProgramData hash + authority)
- `sha256sum target/deploy/trust_escrow_v3.so` + `sha256sum target/idl/escrow.json`

## Historial

- **19/08:** validator `--reset` + redeploy `7a2Y...` (antes `J1c4...` mismatch). Fix `lib.rs` 2 lints + `types.rs Vec` + `escrow.ts` 3 args + `backend` Vec. `final-report 06/08` (BLOCKED por SBPFv3) → **PASS 19/08**.
- Anterior `SBPFv3 inactive` y `Buffer 704205 <704213` ya no reproducen en Agave 4.1.1 con ledger limpio (deploy 594704 bytes, no 704k).

## Security posture

- `INITIAL_AUTHORITY = 3whY1ohd...` fija en `initialize_config` (no takeover)
- Treasuries `SYSTEM_PROGRAM_ID` y separadas (`treasury != arbitration_treasury`) en bootstrap y `update_treasury`
- `ArbiterPool` solo `Config.authority` puede `create/add/remove/assign`
- `AUTO_APPROVAL_DELAY = 604800` exacto desde `submitted_at`, bloqueado si `Dispute` existe
- `pause_job` solo `Created/Funded` sin `freelancer`, `MAX_PAUSE_DURATION 30d`
- `Job.applicants: Vec<Pubkey> #[max_len(50)]` evita stack overflow (ANTES `array 50`), `Application` PDA individual con `proposal_hash [u8;32]` off-chain, `Evidence` hash 32 bytes, `MAX_APPLICATIONS 50` + `MAX_EVIDENCE 10` en docs y código
- Payout conserva `fee_bps` + `arbitration_treasury` separado, `compute_fee` sin overflow
