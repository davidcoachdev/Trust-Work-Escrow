# Stack Local — Trust Work Escrow v3

Guía para levantar **todo el stack** en local: validator Solana, DBs (Postgres + Mongo), backend Axum (SDK 7a2Y), frontend Next.js (tema dcdev) y landing Dioxus.

> **Programa:** `7a2YhCd7iivXfyySkp1pf5jjijGqpjNqwQCUS912q5Vh` (deploy slot 26, 594KB, authority `3whY1ohd...`)  
> **Validator:** Agave 4.1.1, `http://127.0.0.1:8899`  
> **Backend:** `http://127.0.0.1:3000` (Swagger `/swagger-ui`)  
> **Frontend dApp:** `http://localhost:3001` (Next 16, Turbopack, dashboard freelancer/publisher 34/34)  
> **Landing:** Dioxus `landing/` (opcional)

---

## Requisitos

- Rust 1.89 + `cargo`, `anchor 0.32.1`, `solana 4.1.1`
- Node `bun` (frontend) o `npm`/`yarn`
- Docker (para Postgres 16 + Mongo 7) — ver *Docker en WSL2* abajo

---

## 1. Validator Solana (obligatorio)

> **Importante:** ledger en `trust-escrow-v3/.anchor/test-ledger` (disco, no `/tmp` tmpfs) y deploy con `--max-len 700000` (sin eso da `account data too small`).

```bash
# desde la raíz del repo
rm -rf trust-escrow-v3/.anchor/test-ledger
solana-test-validator --reset --ledger trust-escrow-v3/.anchor/test-ledger &
# espera health ok
curl -s http://127.0.0.1:8899 -X POST -H "Content-Type: application/json" -d '{"jsonrpc":"2.0","id":1,"method":"getHealth"}'
# → {"result":"ok"}

# build + deploy
yarn --cwd trust-escrow-v3 build
# si el programa ya existe, usa max-len:
cd trust-escrow-v3
ANCHOR_PROVIDER_URL=http://127.0.0.1:8899 anchor deploy --provider.cluster localnet -- --max-len 700000
# verifica
solana program show 7a2YhCd7iivXfyySkp1pf5jjijGqpjNqwQCUS912q5Vh --url http://127.0.0.1:8899
```

**Troubleshooting validator:**
- `Persistent Config exists ... advisor signer is unavailable` → `rm -rf trust-escrow-v3/.anchor/test-ledger` y redeploy limpio
- `/tmp` lleno (tmpfs 3.9G) → mover ledger a `.anchor/test-ledger` (disco, 60G libres)
- `SBPFv3 inactive` ya no ocurre en Agave 4.1.1 con ledger limpio

---

## 2. DBs (Postgres + Mongo) — Docker

`docker-compose.yml` en la raíz define:

```yaml
postgres: 16-alpine  → 127.0.0.1:5432  (twe-postgres, user twe / db trust_work_escrow)
mongo: 7             → 127.0.0.1:27017 (twe-mongo, user twe)
```

**Opción A — docker compose (si el plugin está instalado):**
```bash
docker compose up -d
# o
docker-compose up -d
```

**Opción B — docker run directo (sin compose, funciona en WSL sin plugin):**
```bash
docker pull postgres:16-alpine
docker run -d --name twe-postgres -e POSTGRES_USER=twe -e POSTGRES_PASSWORD=twe -e POSTGRES_DB=trust_work_escrow -p 5432:5432 postgres:16-alpine

docker pull mongo:7
docker run -d --name twe-mongo -e MONGO_INITDB_ROOT_USERNAME=twe -e MONGO_INITDB_ROOT_PASSWORD=twe -p 27017:27017 mongo:7

docker ps  # debe mostrar twe-postgres y twe-mongo UP
pg_isready -h 127.0.0.1 -p 5432 -U twe  # → accepting connections
```

**Docker en WSL2:**
- Si `docker ps` da `failed to connect to npipe:////./pipe/dockerDesktopLinuxEngine`, activa **WSL Integration** en Docker Desktop → Settings → Resources → WSL Integration → activa tu distro Ubuntu, o usa `docker run` directo (no necesita compose plugin).

**Sin Docker (dev):** el backend usa `InMemoryMetadataRepository` por defecto — `cargo test` y `cargo run` funcionan sin DB (tests 152/152). Postgres/Mongo solo se necesitan para persistencia real.

---

## 3. Backend Axum (SDK 7a2Y)

Env en `.env`, `backend/.env`, `backend/api/.env` (ver `.env.example`):

```bash
# .env.example ya existe en las tres rutas, con:
PORT=3000
RPC_URL=http://127.0.0.1:8899
DATABASE_URL=postgres://twe:twe@127.0.0.1:5432/trust_work_escrow
MONGO_URL=mongodb://twe:twe@127.0.0.1:27017/trust_work_escrow
# + RUST_LOG, CORS, etc.
```

Levantar:

```bash
# desde la raíz
cargo run --manifest-path backend/Cargo.toml -p trust-escrow-api
# → Trust Escrow API listening version=3.0.0 rpc_url=http://127.0.0.1:8899 port=3000
# → Swagger UI at http://0.0.0.0:3000/swagger-ui

# health
curl -s http://127.0.0.1:3000/health | jq
# → {"status":"ok","version":"3.0.0","checks":{"repository":"ok","rpc":"ok"}}

# gates
cargo test --manifest-path backend/Cargo.toml --features solana  # 164+ tests
cargo clippy --manifest-path backend/Cargo.toml --features solana -- -D warnings
./scripts/final-gate.sh  # 20/20 PASS
```

---

## 4. Frontend dApp (Next.js 16) — tema dcdev

Stack: Next 16.3.1 (Turbopack) + React 19 + Zustand 5 + `frontend/src/api/` (por método/endpoint) + `frontend/src/stores/` + wallet adapter + GSAP 3.15 + Framer Motion 13.1

Tema `dcdev` crimson: `bg #120808`, `surface #1E0E0E`, `primary #FF3C3C`, `gradient linear-gradient(135deg,#FF3C3C,#781414)`, `Inter`, gaps 8pt

Env `frontend/.env.local` (ya creado):

```
NEXT_PUBLIC_RPC_URL=http://127.0.0.1:8899
NEXT_PUBLIC_API_URL=http://127.0.0.1:3000
NEXT_PUBLIC_PROGRAM_ID=7a2YhCd7iivXfyySkp1pf5jjijGqpjNqwQCUS912q5Vh
```

Levantar (puerto 3001 para no chocar con backend 3000):

```bash
cd frontend
bun install
bun run dev --port 3001
# o npm run dev -- --port 3001
# → http://localhost:3001
# rutas: / , /jobs , /create , /jobs/[id] , /dashboard , /dashboard/freelancer , /dashboard/client
```

Verificación:

```bash
bun run --cwd frontend test    # 34 passed (api/client + stores + dashboard) — tras dashboard freelancer/publisher
bun run --cwd frontend build   # 12 rutas (5 base + 7 dashboard) — tema dcdev crimson + GSAP + Motion
```

Estructura `api/` (Zustand fuente de verdad):

```
frontend/src/api/
  client.ts          # root apiFetch + ApiError (timeout, 404)
  types.ts           # Job (on Vec + off title/description) → JobResponse, mapJobResponse
  jobs/{list,get,create,deposit,cancel,pause,unpause,submitWork,approveWork,rejectWork}
  applications/{apply,accept}
  milestones/{create,submit,approve,reject}
  disputes/{raise,accept,evidence,assignArbiter,resolve,platformResolve,requestIntervention,finalize}
  support/{open,resolve}
  arbiterPool/{get,create,add,remove}
  config/get.ts, auth/verify.ts, health/check.ts
  index.ts           # barrel re-exporta client + types + namespaces
frontend/src/stores/
  useJobStore.ts, useApplicationStore.ts, useMilestoneStore.ts, useDisputeStore.ts, useSupportStore.ts, useConfigStore.ts, useDashboardStore.ts (búsqueda cursor opaco + polling 15s), useAuthStore.ts (x-pubkey)
frontend/src/lib/
  dashboardUtils.ts  # countdown, autoApprove 7d, metrics, CSV, borradores localStorage
frontend/src/components/dashboard/
  RoleGuard, NotificationBell, OverviewCards, Chart7d, DeadlineCountdown, ChatTab, EvidenceTab, MilestoneTab, DisputeTab, PaymentsTab, HistoryTable
frontend/src/app/dashboard/
  layout.tsx (nav roles + NotificationBell) , page.tsx (selector) , freelancer/page.tsx (overview + filtros) , freelancer/jobs/[id]/page.tsx (5 tabs Chat/Evidencias/Milestones/Disputa/Pagos) , freelancer/history , client/page.tsx (En ejecución) , client/create (borradores) , client/disputes (tab separado) , client/history (métricas + CSV)
```

El back usa `trust-escrow-sdk` (7a2Y) → contrato `Vec<Pubkey>` 50 + `Application` PDA individual `[b"application", job, index, applicant]` + off-chain `metadata.rs` 6 structs.

---

## 5. Landing Dioxus (opcional)

```bash
cd landing
cargo install dioxus-cli --version 0.7.9
dx serve --port 8080
# → http://localhost:8080
# Tema dcdev ya aplicado en frontend; landing comparte tokens en design/themes/dcdev/
```

---

## Orden recomendado para levantar todo

```bash
# terminal 1 — validator
solana-test-validator --reset --ledger trust-escrow-v3/.anchor/test-ledger

# espera getHealth ok, luego en otra terminal:
cd trust-escrow-v3
ANCHOR_PROVIDER_URL=http://127.0.0.1:8899 anchor deploy --provider.cluster localnet -- --max-len 700000

# terminal 2 — DBs
docker run -d --name twe-postgres ... # ver paso 2
docker run -d --name twe-mongo ...

# terminal 3 — backend
cargo run -p trust-escrow-api

# terminal 4 — frontend
cd frontend && bun run dev --port 3001

# terminal 5 — landing (opcional)
cd landing && dx serve --port 8080
```

---

## Troubleshooting rápido

| Síntoma | Causa | Fix |
|---|---|---|
| `AccountNotFound: 7a2Y...` | validator sin deploy o ledger viejo | `rm -rf .anchor/test-ledger` + `anchor deploy -- --max-len 700000` |
| `Persistent Config ... advisor` | ledger con config vieja | reset ledger y redeploy limpio |
| `account data too small` | deploy sin max-len | `anchor deploy -- --max-len 700000` |
| `docker: 'compose' is not a docker command` | compose plugin no instalado | usa `docker run` directo (paso 2B) o `pip install docker-compose` |
| `failed to connect to npipe:////./pipe/dockerDesktopLinuxEngine` | WSL integration off | Docker Desktop → Settings → WSL Integration → activa Ubuntu |
| `ENOSPC /tmp` | ledger en tmpfs 3.9G lleno | mover ledger a `.anchor/test-ledger` (disco) |
| `EADDRINUSE :3000` | backend + frontend mismo puerto | frontend en `3001` (`--port 3001`) |
| `Bail out to client-side rendering: next/dynamic` | WalletMultiButton | normal, es dynamic `ssr:false` |

---

## Verificación final (un comando)

```bash
./scripts/final-gate.sh          # 20/20 PASS
cargo test --manifest-path backend/Cargo.toml --features solana
yarn --cwd trust-escrow-v3 test  # 9/9 (con ANCHOR_PROVIDER_URL=http://127.0.0.1:8899)
bun run --cwd frontend test      # 24/24
```

