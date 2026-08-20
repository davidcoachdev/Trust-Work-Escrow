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

## API endpoints (implemented as stubs)

| Resource | Endpoint | Status |
|---|---|---|
| Health | `GET /health` | ✅ implemented |
| Config | `GET /config` | stub |
| Jobs | `GET /jobs`, `POST /jobs`, `GET /jobs/{id}` | stub |
| Funding | `POST /jobs/{id}/deposit` | stub |
| Applications | `POST /jobs/{id}/apply`, `POST /jobs/{id}/applications/{index}/accept` | stub |
| Work | `POST /jobs/{id}/submit-work`, `/approve-work`, `/reject-work` | stub |
| Job lifecycle | `POST /jobs/{id}/cancel`, `/pause`, `/unpause` | stub |
| Milestones | `POST /jobs/{id}/milestones`, `/submit`, `/approve`, `/reject` | stub |
| Disputes | `POST /jobs/{id}/disputes/*` | stub |
| Support | `POST /jobs/{id}/support`, `/resolve` | stub |
| Arbiter pool | `GET/POST /arbiter-pool`, `/arbiters` | stub |

## Final Gate T20 — validator + CI + coverage

Gate final reproducible que valida el workspace completo contra el plan `context/plans/backend-v3-map.md` (21 requirements + 6 security gates).

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
