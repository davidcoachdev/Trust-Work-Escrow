# Build Site: Backend v3 — SDK Rust + API REST axum

> Fase: **Map**. Este archivo define únicamente el grafo de implementación; no contiene código de producción.
> Estrategia: **quality** (máxima trazabilidad para un backend que firma y envía transacciones on-chain).
> Orden principal: **B1 → B2 → B3 → B6 → B4 → B5**.
> Modelo sugerido para subagents cuando el entorno lo permita: **hy3** en todos los tasks.

## Alcance y decisiones vinculantes

- Programa inmutable: `trust-escrow-v3`, program id `J1c4QsjbV9bFEPrFQZZGe8GrGWFxNhtAhhrxJFK2xc1h`.
- Anchor: 0.32.1.
- Opción B: crate SDK Rust adaptado de `trust-escrow-v2/sdk/` y API REST axum.
- Fuente de verdad: estado on-chain; metadata enriquecida off-chain vinculada por PDA.
- No se crea indexer/DB separada ni frontend en este MVP.
- Las nueve familias de PDA son: `config`, `job`, `application`, `arbiter_pool`, `dispute`, `arb_fee`, `milestone`, `evidence`, `support`.
- El sketch exige 39 wrappers: el source expone 38 handlers `pub fn` en `#[program]` y además `check_not_paused` como guard local reutilizable; el inventario del SDK debe dejar explícita esa diferencia, sin inventar una instrucción on-chain.

## Coverage Matrix

| Requirement / gate | Task(s) |
|---|---|
| R1 / FR-1 cliente, cluster y keypair | T1, T3, T18 |
| R2 / FR-2 nueve PDA y cache | T2 |
| R3 / FR-3 getters y cuentas ausentes | T3 |
| R4 / FR-4 errores tipados v3 | T3, T6 |
| R5 / FR-5 wrappers para 39 instrucciones | T4, T5, T6 |
| R6 / FR-6 listados y cursor | T7 |
| R7 / FR-7 applications derivadas por cuenta | T8 |
| R8 / FR-8 listener y fallback logs | T9, T12 |
| R9 / FR-9 rutas REST 1:1 | T13, T14, T16 |
| R10 / FR-10 validación HTTP | T15, T16 |
| R11 / FR-11 auth por firma del usuario | T15, T16 |
| R12 / FR-12 metadata en respuestas | T12, T16 |
| R13 / FR-13 health y métricas | T13, T16 |
| R14 / FR-14 `.env.example` y cluster switch | T17, T18 |
| R15 / FR-15 program ID configurable | T17, T18 |
| R16 / FR-16 logging y errores centralizados | T13, T19 |
| R17 / FR-17 validator/testnet, nunca mainnet | T6, T9, T20 |
| R18 / FR-18 metadata vinculada por PDA | T10, T12, T16 |
| R19 / FR-19 evidencia completa + hash | T11, T12, T16 |
| R20 / FR-20 índice por filtros y fecha | T11, T16 |
| R21 / FR-21 sincronización on-chain/off-chain | T9, T12 |
| Security B1: secretos/env/file, sin unwrap de red | T1, T3, T19 |
| Security B2: accounts validadas, sin skip-lint | T4, T5, T6 |
| Security B3: timeout RPC y loops acotados | T7, T9 |
| Security B4: HTTPS prod, rate-limit, CORS, input seguro, no keypair expuesto | T13, T15, T16 |
| Security B5: permisos 0600, `.env` ignorado, mainnet bloqueado | T17, T19, T20 |
| Security B6: backup y hash como comprobante | T10, T11, T12 |

## Topología y critical path

### Tier 0 — Setup base

#### T1 — Crear workspace y esqueletos de crates
- **Kit / requisito:** B1 / FR-1; security B1.
- **Descripción:** Definir `backend/Cargo.toml` como workspace y los esqueletos mínimos de `backend/sdk/` y `backend/api/`, fijando versiones compatibles con Anchor 0.32.1 y Solana. Copiar/adaptar la estructura de módulos de v2 sin implementar comportamiento nuevo.
- **Dependencias:** ninguna.
- **Archivos:** crear/modificar `backend/Cargo.toml`, `backend/sdk/Cargo.toml`, `backend/sdk/src/lib.rs`, `backend/sdk/src/{client,pda,types,events,error,utils}.rs`, `backend/api/Cargo.toml`, `backend/api/src/main.rs`, `backend/api/src/{routes,handlers,state,middleware}.rs`.
- **Done criteria:** `cargo metadata --manifest-path backend/Cargo.toml --no-deps` reconoce ambos crates; no hay secretos ni implementación de endpoints; el workspace compila con stubs explícitos y sin warnings nuevos.
- **Subagent/modelo:** `cavekit-make`, **hy3**, depth thorough.

### Tier 1 — SDK core (B1)

#### T2 — Adaptar derivación y cache de las nueve PDA
- **Kit / requisito:** B1 / FR-2.
- **Descripción:** Adaptar `v2/sdk/src/pda.rs` a las semillas y parámetros exactos del contrato v3: config singleton; job `(client, job_id)`; application `(job, index, applicant)`; arbiter pool singleton; dispute `(job)`; arb-fee `(job)`; milestone `(job, index)`; evidence `(dispute, index)`; support `(job)`. Diseñar cache thread-safe y vectores fijos contra el program ID aprobado.
- **Dependencias:** T1.
- **Archivos:** modificar `backend/sdk/src/pda.rs`, `backend/sdk/src/lib.rs`; crear `backend/sdk/tests/pda.rs`.
- **Done criteria:** cada familia tiene función de derivación y bump; los vectores fijos coinciden con `find_program_address` del contrato; cache hit retorna la misma dirección; hay benchmark/test que verifica el objetivo `<1ms` sin hacer RPC.
- **Subagent/modelo:** `cavekit-make`, **hy3**, depth thorough.

#### T3 — Cliente configurable, getters v3 y errores tipados
- **Kit / requisito:** B1 / FR-1, FR-3, FR-4.
- **Descripción:** Adaptar `client.rs`, `types.rs`, `error.rs` y `utils.rs` de v2 para construir `TrustEscrowClient` desde cluster y keypair env/path, deserializar `Config`, `Job`, `Application`, `ArbiterPool`, `Dispute`, `Evidence`, `Milestone` y `SupportTicket`, y mapear todos los errores del `ErrorCode` v3 a errores identificables sin panic.
- **Dependencias:** T2.
- **Archivos:** modificar `backend/sdk/src/{client,types,error,utils}.rs`, `backend/sdk/src/lib.rs`; crear `backend/sdk/tests/core.rs`.
- **Done criteria:** falla con error tipado si falta/invalid keypair; no hay secretos hardcodeados ni `unwrap()` en paths de red; getters devuelven cuenta válida o `None`; test local valida un job sembrado y un error de contrato identificable; `cargo test -p trust-escrow-sdk` y `cargo clippy -p trust-escrow-sdk -- -D warnings` pasan.
- **Subagent/modelo:** `cavekit-make`, **hy3**, depth thorough.

### Tier 2 — Wrappers (B2)

#### T4 — Wrappers de configuración, jobs y aplicaciones
- **Kit / requisito:** B2 / FR-5 (primer grupo).
- **Descripción:** Adaptar del cliente v2 los wrappers para `initialize_config`, `pause`, `unpause`, actualizaciones/retiros de treasury, `create_job`, `deposit_funds`, `apply_to_job`, `accept_application`, `cleanup_applications`, `submit_work`, `auto_approve_work`, `approve_work`, `reject_work`, `cancel_job`, `pause_job`, `unpause_job` y `expire_paused_job`, más el guard local `check_not_paused`, respetando las cuentas v3 y sus seeds.
- **Dependencias:** T3.
- **Archivos:** modificar `backend/sdk/src/client.rs`, `backend/sdk/src/types.rs`; crear/actualizar `backend/sdk/tests/instructions_jobs.rs`.
- **Done criteria:** el primer grupo de 21 entradas (20 handlers/operaciones contables y el guard local según el inventario del sketch) compila contra las cuentas v3; cada wrapper valida inputs y construye accounts sin cuentas omitidas; existe al menos un happy-path local por grupo; no se usa `skip-lint`.
- **Subagent/modelo:** `cavekit-make`, **hy3**, depth thorough.

#### T5 — Wrappers de arbitraje, disputas, soporte, evidencia y milestones
- **Kit / requisito:** B2 / FR-5 (segundo grupo).
- **Descripción:** Implementar los 18 handlers restantes: `create_arbiter_pool`, `add_arbiter`, `remove_arbiter`, `raise_dispute`, `accept_dispute`, `submit_evidence`, `assign_arbiter`, `resolve_dispute`, `resolve_platform_case`, `request_platform_intervention`, `open_support_ticket`, `resolve_support_ticket`, `finalize_dispute_payouts`, `cleanup_dispute_evidence`, `create_milestone`, `submit_milestone`, `approve_milestone` y `reject_milestone`. El inventario final debe reportar 38 handlers on-chain + `check_not_paused` local = 39 entradas del SDK, sin duplicar nombres.
- **Dependencias:** T4.
- **Archivos:** modificar `backend/sdk/src/client.rs`, `backend/sdk/src/types.rs`; crear/actualizar `backend/sdk/tests/instructions_disputes.rs`, `backend/sdk/tests/instructions_milestones.rs`.
- **Done criteria:** inventario automatizado enumera exactamente 38 instrucciones on-chain y el guard local adicional exigido por el sketch; cada entrada tiene un método público tipado; evidence/application cleanup usa remaining accounts deterministas; happy-path local cubre cada familia; errores de accounts inválidas son tipados.
- **Subagent/modelo:** `cavekit-make`, **hy3**, depth thorough.

#### T6 — Verificación integrada de wrappers y errores
- **Kit / requisito:** B2 / FR-5; FR-4; FR-17; security B2/B5.
- **Descripción:** Ejecutar escenarios de validator local para wrappers heredados y nuevos, verificar errores del contrato v3 y consolidar tests de integración SDK sin mainnet.
- **Dependencias:** T5.
- **Archivos:** crear/modificar `backend/sdk/tests/integration.rs`, `backend/sdk/tests/instructions.rs`, scripts de test bajo `backend/` si fueran necesarios.
- **Done criteria:** `cargo test -p trust-escrow-sdk instructions` pasa contra validator local/testnet controlado; cada fallo de precondición esperado identifica `BackendError::Contract`; el harness rechaza endpoint mainnet; `cargo clippy --workspace -- -D warnings` pasa.
- **Subagent/modelo:** `cavekit-check`, **hy3**, depth thorough.

### Tier 3 — Queries y eventos (B3)

#### T7 — Listados read-through con cursor y timeouts
- **Kit / requisito:** B3 / FR-6; security B3.
- **Descripción:** Adaptar listados v2 a jobs v3 por cliente/status, lectura de cuentas por RPC, paginación por cursor y configuración de timeout/retry con límites explícitos.
- **Dependencias:** T3.
- **Archivos:** modificar `backend/sdk/src/client.rs`, `backend/sdk/src/utils.rs`; crear `backend/sdk/tests/queries.rs`.
- **Done criteria:** listados distinguen client/freelancer y status; cursor devuelve páginas sin duplicados en >20 jobs; timeout produce error tipado; ningún loop de polling es infinito o sin cota.
- **Subagent/modelo:** `cavekit-make`, **hy3**, depth thorough.

#### T8 — Consulta de applications por job
- **Kit / requisito:** B3 / FR-7.
- **Descripción:** Crear `list_applications(job)` usando Accounts `Application` y sus semillas `(job, index, applicant)`, tratando `Job.applicants` como índice de candidatos y no como fuente completa de proposal/status.
- **Dependencias:** T2, T7.
- **Archivos:** modificar `backend/sdk/src/client.rs`, `backend/sdk/src/types.rs`; crear `backend/sdk/tests/applications.rs`.
- **Done criteria:** la consulta reconstruye la lista esperada del job sembrado, conserva index/applicant/proposal/status y maneja huecos o cuentas cerradas sin panic.
- **Subagent/modelo:** `cavekit-make`, **hy3**, depth standard.

#### T9 — Listener v3 y verificación de eventos/fallback
- **Kit / requisito:** B3 / FR-8; FR-21; FR-17; security B3.
- **Descripción:** Adaptar `events.rs` para WebSocket/logs, decodificar eventos efectivamente emitidos por v3 y, si el contrato no tiene `#[event]`, documentar el fallback por logs/getProgramAccounts. Exponer callback/canal acotado para sincronización posterior.
- **Dependencias:** T5, T7.
- **Archivos:** modificar `backend/sdk/src/events.rs`, `backend/sdk/src/client.rs`; crear `backend/sdk/tests/events.rs`, `backend/sdk/tests/fixtures/`.
- **Done criteria:** test local captura `submit_work` y `raise_dispute` o deja documentado el fallback utilizado; listener tiene timeout, buffer bounded y shutdown; eventos desconocidos no rompen el stream; cancelación puede emitirse al consumidor.
- **Subagent/modelo:** `cavekit-check`, **hy3**, depth thorough.

### Tier 4 — Metadata off-chain (B6)

#### T10 — Modelo/repositorio de metadata vinculado por PDA
- **Kit / requisito:** B6 / FR-18; security B6.
- **Descripción:** Definir la persistencia mínima para `title`, `description`, `proposal`, `reason`, `resolution` y referencias de entidad, siempre con PDA como clave y sin convertir la capa off-chain en fuente de estado contractual. Incluir estrategia de backup/restore.
- **Dependencias:** T3, T7.
- **Archivos:** crear `backend/api/src/metadata.rs`, `backend/api/src/repository.rs`, `backend/api/tests/metadata.rs`; modificar `backend/api/Cargo.toml`.
- **Done criteria:** se puede guardar/leer metadata por PDA; ausencia de metadata no oculta el estado on-chain; backup/restore de prueba conserva registros; test demuestra aislamiento de claves entre entidades.
- **Subagent/modelo:** `cavekit-make`, **hy3**, depth thorough.

#### T11 — Evidence off-chain, hash e índice consultable
- **Kit / requisito:** B6 / FR-19, FR-20; security B6.
- **Descripción:** Persistir `Evidence.content` completo fuera de cadena, calcular hash verificable y mantener índices por cliente, status, fecha y freelancer. El hash on-chain queda como comprobante, no como sustituto silencioso del contenido.
- **Dependencias:** T10, T8.
- **Archivos:** modificar `backend/api/src/{metadata,repository}.rs`; crear `backend/api/tests/evidence.rs`, `backend/api/tests/index.rs`.
- **Done criteria:** upload/read round-trip conserva bytes; hash reproducible y comparable con la evidencia on-chain; búsqueda por cliente ordena por `created_at`; filtros no mezclan jobs de otros clientes.
- **Subagent/modelo:** `cavekit-make`, **hy3**, depth thorough.

#### T12 — Sincronización por listener y consistencia
- **Kit / requisito:** B6 / FR-21; B3 / FR-8; B4 / FR-12.
- **Descripción:** Conectar eventos/read-through del SDK con el repositorio off-chain para actualizar flags ante cancelación, resolución o cambios relevantes, con reintento acotado e idempotencia.
- **Dependencias:** T9, T10, T11.
- **Archivos:** crear `backend/api/src/sync.rs`; modificar `backend/api/src/{state,metadata,repository}.rs`; crear `backend/api/tests/sync.rs`.
- **Done criteria:** evento de cancelación actualiza metadata una sola vez aunque se reprocesa; error RPC no borra metadata; reconciliación puede detectar divergencia on-chain/off-chain; test integra listener y repositorio.
- **Subagent/modelo:** `cavekit-check`, **hy3**, depth thorough.

### Tier 5 — API REST (B4)

#### T13 — Runtime axum, estado, errores, health y métricas
- **Kit / requisito:** B4 / FR-9, FR-13; FR-16; security B4.
- **Descripción:** Montar router axum stateless, estado compartido con SDK/repositorio, respuestas de error estructuradas, `GET /health` con estado RPC y contador de jobs. Centralizar tracing sin incluir keypairs.
- **Dependencias:** T3, T10, T12.
- **Archivos:** modificar `backend/api/src/{main,routes,handlers,state}.rs`; crear `backend/api/src/error.rs`, `backend/api/tests/health.rs`.
- **Done criteria:** API levanta desde configuración; `/health` devuelve 200 solo cuando el estado esperado está disponible; errores tienen esquema estable; métricas no contienen secretos; test de router no requiere mainnet.
- **Subagent/modelo:** `cavekit-make`, **hy3**, depth thorough.

#### T14 — Endpoints de jobs y acciones 1:1
- **Kit / requisito:** B4 / FR-9.
- **Descripción:** Exponer endpoints HTTP para crear/consultar jobs y acciones sensibles (`POST /jobs`, `GET /jobs/:id`, depósito, disputa y rutas necesarias para el MVP), delegando exclusivamente a wrappers del SDK y combinando metadata cuando corresponda.
- **Dependencias:** T6, T13.
- **Archivos:** modificar `backend/api/src/{routes,handlers,state}.rs`; crear `backend/api/tests/jobs.rs`, `backend/api/tests/transactions.rs`.
- **Done criteria:** body válido para crear job retorna PDA y firma/resultado; GET devuelve campos on-chain y metadata disponible; cada ruta usa método HTTP y status code documentados; no hay lógica duplicada de seeds en handlers.
- **Subagent/modelo:** `cavekit-make`, **hy3**, depth thorough.

#### T15 — Validación de input, auth por firma y middleware de seguridad
- **Kit / requisito:** B4 / FR-10, FR-11; security B4.
- **Descripción:** Validar montos, deadlines, strings y Pubkeys antes de invocar SDK; verificar firma del usuario para requests sensibles; añadir rate-limit, CORS restringido, HTTPS requerido en producción y rechazo explícito de payloads inseguros.
- **Dependencias:** T13, T14, T17.
- **Archivos:** modificar `backend/api/src/{middleware,handlers,routes,error}.rs`; crear `backend/api/tests/auth.rs`, `backend/api/tests/validation.rs`, `backend/api/tests/security.rs`.
- **Done criteria:** input inválido responde 400 estructurado y no toca RPC; firma ausente/inválida responde 401; válida permite proceder; rate-limit y CORS tienen tests; ninguna ruta expone el signer del servidor ni evalúa/shell crudo.
- **Subagent/modelo:** `cavekit-check`, **hy3**, depth thorough.

#### T16 — Integración API + SDK + metadata
- **Kit / requisito:** B4 / FR-9–FR-13; B6 / FR-18–FR-21.
- **Descripción:** Ejecutar suite de integración del API contra validator local y repositorio de metadata, cubriendo respuestas compuestas, auth, validación, health y sincronización.
- **Dependencias:** T12, T14, T15.
- **Archivos:** crear `backend/api/tests/integration.rs`, `backend/tests/`; modificar `backend/Cargo.toml` si se requieren targets de integración.
- **Done criteria:** `cargo test -p trust-escrow-api` y `cargo test --test integration` pasan; `GET /jobs/:id` incluye title/description off-chain cuando existe; no se realizan llamadas on-chain para bodies inválidos; clippy workspace pasa.
- **Subagent/modelo:** `cavekit-check`, **hy3**, depth thorough.

### Tier 6 — Configuración, seguridad operativa y verificación final (B5)

#### T17 — Config loader y archivos de entorno
- **Kit / requisito:** B5 / FR-14, FR-15; security B5.
- **Descripción:** Definir configuración tipada para `RPC_URL`, `CLUSTER`, `KEYPAIR_PATH`, `PROGRAM_ID`, defaults y mensajes de error claros; agregar `.env.example` sin secretos y reglas de ignorado.
- **Dependencias:** T1.
- **Archivos:** crear `backend/config/.env.example`, `backend/config/src.rs` o módulo equivalente, `backend/.gitignore`; modificar `backend/{Cargo.toml,sdk/Cargo.toml,api/Cargo.toml}`.
- **Done criteria:** ausencia de `.env`/variable requerida falla limpio; cambio localnet↔testnet cambia endpoint sin recompilar; program ID configurable con default aprobado; `.env` y keypairs quedan fuera de git; test de carga cubre defaults y errores.
- **Subagent/modelo:** `cavekit-make`, **hy3**, depth standard.

#### T18 — Cluster switch, bloqueo de mainnet y keypair seguro
- **Kit / requisito:** B5 / FR-14, FR-15, FR-17; B1 / FR-1.
- **Descripción:** Integrar config con SDK/API, resolver keypair desde env/path con permisos esperados, y bloquear mainnet en tests/CI mediante allowlist explícita de localnet/testnet controlado.
- **Dependencias:** T3, T17.
- **Archivos:** modificar `backend/config/src.rs`, `backend/sdk/src/client.rs`, `backend/api/src/main.rs`; crear `backend/config/tests.rs` y scripts de validación bajo `backend/`.
- **Done criteria:** mismo binario cambia endpoint y program ID por env; keypair ausente/ilegible produce error tipado; test y CI rechazan URL mainnet; no se imprime material secreto en errores.
- **Subagent/modelo:** `cavekit-check`, **hy3**, depth thorough.

#### T19 — Logging seguro, permisos y revisión de secretos
- **Kit / requisito:** B5 / FR-16; security B1/B4/B5.
- **Descripción:** Aplicar tracing estructurado, redacción de keypair/private key/tokens, permisos 0600 para archivos sensibles y revisión automatizada de `.gitignore`/diff para evitar secretos.
- **Dependencias:** T13, T17, T18.
- **Archivos:** modificar `backend/config/src.rs`, `backend/sdk/src/error.rs`, `backend/api/src/{main,middleware,error}.rs`; crear `backend/tests/security_logging.rs`.
- **Done criteria:** logs de transacción contienen signature/PDA sin secreto; test de redacción pasa; permisos de keypair se verifican o se rechaza configuración insegura; scanner no encuentra credenciales reales en `backend/`.
- **Subagent/modelo:** `cavekit-check`, **hy3**, depth thorough.

#### T20 — Gate final de workspace, validator y CI
- **Kit / requisito:** B5 / FR-17; verificación B1–B6.
- **Descripción:** Consolidar comandos reproducibles para levantar validator efímero/testnet controlado, ejecutar tests SDK/API/integración, clippy y revisión de cobertura de requirements/security gates. No habilitar mainnet.
- **Dependencias:** T6, T9, T16, T19.
- **Archivos:** crear/modificar `backend/Makefile` o scripts de test, `.github/workflows/` si existe CI, `backend/README.md` solo para comandos reproducibles.
- **Done criteria:** un comando documentado ejecuta `cargo test --workspace`; `cargo clippy --workspace -- -D warnings` pasa; validator/testnet controlado se usa de forma determinista; mainnet falla antes de enviar; coverage matrix de este plan queda 100% cubierta.
- **Subagent/modelo:** `cavekit-check`, **hy3**, depth thorough.

## Resumen de waves

| Wave / Tier | Tasks | Propósito |
|---|---|---|
| 0 | T1 | Workspace y crates base |
| 1 | T2 → T3 | PDA, cliente, tipos, getters y errores |
| 2 | T4 → T6 | 39 wrappers y verificación SDK |
| 3 | T7, T8, T9 | Queries, applications y eventos |
| 4 | T10 → T12 | Metadata, evidencia, índice y sync |
| 5 | T13 → T16 | API axum segura e integración |
| 6 | T17 → T20 | Config, secretos, clusters y gate final |

### Critical path

`T1 → T2 → T3 → T4 → T5 → T6 → T9 → T12 → T13 → T14 → T15 → T16 → T19 → T20`.

El camino crítico atraviesa SDK core, wrappers completos, listener, metadata sync, API/auth y verificación operativa. T7/T8 alimentan B3/B6 en paralelo parcial, pero T8 es necesario para metadata de applications y T9 depende de wrappers para validar eventos reales.

## Result Contract

- **Estrategia:** `quality`.
- **Tasks:** 20, distribuidos en 7 tiers/waves.
- **Orden:** B1 → B2 → B3 → B6 → B4 → B5.
- **Coverage:** 21/21 requirements y 6/6 dominios de security gate mapeados; el gate final T20 verifica la cobertura completa.
- **Next:** ejecutar Make sobre el primer task elegible (`T1`) y luego Check por wave; este archivo no implementa producción.
