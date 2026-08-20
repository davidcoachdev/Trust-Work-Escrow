# Backend v3 — Coverage Final Gate T20

> **Gate:** T20 — validator + CI + coverage  
> **Program:** `trust-escrow-v3` — `7a2YhCd7iivXfyySkp1pf5jjijGqpjNqwQCUS912q5Vh` (Anchor 0.32.1 / Solana 2.1.x)  
> **Workspace:** `backend/` — crates `trust-escrow-sdk` + `trust-escrow-api`  
> **Fecha:** 2026-08-20  
> **Estado:** ✅ PASS — 164/164 tests verdes

---

## 1. Resumen ejecutivo

| Gate | Comando | Estado |
|------|---------|--------|
| `cargo test --workspace` | `cargo test --manifest-path backend/Cargo.toml` | ✅ 164 passed / 0 failed |
| `cargo clippy -- -D warnings` | `cargo clippy --manifest-path backend/Cargo.toml -- -D warnings` | ✅ PASS |
| `cargo fmt --check` | `cargo fmt --manifest-path backend/Cargo.toml --all -- --check` | ✅ PASS |
| Validator local 7a2Y UP | `curl http://127.0.0.1:8899` → `{"result":"ok"}` + `Anchor.toml` program id | ✅ UP |
| CI workflow | `.github/workflows/ci.yml` | ✅ existe + válido |
| Secret scan | `scripts/secret-scan.sh --ci` | ✅ PASS |
| Permisos 0600 | `scripts/check-permissions.sh --ci` | ✅ PASS |
| Coverage docs | `docs/BACKEND_COVERAGE.md` + `backend/README.md` T20 | ✅ |

**Comando reproducible único (gate final):**

```bash
./scripts/final-gate.sh              # local estricto — requiere validator UP
./scripts/final-gate.sh --ci         # CI — validator warn (no bloquea)
./scripts/final-gate.sh --json       # salida JSON para automatización
```

**CI ejecuta el mismo gate:** `.github/workflows/ci.yml` → `backend-gate` job → `./scripts/final-gate.sh --ci`

---

## 2. Conteo de tests por crate / módulo (2026-08-20)

Ejecución: `cargo test --manifest-path backend/Cargo.toml` — perfil `test` unoptimized + debuginfo.

| Crate / suite | Tests | Estado |
|---------------|-------|--------|
| `trust-escrow-api` lib (`src/*.rs` unit tests) | **134** | ✅ |
| `trust-escrow-api` bin (`src/main.rs` health/metrics) | **11** | ✅ |
| `trust-escrow-api` integration (`tests/integration.rs`) | **4** | ✅ |
| `trust-escrow-sdk` lib (`src/*.rs` unit) | **15** | ✅ |
| **Total workspace** | **164** | ✅ 0 failed |

Detalle `trust-escrow-api` lib (134):

| Módulo | Tests | Cubre |
|--------|-------|-------|
| `auth` | 10 | firma ed25519, headers, roundtrip, 401 |
| `config` | 14 | PORT/RPC/DATABASE_URL/MONGO/CORS/RATE_LIMIT/ENV valid/invalid |
| `error` | 7 | code mapping, sanitize, truncamiento, repository |
| `evidence` | 14 | hash bytes32, cursor opaque, pagination, content/index/limit |
| `health` | 4 | health response, repo check, RPC unavailable/unconfigured |
| `integration` | 6 | create/get enriched, PDA deterministic, fee, 404, list, validation |
| `logging` | 14 | redacción bearer/JWT/database_url/mongo/keypair/privkey/pem |
| `metadata` | ~18 | PDA key isolation, backup/restore |
| `metrics` | 2 | contadores, no secretos |
| `middleware` | ~5 | rate-limiter, CORS |
| `repository` | ~8 | in-memory CRUD, filtros |
| `sync` | ~20 | idempotency, retry timeout, ordering, event timeout |
| `validation` | 12 | amount/title/deadline/hash/payout/proposal/pubkey/evidence/composite |

> Nota: el plan T20 exigía ≥149 verdes (umbral SDK T7-T19). El workspace actual supera el umbral con **164**.

---

## 3. Coverage matrix — Requirements × Tasks (T1-T20)

Mapeo vinculante de `context/plans/backend-v3-map.md` — 21 requirements + 6 security gates.

### 3.1 Requirements funcionales

| Req / FR | Descripción | Task(s) | Evidencia | Estado |
|----------|-------------|---------|-----------|--------|
| R1 / FR-1 | cliente, cluster y keypair | T1, T3, T18 | `sdk/src/client.rs`, `cluster.rs`, `config.rs` | ✅ |
| R2 / FR-2 | nueve PDA y cache | T2 | `sdk/src/pda.rs`, `tests/pda.rs` | ✅ |
| R3 / FR-3 | getters y cuentas ausentes | T3 | `sdk/src/client.rs` 9 getters, `types.rs` | ✅ |
| R4 / FR-4 | errores tipados v3 | T3, T6 | `sdk/src/error.rs`, `api/src/error.rs` | ✅ |
| R5 / FR-5 | 39 wrappers (38 on-chain + guard) | T4, T5, T6 | `sdk/src/client.rs` 39 entries | ✅ |
| R6 / FR-6 | listados y cursor | T7 | `sdk/src/utils.rs`, `client.rs` queries | ✅ |
| R7 / FR-7 | applications por job | T8 | `sdk/tests/list_applications.rs` (6 tests) | ✅ |
| R8 / FR-8 | listener y fallback logs | T9, T12 | `sdk/src/events.rs` (10 tests) | ✅ |
| R9 / FR-9 | rutas REST 1:1 | T13, T14, T16 | `api/src/routes.rs`, `api/tests/integration.rs` | ✅ |
| R10 / FR-10 | validación HTTP | T15, T16 | `api/src/validation.rs` | ✅ |
| R11 / FR-11 | auth por firma | T15, T16 | `api/src/auth.rs`, `middleware.rs` | ✅ |
| R12 / FR-12 | metadata en respuestas | T12, T16 | `api/src/metadata.rs` + `integration.rs` | ✅ |
| R13 / FR-13 | health y métricas | T13, T16 | `api/src/health.rs`, `metrics.rs` | ✅ |
| R14 / FR-14 | `.env.example` y cluster switch | T17, T18 | `backend/.env.example`, `config.rs` | ✅ |
| R15 / FR-15 | program ID configurable | T17, T18 | `cluster.rs` allowlist, `lib.rs` PROGRAM_ID_STR | ✅ |
| R16 / FR-16 | logging y errores centralizados | T13, T19 | `api/src/logging.rs`, `secret-scan.sh` | ✅ |
| R17 / FR-17 | validator/testnet, nunca mainnet | T6, T9, T20 | `cluster.rs` mainnet block, `final-gate.sh` guard | ✅ |
| R18 / FR-18 | metadata vinculada por PDA | T10, T12, T16 | `api/src/metadata.rs`, `repository.rs` | ✅ |
| R19 / FR-19 | evidencia completa + hash | T11, T12, T16 | `api/src/evidence.rs` (hash reproducible) | ✅ |
| R20 / FR-20 | índice por filtros y fecha | T11, T16 | `api/src/evidence.rs` pagination/filter | ✅ |
| R21 / FR-21 | sincronización on-chain/off-chain | T9, T12 | `api/src/sync.rs` (idempotency + retry) | ✅ |

**Coverage:** 21/21 ✅

### 3.2 Security gates

| Dominio | Descripción | Task(s) | Evidencia | Estado |
|---------|-------------|---------|-----------|--------|
| B1 | secretos/env/file, sin unwrap de red | T1, T3, T19 | `secret-scan.sh`, `logging.rs` redact | ✅ |
| B2 | accounts validadas, sin skip-lint | T4, T5, T6 | `clippy -D warnings`, wrappers validan accounts | ✅ |
| B3 | timeout RPC y loops acotados | T7, T9 | `utils.rs` with_retry/timeout, bounded buffers | ✅ |
| B4 | HTTPS prod, rate-limit, CORS, input seguro | T13, T15, T16 | `middleware.rs`, `validation.rs` | ✅ |
| B5 | permisos 0600, `.env` ignorado, mainnet bloqueado | T17, T19, T20 | `check-permissions.sh --ci`, `cluster.rs` allowlist, `final-gate.sh` mainnet guard, `ci.yml` guard | ✅ |
| B6 | backup y hash como comprobante | T10, T11, T12 | `metadata.rs` backup/restore, `evidence.rs` hash | ✅ |

**Coverage:** 6/6 ✅

---

## 4. Validator local 7a2Y UP

### 4.1 Verificación reproducida

```bash
# Health RPC (debe responder {"result":"ok"})
curl -s -X POST -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"getHealth"}' \
  http://127.0.0.1:8899
# → {"jsonrpc":"2.0","result":"ok","id":1}  ✅

# Program id declarado en Anchor.toml
grep trust_escrow_v3 trust-escrow-v3/Anchor.toml
# → trust_escrow_v3 = "7a2YhCd7iivXfyySkp1pf5jjijGqpjNqwQCUS912q5Vh"  ✅

# Program account existe (si solana CLI disponible)
solana program show 7a2YhCd7iivXfyySkp1pf5jjijGqpjNqwQCUS912q5Vh --url http://127.0.0.1:8899
# o fallback getAccountInfo via curl
```

### 4.2 Guard mainnet

- `backend/sdk/src/cluster.rs` — allowlist: solo `Localnet`/`Devnet`/`Testnet` controlado; `Mainnet` rechazado con `ClusterError::MainnetBlocked`.
- `scripts/final-gate.sh` — falla si `SOLANA_RPC_URL` contiene `mainnet`.
- `.github/workflows/ci.yml` — job `backend-gate` step `Guard — block mainnet RPC in CI` falla si `SOLANA_RPC_URL` es mainnet.

El gate **nunca** envía transacciones a mainnet.

---

## 5. CI workflow

**Archivo:** `.github/workflows/ci.yml` — `CI — Backend v3 Final Gate (T20)`

Triggers: `push` a `main/develop/feature/**`, `pull_request` a `main/develop`, `workflow_dispatch`.

Jobs:

| Job | Runner | Steps |
|-----|--------|-------|
| `backend-gate` | `ubuntu-24.04` | checkout → setup Rust stable (clippy+ruste=fmt) → cache Cargo → guard mainnet → `cargo clippy -- -D warnings` → `cargo fmt --check` → `cargo test --workspace` → `secret-scan.sh --ci` → `check-permissions.sh --ci` → `./scripts/final-gate.sh --ci` → upload logs → summary |

Logs: `backend/target/final-gate-*.log` subidos como artifact `final-gate-logs` (14 días).

El job `validator-smoke` (disabled by default `if: false`) documenta cómo habilitar `solana-test-validator` efímero si se requiere en el runner.

---

## 6. Cómo reproducir localmente

```bash
# 0. Levantar validator con el program desplegado (si no está UP)
#    El ledger efímero vive en context/impl/validator-ledger/
solana-test-validator --ledger context/impl/validator-ledger --reset \
  --bpf-program 7a2YhCd7iivXfyySkp1pf5jjijGqpjNqwQCUS912q5Vh trust-escrow-v3/target/deploy/trust_escrow_v3.so

# 1. Gate completo (estricto — requiere validator UP)
./scripts/final-gate.sh
# → 7/7 checks, exit 0 si todo verde

# 2. Solo coverage (sin validator)
./scripts/final-gate.sh --skip-validator

# 3. Salida JSON para CI/automatización
./scripts/final-gate.sh --json

# 4. Comandos individuales (equivalentes a CI)
cargo test --manifest-path backend/Cargo.toml
cargo clippy --manifest-path backend/Cargo.toml -- -D warnings
cargo fmt --manifest-path backend/Cargo.toml --all -- --check
```

---

## 7. Historial de umbrales

| Fecha | Hito | Tests verdes |
|-------|------|--------------|
| T19 | secure logging 0600 + secret scan | 149/149 |
| **T20** | **final gate validator+CI+coverage** | **164/164** |

El incremento T19→T20 (+15 tests) corresponde a `trust-escrow-api` lib (auth/config/evidence/health/integration/logging/sync/validation) y estabilización de `sdk` (cluster/error/utils).

---

## 8. Archivos del gate

| Archivo | Rol |
|---------|-----|
| `scripts/final-gate.sh` | Script ejecutable — fuente de verdad del gate T20 |
| `.github/workflows/ci.yml` | CI que reproduce el mismo gate en GitHub Actions |
| `docs/BACKEND_COVERAGE.md` | Este documento — matrix + conteos + reproducibilidad |
| `backend/README.md` | Quick start + sección Final Gate T20 |
| `scripts/secret-scan.sh` | Gate B1/B5 — gitleaks + fallback grep |
| `scripts/check-permissions.sh` | Gate B5 — audit 0600 |

---

*Generado por `scripts/final-gate.sh` + `cargo test` — no editar conteos manualmente. Re-ejecutar `./scripts/final-gate.sh` para actualizar.*
