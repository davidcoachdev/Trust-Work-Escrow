# Trust Work Escrow — Backend

Rust workspace for the Trust Work Escrow v3 backend. It contains two crates:

- `trust-escrow-sdk` — Anchor/Solana SDK to read on-chain accounts and build
  program instructions.
- `trust-escrow-api` — axum-based REST API with OpenAPI/Swagger docs.

## Architecture

The on-chain program only stores functional data (pubkeys, amounts, status,
hashes). All descriptive metadata (titles, descriptions, proposals, evidence
content, resolutions) is stored off-chain:

- **PostgreSQL** — structured relational data (jobs, applications, milestones,
  disputes, support tickets, arbiter pool).
- **MongoDB** — large or flexible content (evidence attachments, chat/activity
  logs).

The DB layer is intentionally not wired yet; endpoint handlers return
`501 Not Implemented` until the Docker-backed services are available.

## Project layout

```text
backend/
├── Cargo.toml           # workspace definition
├── sdk/                 # on-chain types + instruction builders
│   ├── src/
│   │   ├── lib.rs
│   │   ├── types.rs     # account structs + AccountDeserialize impls
│   │   ├── pda.rs       # PDA derivation helpers
│   │   ├── client.rs    # TrustEscrowClient + instruction wrappers
│   │   ├── events.rs
│   │   ├── error.rs
│   │   └── utils.rs
│   └── Cargo.toml
├── api/                 # REST API
│   ├── src/
│   │   ├── main.rs      # axum app + OpenAPI doc
│   │   ├── routes.rs    # endpoint handlers (stubs)
│   │   ├── models.rs    # request/response DTOs
│   │   └── state.rs     # AppState (empty until DB is wired)
│   ├── Cargo.toml
│   └── Dockerfile
└── README.md
```

## Quick start

### Build

```bash
cargo build --manifest-path backend/Cargo.toml
```

### Run tests

```bash
cargo test --manifest-path backend/Cargo.toml
```

### Run the API locally

```bash
cargo run --manifest-path backend/Cargo.toml -p trust-escrow-api
```

Then open:

- Swagger UI: <http://localhost:3000/swagger-ui>
- OpenAPI JSON: <http://localhost:3000/api-docs/openapi.json>
- Health check: <http://localhost:3000/health>

### Run with Docker Compose

When Docker is available, the root `docker-compose.yml` starts Postgres, MongoDB
and the API:

```bash
docker compose up --build
```

## SDK usage

Enable the `solana` feature to get Anchor/Solana dependencies and PDA helpers:

```toml
[dependencies]
trust-escrow-sdk = { path = "../sdk", features = ["solana"] }
```

```rust
use trust_escrow_sdk::{Cluster, TrustEscrowClient};

let client = TrustEscrowClient::from_keypair_path(Cluster::Localnet, "path/to/id.json")?;
let job = client.get_job(&client.pubkey(), 1)?;
```

## Applications PDA individual — v3 vigente (T21-T26)

**Contrato:** `trust-escrow-v3` `7a2YhCd7iivXfyySkp1pf5jjijGqpjNqwQCUS912q5Vh` (Anchor 0.32.1) — `Job` compacto `Vec<Pubkey>` 50, no inline. `MAX_APPLICATIONS = 50` en `lib.rs` y `sdk/src/types.rs`.

| Cuenta | Seeds | Bump | Owner | Campos clave |
|---|---|---|---|---|
| `Job` | `[b"job", client, job_id.le_bytes()]` | `job.bump` (u8) | `7a2Y...` | `applicants: Vec<Pubkey>` `#[max_len(50)]`, `bump`, estados `Created→Funded→InProgress…` |
| `Application` | `[b"application", job, &[index], applicant]` | `application.bump` (u8) | `7a2Y...` | `job, index: u8 (0..49), applicant, proposal_hash: [u8;32], status: Pending/Accepted/Rejected/Withdrawn, bump` — **PDA individual por (job, index, applicant)**, off-curve |

**IDL:** `trust-escrow-v3/target/idl/escrow.json` (`address 7a2Y...`, `types.Application`/`Job`, `ApplicationStatus`). Validado vs código en `sdk/tests/t26_idl_docs.rs` (seeds, bump, ownership, args/cuentas, MAX 50, no inline, límites, unicidad, cleanup).

**Instrucciones Applications:**

| Ix | Args | Cuentas | Validaciones clave |
|---|---|---|---|
| `apply_to_job` | `_job_id: u64, application_index: u8, proposal_hash: [u8;32]` | `applicant (Signer, payer)`, `job (mut PDA job)`, `application (init, PDA application)`, `client (Unchecked, PDA job)`, `system_program` | `status==Funded`, `applicant != client`, `!AlreadyApplied`, `len<50`, `index==len`, `hash != [0;32]` (`EmptyProposal`) |
| `accept_application` | `_job_id, application_index` | `client (Signer)`, `job (mut)`, `applicant (SystemAccount)`, `application (mut PDA)` | `Pending`, `index` y `job.applicants[index]==applicant`, `freelancer None`, asigna `job.freelancer`, `status Accepted→InProgress` |
| `reject_application` | `_job_id, application_index` | idem + `application close=applicant` | `Pending`, rent refund al postulante |
| `withdraw_application` | `_job_id, application_index` | `applicant (Signer)`, `job`, `application close=applicant` | `Pending`, solo postulante, rent refund |
| `cleanup_applications` | `_job_id, start_index: u8` | `client (Signer)`, `job (mut)`, `remaining_accounts: [application, applicant]*N` | `InProgress/Submitted/Disputed` + `freelancer Some`, batch `start_index..`, valida PDA y `job.applicants[index]==applicant`, cierra `Pending/Rejected/Withdrawn` con rent al `applicant`, retiene `Accepted`/`closed(allow_closed)` |

**Límites texto:** off-chain `proposal 1..512 chars` (`validation.rs`/`metadata.rs` `ProposalTooLong`/`EmptyProposal`), on-chain `proposal_hash [u8;32]` SHA256, rechazo `EmptyProposal` si `hash==[0;32]`. Hash determinista 32 bytes.

**Unicidad:** `AlreadyApplied` (aunque cambie índice), `CannotWorkOnOwnJob`, `ApplicationIndexMismatch`/`InvalidApplicationIndex` (`0..49`, `index==len`).

**Cleanup/rent:** `close = applicant` en `Reject/Withdraw`; `cleanup_applications` batch vía `remaining_accounts` con validación `InvalidApplicationCleanupAccounts`, rent de cada `Application` no-accepted transferido al `applicant` (`assign SYSTEM_PROGRAM_ID, resize 0`).

**Sin modelo inline:** `Job` no contiene `Vec<Application>` ni `[Application;50]`; solo `Vec<Pubkey>` compacto (`Job::INIT_SPACE <10 KiB`, delta `50*32`). IDL `Job.applicants: vec pubkey` lo prueba. Tests `job_compact` + `t26_idl_docs` blindan.

## API endpoints (implemented as stubs)

| Resource | Endpoint | Status |
|---|---|---|
| Health | `GET /health` | ✅ implemented |
| Config | `GET /config` | stub |
| Jobs | `GET /jobs`, `POST /jobs`, `GET /jobs/{id}` | stub |
| Funding | `POST /jobs/{id}/deposit` | stub |
| Applications | `POST /jobs/{id}/apply`, `POST /jobs/{id}/applications/{index}/accept`, `/reject`, `/withdraw`, `/cleanup` (T21-T26) | stub (SDK wrappers + PDA helpers + validation T21-T25 verdes) |
| Work | `POST /jobs/{id}/submit-work`, `/approve-work`, `/reject-work` | stub |
| Job lifecycle | `POST /jobs/{id}/cancel`, `/pause`, `/unpause` | stub |
| Milestones | `POST /jobs/{id}/milestones`, `/submit`, `/approve`, `/reject` | stub |
| Disputes | `POST /jobs/{id}/disputes/*` | stub |
| Support | `POST /jobs/{id}/support`, `/resolve` | stub |
| Arbiter pool | `GET/POST /arbiter-pool`, `/arbiters` | stub |

## Final Gate T20-T26 — validator + CI + coverage + IDL/docs Applications

Gate final reproducible que valida el workspace completo contra el plan `context/plans/backend-v3-map.md` (21 requirements + 6 security gates) y el modelo Applications PDA individual T21-T26 (IDL, seeds, MAX 50, límites, unicidad, cleanup/rent, no inline).

```bash
# Gate local estricto — requiere validator 7a2Y UP en http://127.0.0.1:8899
./scripts/final-gate.sh

# Modo CI — validator warn (no bloquea si no hay solana-test-validator)
./scripts/final-gate.sh --ci

# Salida JSON para automatización
./scripts/final-gate.sh --json

# Comandos individuales (equivalentes a CI)
cargo test --manifest-path backend/Cargo.toml          # 164 passed / 0 failed (umbral T20: ≥149)
cargo clippy --manifest-path backend/Cargo.toml -- -D warnings
cargo fmt --manifest-path backend/Cargo.toml --all -- --check
```

**CI:** `.github/workflows/ci.yml` (`CI — Backend v3 Final Gate (T20)`) reproduce el mismo gate en GitHub Actions (clippy + fmt + test + secret-scan + 0600 + `./scripts/final-gate.sh --ci`).

**Validator 7a2Y UP:**

```bash
curl -s -X POST -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"getHealth"}' \
  http://127.0.0.1:8899
# → {"jsonrpc":"2.0","result":"ok","id":1}

# Program id inmutable declarado en trust-escrow-v3/Anchor.toml:
# trust_escrow_v3 = "7a2YhCd7iivXfyySkp1pf5jjijGqpjNqwQCUS912q5Vh"
```

**Coverage docs:** `docs/BACKEND_COVERAGE.md` — matrix 21/21 + 6/6, conteos por crate/módulo, reproducibilidad.

**Security:** el gate bloquea `mainnet` en `SOLANA_RPC_URL` (`sdk/src/cluster.rs` allowlist + `final-gate.sh` guard + `ci.yml` guard). Nunca envía a mainnet.

## Next steps

1. Wire Postgres/Mongo repositories into `AppState`.
2. Implement service layer: hash proposals/evidence, store metadata off-chain,
   build Solana transactions via `trust-escrow-sdk`.
3. Replace stub handlers with real business logic.
4. Add authentication/authorization middleware.
5. Add integration tests against a local Solana validator.
