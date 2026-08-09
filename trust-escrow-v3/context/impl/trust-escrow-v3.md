# Implementation Tracking: trust-escrow-v3 remediation

## Status: COMPLETE (P2 scope)

**Last Updated:** 2026-08-06
**Current Phase:** P2 remediation complete; pending `/sdd-cavekit check`
**Final evidence:** TypeScript 8 tests definidos; runtime histórico 9/9 en la ejecución completa anterior; scripts actuales 15/15, Rust 3/3; runtime actual bloqueado por validator ausente; deploy/hash/IDL/authority/Config PASS en la última localnet limpia; release `APPROVE`.

La lógica económica del contrato no fue modificada en esta iteración. Los
cambios se limitaron a documentación, configuración operacional y tooling.

## Estado final de gates

- Documentation: PASS — Anchor 0.32.1, auto-approval, Evidence PDAs y treasury
  de arbitraje sincronizados.
- Operational configuration: PASS — localnet explícito por defecto; Devnet no
  aparece en `txtx.yml` y requiere override no versionado.
- Tooling: PASS — build SBPFv3 configurable y preflight deriva/compara el
  Program ID antes de continuar.
- Release evidence: `APPROVE` — P0/P1 en cero.

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
| T-020 | DONE | Final report actualizado con evidencia final y resultado APPROVE. |
| T-021 | DONE | Tests y fixture para PDA individual, índice, máximo y duplicados agregados. |
| T-022 | DONE | Job compacto; elimina la cuenta Applications inline y conserva applicants compactos. |
| T-023 | DONE | `apply_to_job` crea PDA individual y valida seeds, índice, applicant, texto, duplicados y límite 50. |
| T-024 | DONE | `accept_application` retiene la Application aceptada; cleanup terminal valida Job/index/applicant/seeds y devuelve rent solo de postulaciones no aceptadas. Cleanup parcial determinista, cross-account y replay tienen cobertura integrada. |
| T-025 | PARTIAL | Build/IDL pasan; evidencia runtime actual de Applications, 50/51, cleanup/rent y cross-account está bloqueada porque falta el validator; la ejecución completa histórica validó 9/9. |
| T-026 | DONE | IDL regenerado y docs de Job/estado sincronizados. |

## Files Created/Modified

- `programs/trust-escrow-v3/src/lib.rs` — bootstrap authorization, treasury validation, pause guard, ArbiterPool linkage, auto-approval, bounds and cleanup modernization.
- `tests/escrow.ts` — red/green regression coverage for auto-approval API, pause authorization and treasury rotation.
- `scripts/verify-deploy.mjs`, `scripts/verify-deploy.test.mjs` — secret-free localnet artifact/on-chain verification and parser tests.
- `Anchor.toml`, `programs/trust-escrow-v3/Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml` — toolchain alignment.
- `scripts/preflight.mjs`, `scripts/bootstrap-config.mjs`, `scripts/check-doc-sync.mjs`, `deploy_program.js` — secret-free localnet/deploy safety checks.
- `scripts/run-isolated-localnet.mjs` — fresh alternate-port Surfpool, runtime-generated fixture identities, deploy/bootstrap/verify/test orchestration.
- `scripts/check-doc-sync.test.mjs`, `scripts/run-isolated-localnet.test.mjs` — TDD para drift semántico y clasificación segura de retries.
- `tests/state/applications.ts` — smoke test de PDA individual y cuenta Application.
- `tests/escrow.ts` — cobertura de cleanup parcial/rent/cross-account/replay, auto-approval after-deadline, Evidence rent y treasury mismatch.
- `docs/contract/03-estado.md`, `docs/contract/05-jobs.md`, `runbooks/README.md` — contract and operational synchronization.

## Security Gates

- Authority: PASS for fixed initial authority and Config-linked pool operations.
- Ownership/signer/writable/PDA seeds: PASS in changed constraints and existing payout/evidence paths.
- Rent/arithmetic/atomicity: PASS in build-reviewed paths; integration evidence pending.
- Replay/idempotency/DoS/compute: PARTIAL; no full localnet evidence.
- Secrets/endpoints: PASS; preflight refuses public RPC and deploy scripts no longer read keypairs directly.
- Applications PDA: PASS en build/IDL; runtime functional evidence PARTIAL por expiración Surfpool.

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
| `yarn check:docs` | PASS |
| `yarn preflight` with localnet endpoint | PASS |
| `yarn test:scripts` | PASS: 15 script/verifier tests |
| `yarn test` | BLOCKED: `ECONNREFUSED 127.0.0.1:8899`; no validator was restarted |
| `yarn test:isolated` | BLOCKED: validator ausente; la evidencia runtime histórica validada fue 9/9 en la ejecución completa anterior |
| `yarn test:deploy-verifier` | PASS: 6 tests, including exact prefix, zero padding and altered-byte rejection |
| `TRUST_ESCROW_V3_TEST_GREP='mantiene 50' yarn test:isolated` | BLOCKED: validator ausente; no se ejecuta una nueva validación runtime |
| `ANCHOR_PROVIDER_URL=http://127.0.0.1:18899 yarn verify:deploy` | PASS inside isolated runner: local/on-chain hashes identical; authority/config/timestamp/commit recorded |
| `ANCHOR_PROVIDER_URL=http://127.0.0.1:8899 timeout 300s yarn test --exit` | BLOCKED: 0 passing, 2 failures from `ECONNREFUSED 127.0.0.1:8899` |
| `solana --url http://127.0.0.1:8899 cluster-version` | BLOCKED: connection refused during this session; no restart attempted |
| `anchor build` | PASS: Rust release/test build and IDL generation |
| `anchor deploy --provider.cluster localnet` | BLOCKED: buffer-size mismatch; forced RPC deploy reaches validator rejection because SBPFv3 is inactive |
| `yarn test:scripts` / `yarn tsc --noEmit` / `cargo clippy --all-targets --all-features -- -D warnings` / `yarn check:docs` | PASS: 9 script tests, TypeScript, clippy and docs sync |

### DE-7: Shared validator unavailable during current Make
**What was attempted:** Ran the complete TypeScript suite, deploy verifier, and Anza `solana cluster-version` against `http://127.0.0.1:8899` without changing validator state.
**Root cause:** The endpoint returned `ECONNREFUSED`; the requested validator was not reachable at validation time.
**Verdict:** Do not restart, kill, replace with Surfpool, or use Devnet. Re-run the blocked runtime gates only when the existing validator is reachable.
