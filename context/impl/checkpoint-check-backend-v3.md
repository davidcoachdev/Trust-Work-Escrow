# Checkpoint Review — Trust-Work-Escrow v3 Backend

**Date:** 2026-08-19
**Reviewer:** cavekit-check (hy3)
**Scope:** Backend v3 workspace (`backend/`) — SDK Rust (Axum REST API pending)
**Program:** `J1c4QsjbV9bFEPrFQZZGe8GrGWFxNhtAhhrxJFK2xc1h` (Anchor 0.32.1)

---

## 1. Summary of implemented waves

The workspace (T1) is fully scaffolded and the default build is clean. Waves 1–2 are the
real focus: T2 (nine PDA derivation + thread-safe cache) is implemented and its test file
(`tests/pda.rs`) **compiles and is correct**; T3 (configurable client, nine typed getters,
`ErrorCode` mirror) is present in code but its test (`tests/core.rs`) does **not** compile;
T4/T5 (all 39 instruction wrappers — 38 on-chain handlers + `check_not_paused` guard) exist as
code in `client.rs`. Everything from T7 onward (queries, events/listener, off-chain metadata,
sync, auth/validation middleware, config loader, security/secret hardening, CI gate) is **not
started**. Critically, the entire SDK logic lives behind `#![cfg(feature = "solana")]`, so the
three standard verification commands (`cargo check`/`test`/`clippy`) compile and pass by
ignoring ~all of it — producing a **false green**. The SDK integration tests, the only thing
that verifies the security-critical signing/wrapper layer, do **not compile** under
`--features solana`, and the `client.rs` instruction signatures appear **out of sync with the
contract** (`create_job` missing `title`/`description`, `apply_to_job` expecting `[u8;32]`
instead of a proposal string). This is a P0 gate failure.

---

## 2. Task-by-task status (T1–T20)

| Task | Title | Status | Evidence |
|---|---|---|---|
| T1 | Workspace + crate skeletons | **COMPLETE** | `cargo check` workspace builds clean; `backend/Cargo.toml` workspace with `sdk`+`api`. |
| T2 | 9 PDA derivation + cache | **COMPLETE** | 9 `derive_*` + 9 `get_*` fns in `sdk/src/pda.rs`; `tests/pda.rs` compiles under `--features solana` (no errors) and covers vectors, determinism, cache hit <1ms. |
| T3 | Client, getters, typed errors | **PARTIAL** | `client.rs` (`TrustEscrowClient::new`/`from_env`/9 getters), `error.rs` (`ErrorCode` 6000–6050) present. `tests/core.rs` **fails to compile** (Job type missing `title`/`description`/`created_at`/`updated_at`). `error.rs` unit tests pass (default). |
| T4 | Wrappers config/jobs/apps/work | **COMPLETE** (code) | 21 entries present in `client.rs`. Integration test `tests/instructions_jobs.rs` **does not compile** (10 errors). |
| T5 | Wrappers arbitration/disputes/support/milestones | **COMPLETE** (code) | 18 entries present in `client.rs` (38 handlers + guard = 39). Planned test files `tests/instructions_disputes.rs` / `tests/instructions_milestones.rs` **do not exist**. |
| T6 | Integrated wrapper + error verification | **NOT_STARTED** | `cargo test -p trust-escrow-sdk --features solana` fails to compile; no passing integration run. |
| T7 | Listings read-through + cursor + timeouts | **NOT_STARTED** | No `list_jobs_by_client`/`list_jobs_by_status` in `client.rs`; `utils.rs` is a `todo!()` stub. |
| T8 | `list_applications(job)` | **NOT_STARTED** | Not implemented in `client.rs`. |
| T9 | Event listener / fallback | **NOT_STARTED** | `sdk/src/events.rs` is a stub (`pub struct EscrowEvent;`). |
| T10 | Off-chain metadata model/repo | **NOT_STARTED** | No `api/src/metadata.rs` / `repository.rs`. |
| T11 | Evidence off-chain hash + index | **NOT_STARTED** | Not implemented. |
| T12 | Listener → repo sync | **NOT_STARTED** | No `api/src/sync.rs`. |
| T13 | axum runtime, state, errors, health, metrics | **PARTIAL** | Router + `/health` + Swagger exist (`main.rs`/`routes.rs`); `health` returns 200 unconditionally (no RPC check); no dedicated `error.rs`; no metrics; `state.rs` empty. |
| T14 | Jobs/actions endpoints 1:1 | **PARTIAL** | 31 routes defined in `routes.rs` returning `501 Not Implemented` (no SDK delegation). |
| T15 | Input validation, signature auth, security middleware | **NOT_STARTED** | No `api/src/middleware.rs`; no validation/auth in handlers. |
| T16 | API + SDK + metadata integration | **NOT_STARTED** | No `api/tests/integration.rs`. |
| T17 | Config loader + `.env.example` | **NOT_STARTED** | No `backend/config/` crate, no `.env.example`, no `.gitignore` additions. |
| T18 | Cluster switch, mainnet block, secure keypair | **NOT_STARTED** | `parse_cluster` allows `Mainnet` with no guard; no CI/mainnet rejection. |
| T19 | Secure logging, 0600 perms, secret scan | **NOT_STARTED** | Only basic `tracing` in `main.rs`; no redaction/permissions/scan. |
| T20 | Final gate: validator + CI + coverage | **NOT_STARTED** | No Makefile/scripts/CI. |

---

## 3. Critical gaps (severity → requirement)

**P0 — SDK integration test suite does not compile (blocks R4, R5, R17 verification)**
- `cargo test --features solana --no-run` → `could not compile trust-escrow-sdk (test "instructions_jobs") due to 10 previous errors` and `could not compile ... (test "core") due to 4 previous errors`.
- Consequence: the security-critical signing/wrapper layer has **zero** executed verification. The standard `cargo check/test/clippy` commands pass only because all of it is behind `cfg(feature = "solana")`.

**P0 — Probable SDK↔contract signature mismatch (R5 FR-5 correctness)**
- `client.rs::create_job(&self, job_id, amount, deadline)` — test expects `(job_id, title, description, amount, deadline)` → 5 call sites fail (lines 145, 173, 244, 279, 302).
- `client.rs::apply_to_job(..., proposal_hash: [u8;32])` — test passes `&str` proposal → type mismatch (lines 194, 198, 249, 284).
- `client.rs::reject_work(&self, job_id)` — test passes a reason string (line 293).
- If the contract's v3 `create_job`/`apply_to_job` truly take title/description/proposal on-chain, the SDK is emitting **wrong instructions** and real transactions will fail (or worse, malformed). Must be reconciled against the actual v3 IDL/source.

**P1 — SDK `Job` type missing fields the test deserializes (R3 FR-3)**
- `tests/core.rs` builds `Job { title, description, created_at, updated_at, ... }`; `types.rs::Job` has none of these four fields → 4× `E0560`. Resolve: either (a) add the on-chain fields to `Job` (if v3 stores them), or (b) fix the test to mirror the off-chain metadata split (B6). Either way T3 verification is currently impossible.

**P1 — Process gap: required commands give a false green (Security B1)**
- Tasks 1–3 in the brief run `cargo check/test/clippy` **without** `--features solana`, so they never compile the SDK logic or tests. A reviewer reading only those outputs would wrongly conclude T2–T6 are green.

**P2 — `health` does not reflect RPC state (R13 FR-13)**
- `main.rs::health()` returns `200 ok` unconditionally; done criteria require it to reflect RPC/state availability.

**P2 — Mainnet not blocked (R17 FR-17, Security B5)**
- `client.rs::parse_cluster` maps `mainnet`/`mainnet-beta` → `Cluster::Mainnet` with no allowlist guard; no CI rejection of mainnet URLs.

---

## 4. Concrete next task to implement

**Fix the SDK↔test contract alignment and make the integration suite compile** (prerequisite
for unblocking T3/T6). Concretely, in priority order:
1. Reconcile `client.rs` instruction signatures (`create_job`, `apply_to_job`, `reject_work`)
   and `types.rs::Job` fields against the real v3 contract IDL — adjust the client **or** the
   test so they match on-chain reality (this also closes the P0 correctness risk).
2. Make `cargo test -p trust-escrow-sdk --features solana` compile and run `tests/pda.rs`,
   `tests/core.rs`, and `tests/instructions_jobs.rs` green (T2–T6).
3. Add the missing T5 test files (`instructions_disputes.rs`, `instructions_milestones.rs`).
4. Re-run the three standard commands **with `--features solana`** so the checkpoint is honest.

Until step 1–2 pass, do **not** advance to T7+ (queries/events/metadata/auth).

---

## 5. Verdict

Verdict: REJECT
