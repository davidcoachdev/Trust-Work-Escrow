# Backend v3 — Sketch de Kits (Cavekit, opción B: SDK + API REST)

> **Decisión:** Backend en Rust = SDK de integración (adaptado de v2) + API REST (axum).
> **Contrato:** `trust-escrow-v3` (programa `J1c4QsjbV9bFEPrFQZZGe8GrGWFxNhtAhhrxJFK2xc1h`, Anchor 0.32.1).
> **Reuse (Fase 0):** REUSE de `trust-escrow-v2/sdk/` (client.rs 68 fns, pda.rs 34 fns, types/events/utils/error).
>   Adaptar, NO reescribir. v3 añade 8 instrucciones y 1 PDA (`evidence`) vs v2.
> **Modelo subagentes:** hy3 (mismo para todos).
> **Fecha:** 2026-08-12

## Estructura propuesta
```
backend/
├── Cargo.toml            # workspace
├── sdk/                  # crate trust-escrow-sdk (fork de v2 sdk → v3)
│   ├── src/{client,pda,types,events,error}.rs
│   └── tests/
├── api/                  # crate trust-escrow-api (axum)
│   ├── src/{routes,handlers,state,middleware}.rs
│   └── tests/
└── config/               # env (cluster, keypair path), .env.example
```

---

## Kit B1 — SDK Core (cliente + PDA derivation)

**Goal:** Cliente Anchor que conecta al contrato v3 y deriva las 9 PDAs de forma determinista.
**Constraints:** Calidad+seguridad primero; Strict TDD; reusa patrones de v2 sdk/pda.rs.

**FR-1:** `TrustEscrowClient::new(cluster, keypair)` inicializa con cluster configurable
(localnet/testnet/mainnet) y keypair cargado desde env/path. Sin secretos hardcodeados.
- [ ] `cargo test` new_localnet conecta a validator en 8899 (o testnet) sin error.
- [ ] Keypair desde `TEST_KEYPAIR` env carga; falla limpio si no existe (error tipado, no panic).

**FR-2:** Derivación de las 9 PDAs de v3 con cache (reusa v2 pda.rs: `derive_*_pda`).
- [ ] `derive_job_pda(client, freelancer)` produce address idéntico al del contrato (test vector fijo).
- [ ] Seeds correctas: config, job, application, arbiter_pool, dispute, arb_fee, milestone, evidence, support.
- [ ] Cache hit devuelve misma dirección en <1ms (test de rendimiento).

**FR-3:** Getters `get_config/get_job/get_application/get_arbiter_pool/get_dispute/get_milestone/get_evidence/get_support_ticket` deserializan tipos v3.
- [ ] `get_job(addr)` devuelve struct con campos correctos para un job sembrado en test.
- [ ] Cuenta inexistente → `None` (no panic).

**FR-4:** `ErrorCode` de v3 mapeado a enum de errores tipados del backend.
- [ ] Llamada que viola regla del contrato devuelve `BackendError::Contract(ErrorCode::X)` identificable.

**Security Gates:** sin secretos en código; keypair solo desde env/file con permisos; sin `unwrap()` en paths de red.
**Verification:** `cargo test -p trust-escrow-sdk`, `cargo clippy`.
**DoD:** cliente + 9 PDAs + getters + error mapping, todos tests verdes contra validator local.

---

## Kit B2 — Wrappers de 39 instrucciones

**Goal:** Una función async tipada por cada instrucción v3.
**Constraints:** Reusa cuerpo de v2 client.rs; añade las 8 nuevas de v3.

**FR-5:** Wrappers para las 39 instrucciones (`check_not_paused` → `reject_milestone`), con accounts + args tipados.
- [ ] Las 31 heredadas de v2 compilan y ejecutan contra v3 (adaptadas a cambios de accounts).
- [ ] Las 8 nuevas v3 tienen wrapper: `open_support_ticket`, `resolve_support_ticket`,
  `request_platform_intervention`, `resolve_platform_case`, `create_arbiter_pool`, `add_arbiter`,
  `remove_arbiter`, `finalize_dispute_payouts` (más las de pause/unpause/cleanup/auto_approve).
- [ ] Cada wrapper tiene al menos 1 test de happy-path contra validator local.

**Security Gates:** validación de accounts antes de enviar; sin `skip-lint`.
**Verification:** `cargo test -p trust-escrow-sdk instructions`.
**DoD:** 39 wrappers, cobertura happy-path mínima.

---

## Kit B3 — Queries & eventos (read-through)

**Goal:** Consultas compuestas y escucha de eventos on-chain.
**Constraints:** Fuente de verdad = on-chain; backend es espejo.

**FR-6:** Listados por cliente/job/status (reusa `list_escrows_*` de v2 → `list_jobs_by_client`, `list_jobs_by_status`).
- [ ] `list_jobs_by_client(pubkey)` devuelve todos los jobs donde es client/freelancer.
- [ ] Paginación por cursor (test con >20 jobs).

**FR-7:** `Job.applicants` se deriva consultando Accounts `Application` por job (no del Vec on-chain).
- [ ] `list_applications(job)` devuelve misma lista que el Vec on-chain en test.

**FR-8:** Event listener (WebSocket/logs) decodifica eventos v3.
- [ ] Listener captura `submit_work`/`raise_dispute` en test y dispara callback.
- [ ] (Si v3 no emite `#[event]`, documentar y usar getProgramAccounts/logs como fallback.)

**Security Gates:** timeouts en RPC; sin bucles infinitos sin cota.
**Verification:** `cargo test -p trust-escrow-sdk queries`.
**DoD:** listados + derivación applicants + listener, tests verdes.

---

## Kit B4 — API REST (axum)

**Goal:** Endpoints HTTP 1:1 con instrucciones, para que landing/frontends consuman.
**Constraints:** Stateless; validación de input; auth por keypair firmado.

**FR-9:** Rutas REST mapean instrucciones (ej. `POST /jobs`, `POST /jobs/:id/deposit`, `POST /disputes`, `GET /jobs/:id`).
- [ ] `POST /jobs` con body válido crea job on-chain y devuelve PDA.
- [ ] `GET /jobs/:id` devuelve JSON del job (campos on-chain).

**FR-10:** Validación de input (montos >0, deadlines futuros, strings no vacíos).
- [ ] Body inválido → 400 con mensaje estructurado; no llama al contrato.

**FR-11:** Auth: requests sensibles requieren firma del keypair del usuario (no el del servidor).
- [ ] Request sin firma válida → 401.
- [ ] Request con firma válida procede.

**FR-12:** Respuestas incluyen metadata off-chain (del Kit B6) cuando aplica.
- [ ] `GET /jobs/:id` incluye `title`/`description` desde capa off-chain.

**FR-13:** Health + metrics (`GET /health`, conteo de jobs).
- [ ] `/health` devuelve 200 con estado de conexión RPC.

**Security Gates:** HTTPS solo en prod; rate-limit; sin exposición de keypair del servidor;
  CORS restringido; sin `eval`/shell crudo; input sanitizado.
**Verification:** `cargo test -p trust-escrow-api`, `cargo test --test integration`.
**DoD:** API levanta, endpoints mapean instrucciones, auth + validación + tests verdes.

---

## Kit B5 — Config & clusters & seguridad

**Goal:** Gestión de entorno sin secretos en código.
**Constraints:** Mismos patrones de v2 cli (env, keypair path, cluster switch).

**FR-14:** `.env.example` define `RPC_URL`, `CLUSTER`, `KEYPAIR_PATH`, `PROGRAM_ID`.
- [ ] Cargar sin `.env` en local falla con error claro (no panic).
- [ ] Switch localnet↔testnet vía env cambia endpoint sin recompilar.

**FR-15:** `PROGRAM_ID` configurable; default = `J1c4Q...`.
- [ ] Test usa programa por env; misma binaria sirve local y testnet.

**FR-16:** Logging estructurado (sin secretos) y manejo de errores centralizado.
- [ ] Log de transacción no imprime keypair/private key.

**FR-17:** Tests usan validator local (8899) o testnet controlado, nunca mainnet.
- [ ] CI/script levanta validator efímero o usa testnet; mainnet bloqueado por flag.

**Security Gates:** secretos solo en env/file con 0600; nunca en git; `.env` en .gitignore.
**Verification:** `cargo test`, revisión de `.gitignore`.
**DoD:** config carga, cluster switch, logging seguro, tests aislados.

---

## Kit B6 — Capa metadata off-chain

**Goal:** Mantener en backend la metadata rica (ver `docs/auditoria/separation-onchain-offchain-backend.md`).
**Constraints:** On-chain = verdad; off-chain = espejo enriquecido vinculado por PDA.

**FR-18:** Tablas off-chain para `title`, `description`, `proposal`, `reason`, `resolution` de cada entidad,
vinculadas por clave PDA.
- [ ] Crear job on-chain + guardar description off-chain → `GET` devuelve ambos.
- [ ] Hash de content (evidence) guardado como integridad.

**FR-19:** `Evidence.content` (Vec<u8>) se guarda completo off-chain; on-chain queda como respaldo/hash.
- [ ] Subir evidencia guarda content en backend y refleja en on-chain.

**FR-20:** Índice consultable (por cliente, status, fecha, freelancer) sobre metadata off-chain.
- [ ] Búsqueda por cliente devuelve jobs ordenados por `created_at`.

**FR-21:** Sincronización: si on-chain cambia (ej. job cancelado), metadata off-chain refleja estado.
- [ ] Evento de cancelación actualiza flag off-chain (vía Kit B3 listener).

**Security Gates:** persistencia con backup; no perder metadata si backend cae; hash on-chain como comprobante.
**Verification:** `cargo test -p trust-escrow-api metadata`.
**DoD:** metadata rica guardada/vinculada/indexada, tests verdes.

---

## Out of Scope (MVP)
- Indexer propio con DB separada (se hace post-MVP; MVP usa read-through + listener en memoria).
- Frontend (landing ya existe, consume la API).
- Migración de campos del contrato v3 a hash-on-chain (v3 ya aprobado; futuro v4).

## Cross-References
- B1 ← base de B2, B3, B4.
- B4 depende de B1 (SDK) y B6 (metadata).
- B6 depende de B3 (listener) para sync.
- Reuse: v2 `sdk/client.rs`, `sdk/pda.rs`, `sdk/types.rs`, `sdk/events.rs`.

## Result Contract
6 kits, 21 requirements, ~30 acceptance criteria.
Security gates: 6 dominios cubiertos.
Next: `/sdd-cavekit map`
