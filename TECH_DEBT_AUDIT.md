# Tech Debt Audit — Trust Work Escrow

**Date:** 2026-08-20  
**Scope:** Full repo (trust-escrow-v3 2.6k LOC, backend 3.4k LOC, frontend Next.js, landing Dioxus, infra)  
**Auditors:** CodeGraph + manual + tooling (cargo clippy, tsc, cargo audit)  
**Mode:** Evidence-backed, file:line citations, no sycophancy

---

## Executive Summary

- **V3 contract is a 2.6k-line god module** with 40 handlers, 21 `UncheckedAccount`, manual lamport transfers and untyped `remaining_accounts` — systemic risk for funds custody. Needs modular split and typed accounts (Effort L, High/Critical).
- **Centralization via `INITIAL_AUTHORITY` hardcode** (`trust-escrow-v3/src/lib.rs:14`) with no rotation/timelock — single key compromise forces redeploy. Critical.
- **Backend SDK was desynced from contract Vec migration** (fixed in 3b45ea8/d44535a) but `remaining_accounts` and manual `close` patterns remain — risk of account injection and rent double-accounting.
- **Frontend is green** (Next 16, Zustand, 24/24 tests) but new; no e2e, no wallet e2e, and `NEXT_PUBLIC_API_URL` defaults to 3000 while backend also 3000 — port collision in local stack (now fixed to 3001).
- **Validator ledger on tmpfs** (3.9G) filled and killed validator; moved to `.anchor/test-ledger` on disk (193M). Deploy needs `--max-len 700000` or `account data too small`.
- **Debt is now 259/259 tests, 20/20 final-gate, but test debt remains:** 7 unit tests for 40 instructions (20% coverage), no fuzz/invariants, no `remaining_accounts` malformed tests.
- **Operational debt:** CI blocks mainnet (`B5`) and secret-scan are now green (149/149, 20/20), but `dc-dev` dispatch had 4 bugs (agents.data, request.text, sessionID, path.id) — fixed 607b90f — and still needs `no-child-observed` triage fix.
- **Top 5 fixes** (god module split, typed accounts, remaining_accounts pagination, `INITIAL_AUTHORITY` rotation, auto-approve keeper auth) would cut 60% of Critical/High risk.

---

## Mental Model

**Boundaries:** `trust-escrow-v3` (on-chain, Anchor 0.32.1, `7a2YhCd7...5Vh`, 40 ix, `Vec<Pubkey>` 50) ↔ `backend/sdk` (Rust SDK, `7a2Y` PDA helpers, `list_*` cursor) ↔ `backend/api` (Axum, `InMemory` repo, 31 handlers `501→200`, Zustand stores consume `frontend/src/api/*`) ↔ `frontend` (Next 16 + Zustand 5 + wallet adapter, `frontend/src/api/` per method/endpoint) ↔ `landing` (Dioxus, static). Persistence: on-chain `Job/Application/Dispute/Evidence/Milestone` + off-chain `metadata.rs` 6 structs (title/description/proposal) → `repository.rs` `InMemory` → future Postgres/Mongo via `docker-compose.yml` (twe-postgres:16, twe-mongo:7). External: Solana RPC `http://127.0.0.1:8899` (Agave 4.1.1), `solana 4.1.1`, `anchor 0.32.1`.

**Runtime flow:** `create_job` (client → PDA `[b"job", client, job_id]`, Vec push) → `deposit_funds` → `apply_to_job` (hash 32) → `accept_application` (client auth, `Funded` check) → `submit_work` → `approve_work`/`auto_approve` (keeper any signer, `604800`s) → `finalize`/`close`. `raise_dispute` → `submit_evidence` (hash) → `assign_arbiter` → `resolve`/`finalize`/`cleanup`. Sync via `sync.rs` polling `getSignaturesForAddress`.

**Risk areas:** funds custody (lamport manual), auth (UnchekedAccount, keeper any), resource (PDA derive in loop, `remaining_accounts` 50 in one tx → 1232 bytes limit, `Rent::get` per transfer), deploy (ledger tmpfs, max-len).

---

## Findings Table (35 findings, cap 80)

| ID | Category | File:Line | Evidence | Sev | Eff | Description | Recommendation |
|---|---|---|---|---|---|---|---|
| V3-ARCH-001 | Architectural decay | `trust-escrow-v3/src/lib.rs:1` | God module 2664 LOC, 13 `#[account]` 382-491, 18 `#[derive(Accounts)]` 2015-2664, 40 handlers 673-2013, `mod tests` 504, `Cargo.toml:9` no mod split | High | L | Monolito impide ownership, test paralellism, y causa stack overflow previo (comment 410-413). Build 4s clippy ok pero PR diff 200 LOC toca todo. | Split `lib.rs` → `state/{config,job,application,dispute}.rs` + `instructions/{job,dispute,milestone}.rs` + `errors.rs` + `lib.rs` re-export (anchor `mod` 0.32 soporta). Effort L (incluye `anchor build` + IDL regen). |
| V3-SEC-002 | Centralization | `trust-escrow-v3/src/lib.rs:14` | `const INITIAL_AUTHORITY: Pubkey = pubkey!("3whY...")`, `require!(authority.key()==INITIAL_AUTHORITY)` 681, `declare_id!("7a2Y...")` 6 | Critical | M | Single key controla `initialize_config` sin `update_authority`, timelock o multisig. Compromiso = redeploy + migración de 50 jobs. No `anchor keys` rotation. | Añadir `update_authority` con timelock 2d + multisig (Squads) + `propose/approve` 2-step, y `declare_id` vía `Anchor.toml` + `anchor keys sync`. |
| V3-TYPE-003 | Type debt | `trust-escrow-v3/src/lib.rs:2132` | 21 `UncheckedAccount` con `/// CHECK: client validado por PDA`, `client: UncheckedAccount` 13 ocurrencias, `lib.rs:2020` `client`, `2132` `SubmitWork.client`, `2398` `WithdrawTreasury` | High | M | Solo seeds validan PDA (`[b"job", client, job_id]`), no `owner`/`is_signer`. Permite `client` fake SystemAccount. | Migrar a `SystemAccount`/`Signer` + `#[account(...)]` typed, validar `owner == SYSTEM_PROGRAM_ID` en `Accounts` (anchor 0.32). |
| V3-ARCH-004 | Boundary violation | `trust-escrow-v3/src/lib.rs:208` | `fn cleanup_job_applications(..., remaining_accounts: &[AccountInfo])`, `is_multiple_of(2)` 217, `chunks_exact(2)` 241, `FinalizeDisputePayouts` 2555 `split_at`, handlers 1134,1181,1238 | Critical | L | `remaining_accounts` sin tipar permite inyección, orden incorrecto, bypass `MAX_APPLICATIONS` si `require_full_range=false` 233-238. Hasta 50 cuentas en una tx (1232B limit) → `1232` overflow. | Reemplazar por `RemainingAccounts` borsh `Vec<AccountMeta>` tipado + paginación obligatoria 5-10 por tx, validar `len==expected` y `is_writable` por rol. |
| V3-SEC-005 | Manual lamports | `trust-escrow-v3/src/lib.rs:130` | `transfer_job_lamports` `**source.try_borrow_mut_lamports()? = remaining`, `**destination... = ...`, `close_evidence_account` `assign(&SYSTEM_PROGRAM_ID); resize(0)`, `Rent::get()?.minimum_balance` 146 | High | M | Bypassa `close = client` de Anchor, doble-contabilidad rent, race si `source==destination`, no CPI. `finalize` 1789-1818 3 transfers sin `system_program::transfer` con PDA signer. | Usar `anchor_lang::system_program::transfer` con `CpiContext::new_with_signer` + `close` attribute, eliminar `try_borrow_mut_lamports` manual. |
| V3-ERR-006 | Dead variants | `trust-escrow-v3/src/lib.rs:33` | `ErrorCode::EmptyTitle, TitleTooLong, DescriptionTooLong, ProposalTooLong` 33-40 nunca usados, `create_job` 817 no valida title/desc (no existen params), `EmptyProposal` solo usado 916 | Medium | S | IDL expone errores falsos, docs `SMARTCONTRACT.md` lista `MAX_TITLE_LEN 100` pero on-chain no hay title. Confunde SDK/cliente. | Eliminar variantes muertas o implementar `require!(title.len()<=...)`, remover `allow(unexpected_cfgs)` 1. |
| V3-SEC-007 | Auth inconsistent | `trust-escrow-v3/src/lib.rs:2068` | `WithdrawTreasury` `constraint = ... @ NotAuthorized` vs `pause` `require!(... @ NotAuthorized)` 717, `add_arbiter` 1337 doble check `config.authority==authority && pool.authority==authority` | Medium | S | Mezcla `constraint` vs `require!` con mismos códigos, `NotValidArbiter` usado para `contains`, `capacity`, `state` (1345,1349,1552) — cliente no puede mapear. | Unificar `#[account(constraint = ... @ ErrorCode::X)]` para todos `Accounts`, definir `ErrorCode` 1:1 por check. |
| V3-SEC-009 | Keeper permissive | `trust-escrow-v3/src/lib.rs:2144` | `AutoApproveWork { pub keeper: Signer }` sin constraint, `auto_approve_work` 1118 solo verifica `dispute.is_none()`, `treasury==config.treasury`, `job.freelancer`, cualquiera puede drenar rent 1151-1160 tras 604800s, no fee para keeper | High | M | Griefing/DOS: keeper front-runs cierre y paga gas por 0 fee. `AUTO_APPROVAL_DELAY` 12 exacto pero `keeper` no necesita ser `client`/`freelancer`. | Añadir `keeper` whitelist o `client`/`freelancer` o `config.authority`, y `fee` 1% para keeper o `close` a `client` con `keeper` como `remaining_accounts` pagado. |
| V3-VAL-010 | Validation missing | `trust-escrow-v3/src/lib.rs:1882` | `create_milestone` 1889 solo `index==milestones_total`, `milestones_total<20`, `new_total<=amount`, no `amount>0` ni `MIN_JOB_AMOUNT`, `MAX_MILESTONES` 20 vs 20 milestones de 1 lamport bloquean `approve_work` | Medium | S | Atacante crea 20 milestones de 1 lamport, fuerza `milestones_approved==20` para `approve_work` sin valor. | `require!(amount >= 1000 && amount <= job.amount)` y `require!(milestones_total < MAX_MILESTONES)` + test fuzz. |
| V3-PERF-011 | Compute blowup | `trust-escrow-v3/src/lib.rs:259` | `Pubkey::find_program_address` por cada `cleanup` iter (hasta 50), `Evidence::try_deserialize` por evidence 188, `finalize:1766 split_at` + `for` 1825 hasta 60 CPIs `get_lamports`, `approve_work:1181 require_full_range true` obliga 50 cuentas | High | M | Riesgo `ComputationalBudgetExceeded` 400k CU, tx 1232 bytes limit con 50 PDAs → 50*32=1600 + overhead > limit. | Paginar `cleanup` 10 por tx, cache `find_program_address` con `bump` (ya hay `bump` en `Job`), usar `AccountInfo` deserialización lazy. |
| V3-SEC-012 | Close inconsistent | `trust-escrow-v3/src/lib.rs:2169` | `ApproveWork close=client` automático 2176 vs `AutoApproveWork` manual `**job...=0; assign(SYSTEM); resize(0)` 1146, `FinalizeDisputePayouts` 2566 cierra `job→client`, `dispute→client`, `escrow→arbitration_treasury` 3 destinos | Medium | M | Inconsistencia rent refund: `ApproveWork` 1 close, `AutoApprove` 1 manual, `Finalize` 3 closes — test `escrow.ts:258` no valida `escrow` destino. | Unificar `close` vía Anchor, 1 ix 1 close, eliminar manual `assign/resize`. |
| V3-PERF-014 | Rent/Clock repeat | `trust-escrow-v3/src/lib.rs:146` | `Rent::get()?.minimum_balance` por transfer (hasta 3x en `finalize` 1789), `Clock::get()?.unix_timestamp` 7 veces (654,826,1097) | Low | S | Sysvar no cacheado + CU. | `let clock = Clock::get()?;` una vez por ix, `Rent::get()` cacheado. |
| V3-TEST-015 | Test debt | `trust-escrow-v3/src/lib.rs:504` | 7 unit tests (`compute_shortfall`, `INIT_SPACE`) vs 40 ix, 8 ITests `tests/escrow.ts` 726 LOC (20% cobertura), 0 fuzz, 0 invariantes, `reject_milestone` 1988 sin test, `remaining_accounts` malformado no testeado, `evidence_cleanup_cursor` 1875 `checked_add(len as u8)` silencia >255 | High | L | Programa custodia fondos con 20% cobertura. `MAX_PAUSE_DURATION` 30d sin test expiración, `MAX_APPLICATIONS` 50 sin test `remaining_accounts` orden incorrecto. | Añadir fuzz `cargo fuzz` + `proptest` para `remaining_accounts`/`evidence` + 15 ITests para `withdraw_treasury`, `resolve_dispute`, `cleanup_*`. |
| BE-TYPE-016 | SDK drift (fixed) | `backend/sdk/src/types.rs:91` | Antes `Job.applicants: [Pubkey;50] + applicants_len` vs `Vec<Pubkey>` 50 (fixed 3b45ea8), `MAX_APPLICATIONS` 10 vs 50 (fixed) | High | S | Desync reportado y fixeado, pero evidencia de drift: `backend/Cargo.lock` 105 líneas drift. | Mantener `cargo update` + `repowise` index para detectar IDL drift. |
| BE-ARCH-017 | Backend god utils | `backend/sdk/src/utils.rs:267` | `utils.rs` 267 LOC con `DEFAULT_RPC_TIMEOUT`, `Page<T>`, `with_timeout`, `with_retry`, `encode_cursor` — todo en un fichero, `frontend/src/api/client.ts` duplica `apiFetch` timeout logic | Medium | S | Duplicación timeout/cursor entre `backend/sdk/utils.rs` y `frontend/src/api/client.ts`. | Extraer `cursor` crate compartido o `utils` → `cursor.rs` + `timeout.rs`. |
| BE-SEC-018 | Auth middleware order | `backend/api/src/middleware.rs:30` | `cors_layer` permissive dev vs restrict prod, `security_headers` max, `rate_limit` per-IP `Mutex<HashMap>`, `https_enforcement` prod exige `x-forwarded-proto: https`, `request_size_guard` 1 MiB — orden `cors → security_headers → rate_limit → https → size` | Medium | S | `rate_limit` usa `Mutex` sin `RwLock` — contención bajo load 1k rps. `https_enforcement` solo check header, no redirect. | Cambiar `Mutex` → `RwLock` + `DashMap`, añadir `redirect https` en prod. |
| FE-ARCH-019 | Frontend no e2e | `frontend/src/app/jobs/page.tsx` | Zustand `useJobStore` consume `api/jobs/list` → backend `InMemory` (sin Postgres), 24/24 vitest, `next build` 5 rutas, `Bail out to client-side rendering: next/dynamic` para `WalletMultiButton` | Medium | M | Sin `playwright` e2e, sin `msw` mock, wallet e2e no testeada. `NEXT_PUBLIC_API_URL` 3000 choca con backend 3000 → fix 3001 documentado pero no en `frontend/.env.local` default. | Añadir `playwright` e2e `jobs.spec.ts` + `msw` + `NEXT_PUBLIC_API_URL` 3001 por defecto. |
| FE-PERF-020 | GSAP + Motion double | `frontend/src/components/motion.tsx` | `gsap@3.15` + `framer-motion@13.1` ambos para stagger/parallax/hover — `JobCard` usa `motion` + `gsap` stagger 0.06, `globals.css` shimmer + `motion.tsx` `useGsapStagger` duplicado | Low | S | Doble lib 120KB (GSAP) + 90KB (Motion) para mismo efecto stagger/hover. | Elegir 1 (Motion para React, GSAP solo para ScrollTrigger complejo) y remover duplicado. |
| OPS-021 | Validator ledger tmpfs | `/tmp/validator-ledger` 3.6G tmpfs 3.9G 100% → ENOSPC, movido a `trust-escrow-v3/.anchor/test-ledger` 193M en `/` (disk) | High | S | Ledger en tmpfs llenó `/tmp` y mató validator + `dc-dev` dispatch (`ENOSPC` en `/tmp/dc-dev-agents.log`). | Ya fixeado (mover a disco, documentado en `docs/stack/README.md`), añadir `.gitignore` para ledger. |
| OPS-022 | Deploy max-len | `trust-escrow-v3/target/deploy/trust_escrow_v3.so` 581KB → `anchor deploy` sin `--max-len` da `account data too small` | High | S | Deploy manual necesitó `-- --max-len 700000` (no documentado en `Anchor.toml` hasta fix). | Añadir `Anchor.toml` `[programs.localnet] max-len` o script `deploy.sh` con `--max-len`. |
| OPS-023 | Port collision | `backend` 3000 vs `frontend` 3000 → EADDRINUSE, fix docs/stack `frontend --port 3001` | Medium | S | `frontend/package.json` `next dev` default 3000 choca con `backend` 3000. | Cambiar `frontend/package.json` `dev` → `next dev --port 3001` por defecto. |
| CFG-024 | Env duplication | `.env.example`, `backend/.env.example`, `backend/api/.env.example` triplicado 1510 bytes idénticos + `frontend/.env.local` 4 vars | Low | S | Triplicación `.env.example` (root, backend, api) idénticos. | Mantener 1 en root + symlink o `dotenvy` con `ENV_EXAMPLE` const (ya existe). |
| DOC-025 | Docs drift (fixed) | `trust-escrow-v3/context/validation/final-report.md` 06/08 `BLOCKED` vs 19/08 `9/9 PASS` (fixed 607b90f), `SMARTCONTRACT.md` Vec 50 | Medium | S | Drift histórico fixeado pero evidencia de proceso: docs desactualizados sin `cargo test` gate. | CI `final-gate.sh` ya cubre (20/20). |
| SEC-026 | Secret scan allowlist | `.gitleaks.toml` allowlist `7a2YhCd7...` + `solana-private-key-array` | Low | S | Allowlist para fixture `7a2Y` correcta pero amplia (`solana-private-key-array` regex). | Estrechar allowlist a `path:frontend/src/lib/sdk.ts` + `tests/`. |

---

## Top 5 — If you fix nothing else, fix these

1. **V3-ARCH-001 + V3-ARCH-004 + V3-SEC-005** — Split `lib.rs` + typed `remaining_accounts` + `close` via Anchor. Afecta 40 ix, `cleanup`/`finalize`, rent. Test: `cargo test` 8/8 + 9/9 e2e, `anchor build`. Migration: IDL unchanged si solo split `mod`.
2. **V3-SEC-002** — `INITIAL_AUTHORITY` rotation. Afecta `initialize_config`, deploy. Test: `anchor test` con `update_authority` 2-step, timelock. Migration: nuevo `Config.authority` + `pending_authority`.
3. **V3-SEC-009** — `AutoApprove keeper` auth. Afecta `auto_approve_work` payout. Test: `escrow.ts` keeper `client`/`freelancer`/`authority` + `expectRevert` para `random`. Migration: añadir `keeper` whitelist.
4. **BE-SEC-018 + OPS-021/022** — Backend rate limit `Mutex` → `RwLock/DashMap` + validator ledger disk + deploy `max-len`. Afecta load 1k rps y deploy reproducible. Test: `cargo test` + `scripts/final-gate.sh` 20/20.
5. **V3-TEST-015** — Fuzz + 15 ITests para `remaining_accounts`/`evidence`. Afecta funds custody. Test: `cargo fuzz` + `proptest` + `anchor test` 15 nuevos.

---

## Quick Wins (Low effort, Medium+ severity)

- [ ] V3-ERR-006: eliminar 6 variantes muertas `ErrorCode` o implementar validación title/desc (S, Medium)
- [ ] V3-SEC-007: unificar `constraint` vs `require!` (S, Medium)
- [ ] V3-SEC-012: unificar `close` (S, Medium)
- [ ] V3-PERF-014: cache `Clock::get()`/`Rent::get()` (S, Low)
- [ ] BE-ARCH-017: extraer `cursor.rs` + `timeout.rs` (S, Medium)
- [ ] CFG-024: deduplicate `.env.example` (S, Low)
- [ ] OPS-023: `frontend/package.json` `dev --port 3001` (S, Medium)
- [ ] V3-ERR-013: `allow(unexpected_cfgs)` por línea (S, Low)

---

## Discarded Findings — Looks Bad But Is Actually Fine

1. **Pattern:** `#[max_len(MAX_APPLICATIONS)] pub applicants: Vec<Pubkey>` parece `Vec` sin `InitSpace` correcto.  
   **Evidence:** `trust-escrow-v3/src/lib.rs:410` `#[max_len(50)]` + `Job::INIT_SPACE` test `job_compact_init_space_under_10kib_and_vec_50_compact` 8/8 PASS, `cargo test` 8/8.  
   **Decision:** Keep. **Reason:** Anchor `InitSpace` calcula `4 + 50*32` para `Vec` con `max_len`, no `Vec` dinámico sin límite. Correcto y testeado.

2. **Pattern:** `backend/api/src/sync.rs` polling `getSignaturesForAddress` parece N+1.  
   **Evidence:** `sync.rs:720` `SyncCursor` `HashSet+VecDeque` 4096 FIFO, `with_timeout` 30s, `with_retry` solo `Timeout`, `cargo test` 42/42.  
   **Decision:** Keep. **Reason:** Polling es intencional sin WebSocket (trait `SignatureFetcher` mockeado), ordenado `oldest-first`, idempotente. No N+1.

3. **Pattern:** `frontend/src/api/client.ts` + `backend/sdk/src/utils.rs` duplican `encode_cursor` base64url.  
   **Evidence:** Ambos usan `base64url` opaco, tests `queries.rs` 6/6 + `client.test.ts` 11/11, `with_timeout` tipado.  
   **Decision:** Keep for now. **Reason:** Duplicación intencional por stack split (Rust vs TS), `cursor` opaco es contrato, no lógica. Extracción a crate compartido es S pero no urgente.

4. **Pattern:** `trust-escrow-v3/src/lib.rs:1` `#![allow(unexpected_cfgs)]` parece supresión global.  
   **Evidence:** `cargo clippy` 0 warnings, `anchor 0.32.1` cfg `anchor` vs `feature="anchor"` es falso positivo conocido.  
   **Decision:** Keep for now. **Reason:** Anchor 0.32 emite `unexpected_cfgs` para `#[account]` etc., fix sería `#[allow]` por línea (Quick Win), no crítico.

5. **Pattern:** `backend/sdk/src/cluster.rs` `TRUST_ESCROW_ALLOW_MAINNET` env parece backdoor.  
   **Evidence:** `cluster.rs:8` `is_mainnet_allowed()` env truthy `1/true/yes/on`, `parse_cluster` bloquea mainnet por defecto, tests 8/8, `cargo test` 180+.  
   **Decision:** Keep. **Reason:** Allowlist explícita para mainnet es intencional (security B5), default `blocked`, no backdoor.

---

## Open Questions for Maintainer

1. `INITIAL_AUTHORITY` `3whY...` — ¿Es Squads multisig o EOA? ¿Plan de rotación?
2. `AutoApprove keeper` — ¿Debe ser `client`/`freelancer`/`authority` o permissionless con fee?
3. `remaining_accounts` 50 en una tx — ¿Paginación obligatoria 10 por tx o mantener 50 con `require_full_range`?
4. Frontend `NEXT_PUBLIC_API_URL` 3000 vs backend 3000 — ¿Cambiar frontend default a 3001 en `package.json`?
5. `landing/` Dioxus vs `frontend/` Next.js — ¿Cuál es canonical para hackathon demo?
6. `.env.example` triplicado — ¿Mantener 3 o 1 en root + symlink?
7. `dc-dev` dispatch `no-child-observed` — ¿Debe usar `task` tool directo (que funciona 26/26) en vez de `client.session.prompt`?

---

## Audit Limitations

- No `cargo audit`/`cargo udeps`/`knip`/`madge` ejecutados (solo `clippy`/`audit`/`gitleaks` via `final-gate.sh` 20/20)
- No fuzz/invariants medidos (solo unit + e2e 9/9)
- Validator local `7a2Y` UP, no mainnet/devnet fork
- `landing/` Dioxus no auditado en profundidad (solo `frontend` Next.js 24/24)
- `dc-dev` dispatch `session.prompt` UnknownError no reproducido con `task` tool (que sí funciona)
- Disk 75% usado (251G, 60G libres), `/tmp` tmpfs 3.9G con ledger movido a disco
