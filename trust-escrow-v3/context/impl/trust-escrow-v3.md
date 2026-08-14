# Implementation Tracking: trust-escrow-v3 remediation

## Status: IN_PROGRESS (T-027/T-029 localnet happy path verified; backend-v3 release gates remain partial)

**Last Updated:** 2026-08-13 (Iter 8)
**Current Phase:** backend-v3 Make; fresh localnet validator reachable and advancing, focused SDK flow verified, dedicated replay/finality and signer/deploy evidence remain incomplete
**Evidence note:** TypeScript 8 tests definidos y evidencia runtime histórica 9/9
no son evidencia vigente. Runtime actual está bloqueado por validator ausente;
deploy/hash/authority/Config no deben presentarse como PASS actual. Backend v3
tiene 27 mappings de métodos SDK source-backed; API/application service,
proyección/sync DB, idempotencia/finality/reconciliación y signer runtime gates
siguen planned o blocked por las dependencias del build site.

La lógica económica del contrato no fue modificada en esta iteración. Los
cambios se limitaron a documentación, configuración operacional y tooling.

## Estado final de gates

- Documentation: PASS — Anchor 0.32.1, auto-approval, Evidence PDAs y treasury
  de arbitraje sincronizados; backend v3 boundary documentado como plan.
- Operational configuration: PASS — localnet explícito por defecto; Devnet no
  aparece en `txtx.yml` y requiere override no versionado.
- Tooling: PASS — build SBPFv3 configurable y preflight deriva/compara el
  Program ID antes de continuar.
- Release evidence: `BLOCKED` para runtime actual; el APPROVE histórico no se
  reutiliza como evidencia vigente.

## Historial Make anterior

## Task Status

| Task | Status | Notes |
|---|---|---|
| T-001 | DONE | Anchor/Rust/JS aligned to 0.32.1; Rust 1.89 pinned. |
| T-002 | PARTIAL | Localnet preflight and safe public-key fixtures added; persistent advisor remains unavailable. |
| T-003 | PARTIAL | Regression tests added; complete anti-frontrun suite blocked by persistent Config. |
| T-004 | PARTIAL | ArbiterPool is linked to Config authority in code; dedicated negative suite not completed. |
| T-005 | PARTIAL | Auto-approval uses inclusive `submitted_at + 604800`; exact boundary is covered by Rust unit test and after-deadline integration test, but RPC execution is blocked. |
| T-006 | PARTIAL | pause/dispute guards implemented; replay/concurrency suite blocked. |
| T-007 | PARTIAL | Evidence cleanup now asserts recovered rent; full conservation/replay suite remains blocked by unavailable RPC. |
| T-008 | DONE | Contract docs inventory updated for Submitted, deadlines, limits and treasury separation. |
| T-009 | PARTIAL | Bootstrap validates fixed initial authority, non-null/distinct system treasuries and fee bounds; rotation validation now added and needs localnet execution. |
| T-010 | DONE | Added exact `submitted_at + 604800` auto-approval and constrained pause behavior. |
| T-011 | PARTIAL | Existing cleanup preserved; auto-approval close path added; complete terminal matrix blocked. |
| T-012 | DONE | Pool create/add/remove/assign require Config authority linkage. |
| T-013 | PARTIAL | Existing direct payouts/refund/AcceptDispute fixes preserved; full negative payout suite blocked. |
| T-014 | PARTIAL | Validator real responde `4.1.1` en 8899, pero la feature SBPFv3 está inactiva; no se usa Surfpool. |
| T-015 | DONE | Verifier now proves current `.so` byte hash, ProgramData upgrade authority, IDL/Anchor.toml/program ID, Config readback, timestamp and git commit on isolated Surfpool. |
| T-016 | PARTIAL | IDL regenerated; `check:docs` ahora valida semántica de Evidence PDA, `arbitration_treasury`, Submitted y ausencia de Received en todo `docs/`. |
| T-017 | PARTIAL | Se reordenó duplicado para probar `AlreadyApplied` antes del límite; se agregaron negativos de arbiter y evidencia/treasury; la suite completa sigue expirada bajo carga. |
| T-018 | DONE | Build, TypeScript, docs drift check, deploy-verifier unit tests and clippy gates pass. |
| T-019 | PARTIAL | Deploy al validator real no puede comenzar: `invalid account data` porque SBPFv3 está inactiva; Program account ausente, por lo que hash on-chain/authority/Config no son verificables. |
| T-020 | PARTIAL | Final report reclasificado: baseline histórico; runtime actual BLOCKED. |
| T-021 | DONE | Tests y fixture para PDA individual, índice, máximo y duplicados agregados. |
| T-022 | DONE | Job compacto; elimina la cuenta Applications inline y conserva applicants compactos. |
| T-023 | DONE | `apply_to_job` crea PDA individual y valida seeds, índice, applicant, texto, duplicados y límite 50. |
| T-024 | DONE | `accept_application` retiene la Application aceptada; cleanup terminal valida Job/index/applicant/seeds y devuelve rent solo de postulaciones no aceptadas. Cleanup parcial determinista, cross-account y replay tienen cobertura integrada. |
| T-025 | PARTIAL | Build/IDL pasan; evidencia runtime actual de Applications, 50/51, cleanup/rent y cross-account está bloqueada porque falta el validator; la ejecución completa histórica validó 9/9. |
| T-026 | DONE | IDL regenerado y docs de Job/estado sincronizados. |

## Backend v3 Make status

| Task range | Status | Notes |
|---|---|---|
| T-027 | PARTIAL | Fresh `/tmp/twe-ledger4` validator is reachable at `127.0.0.1:8899`; focused SDK boundary flow passes, but API/TUI equivalence and full threat-model evidence remain pending. |
| T-028 | PARTIAL | Offline signer/endpoint gate added: public and arbitrary custom clusters rejected before keypair loading; full signer-mode/runtime checks remain pending. |
| T-029 | PARTIAL | Fresh validator is advancing and the SDK happy path passes; replay, atomicity, finality, reorg and tombstone evidence are not covered by the focused test. |
| T-030–T-046 | BLOCKED | Dedicated backend contracts/tests are still prerequisites; only the existing SDK runtime smoke path was executed. |
| T-047–T-061 | NOT_STARTED/BLOCKED | Implementation and release gates remain downstream of the blocked contract/test waves. |

## Files Created/Modified

- `programs/trust-escrow-v3/src/lib.rs` — bootstrap authorization, treasury validation, pause guard, ArbiterPool linkage, auto-approval, bounds and cleanup modernization.
- `tests/escrow.ts` — red/green regression coverage for auto-approval API, pause authorization and treasury rotation.
- `scripts/verify-deploy.mjs`, `scripts/verify-deploy.test.mjs` — secret-free localnet artifact/on-chain verification and parser tests.
- `Anchor.toml`, `programs/trust-escrow-v3/Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml` — toolchain alignment.
- `scripts/preflight.mjs`, `scripts/bootstrap-config.mjs`, `scripts/check-doc-sync.mjs`, `deploy_program.js` — secret-free localnet/deploy safety checks.
- `scripts/run-isolated-localnet.mjs` — fresh alternate-port Surfpool, runtime-generated fixture identities, deploy/bootstrap/verify/test orchestration.
- `scripts/check-doc-sync.test.mjs`, `scripts/run-isolated-localnet.test.mjs` — TDD para drift semántico y clasificación segura de retries.
- `scripts/sdk-operation-inventory.test.mjs` — TDD para separar 27 mappings de SDK source-backed de 18 operaciones ausentes.
- `../backend/sdk/src/client.rs`, `../backend/sdk/tests/core.rs` — T-028 offline endpoint safety gate; public/custom clusters are rejected before secret loading.
- `tests/state/applications.ts` — smoke test de PDA individual y cuenta Application.
- `tests/escrow.ts` — cobertura de cleanup parcial/rent/cross-account/replay, auto-approval after-deadline, Evidence rent y treasury mismatch.
- `docs/contract/03-estado.md`, `docs/contract/05-jobs.md`, `runbooks/README.md` — contract and operational synchronization.

## Security Gates

- Authority: PASS for fixed initial authority and Config-linked pool operations.
- Ownership/signer/writable/PDA seeds: PASS in changed constraints and existing payout/evidence paths.
- Rent/arithmetic/atomicity: PASS in build-reviewed paths; integration evidence pending.
- Replay/idempotency/DoS/compute: PARTIAL; fresh localnet happy path passes, but dedicated replay/finality evidence is not implemented.
- Secrets/endpoints: PASS; preflight refuses public RPC and deploy scripts no longer read keypairs directly.
- Applications PDA: PASS en build/IDL; runtime functional evidence PARTIAL por expiración Surfpool.
- SDK inventory: PASS — 27 source-backed mappings are current, 18 absent operations
  remain planned, and no mapping is presented as localnet runtime evidence.
- Endpoint/signer preflight: PASS for offline public-cluster rejection and localnet endpoint; full signer runtime gate remains PARTIAL because advisor and expected-authority inputs are unavailable.

## Dead Ends / Environmental Blockers

### DE-1: Public RPC test execution
**What was attempted:** Ran the existing suite with inherited provider configuration.
**Root cause:** `ANCHOR_PROVIDER_URL` was `https://api.devnet.solana.com`, causing 429 faucet responses.
**Verdict:** Do not reattempt. Tests now force/refuse public RPC and require localnet.

### DE-2: Fresh Anchor localnet
**What was attempted:** `anchor test --skip-build --provider.cluster localnet`.
**Root cause:** RPC port 8899 is already occupied by an existing local process.
**Verdict:** Do not reset shared state; use `yarn test:isolated` on the alternate loopback port.

### DE-3: Persistent advisor
**What was attempted:** Local test suite against the already-running local endpoint.
**Root cause:** Existing Config points to an advisor whose `TRUST_ESCROW_V3_ADVISOR_KEYPAIR` is not available.
**Verdict:** Reattempt only when the authorized advisor signer is provisioned through the environment; never store or print it.

### DE-4: Isolated integration allocation
**What was attempted:** Fresh Surfpool on `127.0.0.1:18899`, current `.so` deploy, runtime fixture bootstrap, verifier, then full `yarn test`.
**Root cause:** Anchor inner instructions reject `Applications::INIT_SPACE` (~28KiB) during `create_job` because the account exceeds the 10KiB inner allocation limit.
**Verdict:** Do not bypass the runtime error or use shared state. Redesign application storage or an explicitly supported pre-allocation path before claiming full integration coverage.

### DE-5: Surfpool transaction expiry under application load
**What was attempted:** Fresh alternate-port Surfpool con 50 postulaciones PDA individuales, retries de hasta 3 intentos y backoff lineal.
**Root cause:** El usuario indicó usar validator real; Surfpool queda fuera del alcance de esta iteración y sus expiraciones no son evidencia válida.
**Verdict:** No reintentar Surfpool. Ejecutar solo contra el validator real en 8899.

### DE-6: Real validator rejects current SBF artifact
**What was attempted:** `anchor build`, `anchor deploy --provider.cluster localnet`, `anchor deploy ... --use-rpc` y `solana program deploy --url http://127.0.0.1:8899 --use-rpc`, sin reiniciar ni resetear el validator.
**Root cause:** El RPC reporta `solana-core 4.1.1`; `solana feature status --output json` marca inactiva `BUwGLeF3Lxyfv1J1wY8biFHBB2hrk2QhbNftQf3VV3cC` (SIMD-0178/0179/0189). La clave proporcionada `5cC3...` no corresponde a esa feature en el feature set del cluster. El deploy Anchor además aborta con `Buffer account data size (704205) is smaller than the minimum size (704213)`; al forzar el tamaño, la simulación confirma `Detected sbpf_version required by the executable which are not enabled`. El Program account sigue ausente.
**Verdict:** Setup BLOCKED; no alterar lógica del escrow ni desactivar la validación. Reanudar cuando el validator limpio habilite SBPFv3 o se provea un toolchain compatible que produzca un artifact aceptado sin degradar Anchor/dependencias.

## Test Health

| Command | Result |
|---|---|
| `yarn build` | PASS |
| `yarn tsc --noEmit` | PASS |
| `cargo clippy --all-targets --all-features -- -D warnings` | PASS on Rust 1.89 |
| `cargo test --workspace` | PASS: 4 tests across 6 suites |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | PASS |
| `yarn check:docs` | PASS |
| `node --test scripts/*.test.mjs` | PASS: 16 script/documentation tests |
| `yarn preflight` with localnet endpoint | PASS |
| `yarn test:scripts` | PASS: 15 script/verifier tests |
| `yarn test` with `ANCHOR_PROVIDER_URL=http://127.0.0.1:8899` | BLOCKED: 1 passing, 1 failing; persistent Config requires unavailable `TRUST_ESCROW_V3_ADVISOR_KEYPAIR` |
| `yarn test:isolated` | NOT RUN: user requested local validator only; no Surfpool execution |
| `yarn test:deploy-verifier` | PASS: 6 tests, including exact prefix, zero padding and altered-byte rejection |
| `TRUST_ESCROW_V3_TEST_GREP='mantiene 50' yarn test:isolated` | BLOCKED: validator ausente; no se ejecuta una nueva validación runtime |
| `ANCHOR_PROVIDER_URL=http://127.0.0.1:18899 yarn verify:deploy` | PASS inside isolated runner: local/on-chain hashes identical; authority/config/timestamp/commit recorded |
| `ANCHOR_PROVIDER_URL=http://127.0.0.1:8899 timeout 300s yarn test --exit` | BLOCKED: 0 passing, 2 failures from `ECONNREFUSED 127.0.0.1:8899` |
| `solana --url http://127.0.0.1:8899 cluster-version` | PASS: `4.1.1` on fresh `/tmp/twe-ledger4`; observed slots advancing `11 -> 18` and `334 -> 340` |
| `solana program show J1c4QsjbV9bFEPrFQZZGe8GrGWFxNhtAhhrxJFK2xc1h --url http://127.0.0.1:8899` | PASS: Program ID loaded; ProgramData `97AfHKADS7iVb1bG2LMF65KNH6yTzh3wmr4HgFpJNGzQP`, authority `11111111111111111111111111111111` |
| `cargo test -p trust-escrow-sdk --features solana --test instructions_jobs -- --nocapture` | PASS: 1 test, `16.36s`; config → jobs → applications → work happy path |
| `cargo test -p trust-escrow-sdk --features solana -- --nocapture` | PASS: 15 tests across 5 suites, `15.37s` |
| `cargo test -p trust-escrow-sdk --features solana --test core` | PASS: 6 tests, including public-cluster rejection before keypair loading |
| `cargo test -p trust-escrow-sdk --features solana --test pda` | PASS: 4 tests |
| Secret scan (`gitleaks`, `trufflehog`) | BLOCKED: `gitleaks` and `trufflehog` unavailable; no scanner evidence claimed |
| Dependency audit (`cargo audit`, `cargo-deny`) | BLOCKED: executables unavailable; no advisory/license evidence claimed |
| Coverage (`cargo-llvm-cov`) | BLOCKED: executable unavailable; no coverage percentage claimed |
| `anchor build` | PASS: Rust release/test build and IDL generation |
| `anchor deploy --provider.cluster localnet` | BLOCKED: buffer-size mismatch; forced RPC deploy reaches validator rejection because SBPFv3 is inactive |
| `yarn test:scripts` / `yarn tsc --noEmit` / `cargo clippy --all-targets --all-features -- -D warnings` / `yarn check:docs` | PASS: 9 script tests, TypeScript, clippy and docs sync |

### DE-7: Shared validator unavailable during current Make
**What was attempted:** Ran the complete TypeScript suite, deploy verifier, and Anza `solana cluster-version` against `http://127.0.0.1:8899` without changing validator state.
**Root cause:** The endpoint returned `ECONNREFUSED`; the requested validator was not reachable at validation time.
**Verdict:** Do not restart, kill, replace with Surfpool, or use Devnet. Re-run the blocked runtime gates only when the existing validator is reachable.

### DE-8: Loopback validator still unavailable on 2026-08-13
**What was attempted:** Verified `http://127.0.0.1:8899` before runtime work and after the offline SDK change; checked `/tmp/twe-ledger3` presence.
**Root cause:** Both loopback probes returned connection refused. The ledger directory exists, but that is not runtime evidence and does not justify starting/replacing the validator.
**Verdict:** Do not restart, kill, replace with Surfpool, or use Devnet. Resume runtime tasks only after the user-provided validator is reachable.

### DE-9: Existing v3 ledger starts but does not persist slots
**What was attempted:** Started `solana-test-validator --ledger /tmp/twe-ledger3 --rpc-port 8899 --faucet-port 9900 --bind-address 127.0.0.1` in the current execution context; `solana cluster-version` succeeded; ran `cargo test -p trust-escrow-sdk --features solana --test instructions_jobs`.
**Root cause:** Validator restored `/tmp/twe-ledger3` at processed/confirmed/finalized slot `1400` and remained at slot `1400` for a 2-second probe. The SDK integration test emitted only the 60-second harness warning and timed out at 90 seconds (exit 124), indicating no advancing ledger/transaction confirmation. Validator log shows repeated `Couldn't vote on heaviest fork` at slot `2176` after startup and no useful transaction progress.
**Verdict:** Do not claim runtime integration evidence or retry this stalled ledger. Do not reset it, use Surfpool, or use Devnet. A fresh compatible local validator/ledger with advancing slots is required.

### DE-10: Fresh localnet full suite still requires persistent advisor
**What was attempted:** Ran `ANCHOR_PROVIDER_URL=http://127.0.0.1:8899 yarn test` against fresh `/tmp/twe-ledger4`.
**Root cause:** The fresh ledger reaches a persistent Config whose advisor signer is not available in the execution context; the suite fails in its `before all` hook before the full flow.
**Verdict:** Do not guess or print the signer. Re-run only when the authorized advisor signing fixture is provisioned through the environment; the focused SDK integration remains valid and passing.

### DE-11: Deploy verifier requires explicit expected authority
**What was attempted:** Ran `ANCHOR_PROVIDER_URL=http://127.0.0.1:8899 yarn verify:deploy`.
**Root cause:** `TRUST_ESCROW_V3_EXPECTED_AUTHORITY` is intentionally required as public identity input; it was not provided, so verification stopped before claiming authority/hash evidence.
**Verdict:** Re-run with the authorized public authority value only; never substitute a guessed identity or secret.

### Iter 7 validation additions

| Command | Result |
|---|---|
| `solana-test-validator --ledger /tmp/twe-ledger3 --rpc-port 8899 --faucet-port 9900 --bind-address 127.0.0.1` | STARTED locally; `cluster-version` PASS (`4.1.1`), but ledger stalled at slot 1400 and validator was stopped |
| `cargo test -p trust-escrow-sdk --features solana --test instructions_jobs -- --nocapture` | BLOCKED: no progress beyond test start; timeout after 90s, exit 124 |
| `yarn build` / `yarn tsc --noEmit` | PASS |
| `cargo test --workspace` | PASS: 4 tests across 6 suites |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | PASS |
| `ANCHOR_PROVIDER_URL=http://127.0.0.1:8899 yarn preflight` | PASS: localnet-only endpoint/program/payer checks |
| `yarn test:deploy-verifier` | PASS: 6 tests |
| `yarn test:scripts` | PASS: 16 tests |
| `cargo test -p trust-escrow-sdk --features solana --test pda` | FLAKY: cache benchmark once measured 1.6910 ms; exact benchmark rerun PASS |
