# Cavekit: Backend v3 — bridge SDK, proyección y sincronización

## Goal
Definir un backend v3 donde terminal/TUI y API compartan `trust-escrow-sdk` como única frontera blockchain, Solana/on-chain sea la autoridad contractual y DB sea únicamente una proyección enriquecida, metadata, auditoría y estado de sincronización.

## Constraints
- **Calidad:** cada flujo debe ser trazable `API → application service → trust-escrow-sdk → Solana`, con estados, errores y evidencia verificables.
- **Seguridad:** API, workers y DB no pueden saltar el SDK para hablar con Anchor/RPC; no hay secretos en DB, logs, fixtures o documentación; autorización y ownership se validan antes de mutar.
- **Strict TDD:** cada criterio comienza con una prueba RED, pasa por el cambio mínimo GREEN y termina con refactor; los dobles del SDK no sustituyen las pruebas de integración necesarias.
- **Autoridad:** el estado contractual, ownership de cuentas, firmas y finality canónica provienen de Solana/IDL/commitment; DB nunca autoriza ni reemplaza una decisión on-chain.
- **ROI/YAGNI:** se permite un worker/reconciliador in-process durable para MVP; no se agrega un indexer genérico, microservicio separado, event-sourcing general, bus distribuido ni nueva superficie de producto.

## Reuse Obligatorio
Este kit implementa el veredicto **PARTIAL** de `context/refs/reuse-report.md` y reutiliza:
- `07-docs-idl-sync.md` R1–R4 como gate de verdad documental, extendido al bridge SDK, límites de autoridad y finality.
- `08-final-validation.md` R1–R4 como formato de gates, evidencia reproducible y estados PASS/FAIL/BLOCKED/ACCEPTED.
- `05-security-tests.md` R1–R5 como patrón de pruebas negativas, ownership, atomicidad, replay, estados terminales y conservación.
- `04-deploy-runbook.md` R1–R4 y `01-config-bootstrap.md` R1–R4 para endpoint/program ID, autoridades, treasury y modelo de signer.

## Requirements

### R1: Ownership y autoridad de datos
**Description:** Cada dato queda clasificado por autoridad y ningún consumidor interpreta una proyección local como autoridad contractual.
**Acceptance Criteria:**
- [ ] Una matriz verificable clasifica cada campo como **on-chain canónico**, **metadata DB**, **auditoría/sync**, o **evidencia externa**, e indica su fuente y regla de actualización.
- [ ] Para un conflicto entre Solana y DB, el sistema conserva la discrepancia, marca DB como stale/divergente y usa el valor on-chain para estado contractual, ownership, balances y finality.
- [ ] Ningún endpoint o comando terminal puede aprobar, pagar, cerrar, reasignar o cambiar estado contractual basándose únicamente en DB.
- [ ] Un test negativo demuestra que modificar una proyección local no habilita una mutación contractual ni cambia el resultado autorizado.
**Dependencies:** `01-config-bootstrap.md` R1–R4; `03-deadlines-auto-approval.md` R1–R5.

### R2: Frontera única `API → application service → SDK`
**Description:** La API y la terminal/TUI ejecutan operaciones blockchain mediante el mismo contrato de aplicación y `trust-escrow-sdk`, sin acceso directo a Anchor ni RPC.
**Acceptance Criteria:**
- [ ] Cada operación mutante y de lectura expone una ruta trazable desde API o terminal/TUI hasta una operación tipada del SDK y su resultado Solana.
- [ ] Un análisis automatizado de dependencias o test de arquitectura falla si API, application services, terminal/TUI, workers o reconciliador importan/callan Anchor, clientes RPC o primitivas blockchain fuera de `trust-escrow-sdk`.
- [ ] API y terminal/TUI reciben los mismos tipos de dominio, errores normalizados y estados de finality para la misma operación.
- [ ] Un test con un fake del SDK verifica que el application service no construye transacciones ni modifica cuentas por su cuenta.
**Dependencies:** `05-security-tests.md` R1–R3; R1.

### R3: Contrato público del SDK compartido
**Description:** `trust-escrow-sdk` define la frontera compartida para operaciones, tipos, errores, eventos, versionado y límites de signer/lectura.
**Acceptance Criteria:**
- [ ] Un inventario machine-readable relaciona cada operación pública con entradas validadas, cuentas/ownership esperados, signer requerido, resultado, errores y estado de finality.
- [ ] Las mismas operaciones consumidas por terminal/TUI y API producen resultados equivalentes ante éxito, rechazo on-chain, timeout y desconexión.
- [ ] El versionado del contrato SDK rechaza incompatibilidades explícitamente y no degrada silenciosamente a una operación distinta.
- [ ] Tests unitarios cubren validación de entradas, errores tipados y ausencia de panic/catch silencioso en rutas críticas.
**Dependencies:** R2; `07-docs-idl-sync.md` R3–R4.

### R4: Schema DB y reglas de proyección
**Description:** La persistencia representa cuentas y hechos on-chain como proyecciones enriquecidas, separa metadata mutable y auditoría, y conserva claves suficientes para reconciliar.
**Acceptance Criteria:**
- [ ] El esquema define claves on-chain estables (program ID, cluster, account/PDA), versión de proyección, slot/commitment observado, timestamps de ingestión y origen del dato.
- [ ] Metadata mutable, auditoría de requests/transacciones y cursores de sincronización no se mezclan con campos contractuales sin una marca de autoridad explícita.
- [ ] Cada proyección de estado puede vincularse a account/evento/slot/transacción o queda marcada como no verificable; no se inventan valores contractuales ausentes.
- [ ] Tests de persistencia demuestran upsert seguro, aislamiento por cluster/program ID, rechazo de claves inválidas y que DB no puede escribir una transición contractual como hecho confirmado sin evidencia on-chain.
**Dependencies:** R1; R5; `04-deploy-runbook.md` R1–R4.

### R5: Seguimiento durable de transacciones
**Description:** Toda operación que pueda producir una transacción tiene un registro durable, correlacionable e independiente del ciclo de vida de la request HTTP o TUI.
**Acceptance Criteria:**
- [ ] Antes de enviar una mutación se registra una intención con idempotency key, actor/subject autorizado, operación SDK, cluster/program ID y payload sanitizado sin secretos.
- [ ] El registro conserva firma si existe, timestamps, último estado de finality, intento, error clasificado y relación con las proyecciones afectadas.
- [ ] Reiniciar el proceso conserva y reanuda transacciones pendientes sin perder la correlación ni crear una segunda intención equivalente.
- [ ] Un test de crash entre registro, envío y confirmación demuestra recuperación determinista y ausencia de doble aplicación lógica.
**Dependencies:** R2–R4; `05-security-tests.md` R3/R5.

### R6: Listener durable y reconciliación periódica
**Description:** La proyección se actualiza mediante escucha durable y una reconciliación periódica contra Solana, tolerando reinicios, pérdida de conexión, duplicados y divergencias.
**Acceptance Criteria:**
- [ ] El listener persiste cursor/checkpoint por cluster/program ID, reanuda desde el último punto seguro y hace backoff acotado ante desconexión.
- [ ] Eventos repetidos o entregados fuera de orden no duplican efectos; la deduplicación usa una identidad verificable de evento/transacción/slot y conserva auditoría del descarte.
- [ ] La reconciliación consulta la fuente canónica mediante el SDK, detecta proyecciones faltantes, stale, divergentes y cuentas cerradas, y repara solo DB/auditoría.
- [ ] Un test de reinicio, duplicación, hueco de cursor y discrepancia confirma que no se muta Solana desde el reconciliador y que cada reparación queda trazada.
**Dependencies:** R4–R5; R7; `08-final-validation.md` R2/R4.

### R7: Idempotencia, retries y estados de finality
**Description:** Requests, envíos, eventos y trabajos de sincronización tienen deduplicación y reintentos explícitos, con un vocabulario único de finality.
**Acceptance Criteria:**
- [ ] Repetir una request con la misma idempotency key devuelve el resultado durable original o su estado actual, sin crear una segunda mutación contractual.
- [ ] Retries solo ocurren para errores clasificados como transitorios, tienen límite/backoff y no repiten una operación no idempotente sin correlación durable.
- [ ] El sistema usa un vocabulario único verificable —por ejemplo `intent`, `submitted`, `processed`, `confirmed`, `finalized`, `failed`, `reorged`— y define transiciones permitidas y terminales.
- [ ] API, terminal/TUI, SDK, DB, listener y reconciliador exponen el mismo estado y no muestran `finalized` antes de la evidencia/commitment correspondiente.
- [ ] Tests cubren timeout después del envío, firma conocida, fallo permanente, reintento, reorg/reversal si aplica al proveedor y transición a estado terminal sin doble payout.
**Dependencies:** R5–R6; `05-security-tests.md` R3; `08-final-validation.md` R2/R4.

### R8: Tombstones de cuentas cerradas
**Description:** El cierre de una cuenta on-chain queda representado de forma durable y evita que una proyección vieja reaparezca como cuenta activa.
**Acceptance Criteria:**
- [ ] Al observar un cierre válido, la proyección conserva tombstone con PDA, program ID/cluster, slot/finality, firma si existe y motivo/origen observado.
- [ ] Reconciliación y lecturas no recrean una cuenta activa desde metadata stale ni desde un evento anterior al tombstone sin evidencia canónica posterior.
- [ ] Un test cubre evento de cierre duplicado, cierre observado antes que metadata, reaparición de datos stale y consulta de una cuenta tombstoned.
- [ ] Rent/lamports recuperados por cierre no se presentan como payout, balance contractual o evidencia económica no definida por el contrato.
**Dependencies:** R4/R6/R7; `05-security-tests.md` R4–R5.

### R9: Modos user-signed y server-signed
**Description:** El bridge distingue operaciones firmadas por la wallet del usuario de operaciones autorizadas por un signer del servidor, sin confundir identidad, custodia o permisos.
**Acceptance Criteria:**
- [ ] Cada operación declara explícitamente su modo de firma, signer esperado, autoridad on-chain y actor que la solicita; el modo no se infiere solo desde la ruta API.
- [ ] En `user-signed`, el backend no custodia ni registra la private key y la transacción no se marca enviada/finalizada hasta recibir la firma/autorización verificable requerida.
- [ ] En `server-signed`, solo operaciones permitidas por política y autoridad on-chain pueden usar el signer; una operación de usuario no puede escalarse silenciosamente a server-signed.
- [ ] Tests negativos cubren signer ausente, signer equivocado, intento de usar secretos en DB/logs, cambio de modo durante retry y autorización insuficiente.
**Dependencies:** R2–R3; `01-config-bootstrap.md` R1–R4; `04-deploy-runbook.md` R2–R3.

### R10: Evidencia y hash honesto
**Description:** La evidencia de una operación distingue firma, slot, commitment, payload, hash externo y hash realmente comprometido por el contrato; nunca afirma un hash on-chain inexistente.
**Acceptance Criteria:**
- [ ] Cada evidencia declara tipo, origen, timestamp, relación con operación/transacción y nivel de verificación (`observed`, `confirmed`, `finalized`, `external` o equivalente).
- [ ] Si el contrato no almacena/expone un hash, la API, DB, TUI y documentación lo nombran como hash externo o digest de evidencia y no como “on-chain hash”.
- [ ] Tests de documentación/API fallan ante un campo, etiqueta o mensaje que afirme compromiso on-chain sin instrucción/cuenta/IDL que lo pruebe.
- [ ] La evidencia reproducible no contiene secretos y enlaza commit/IDL/cluster/program ID/transaction signature cuando estén disponibles, diferenciando evidencia actual de histórica o bloqueada.
**Dependencies:** R1/R4/R5/R7; `07-docs-idl-sync.md` R1–R4; `08-final-validation.md` R4.

### R11: Documentación veraz y matriz de trazabilidad
**Description:** Docs, IDL, API, SDK, TUI y reportes describen la misma autoridad, flujo, estados, signer model, proyección y semántica de evidencia.
**Acceptance Criteria:**
- [ ] Existe una matriz `endpoint/comando → application service → SDK operation → instrucción/transacción → fuente de proyección → estado de finality` sin saltos directos API→RPC.
- [ ] Un check de drift falla si docs o ejemplos dicen que DB es fuente contractual, que API llama Anchor/RPC directamente, que existe un estado no soportado o que hay hash on-chain sin respaldo contractual.
- [ ] Reportes de validación etiquetan fecha, commit, ambiente, hash y estado PASS/FAIL/BLOCKED/ACCEPTED; evidencia stale no se presenta como prueba vigente.
- [ ] Los ejemplos de user-signed/server-signed y cuentas cerradas incluyen errores y límites reales; ningún placeholder se presenta como feature implementada.
**Dependencies:** R2–R10; `07-docs-idl-sync.md` R1–R4; `08-final-validation.md` R3–R4.

### R12: Operación MVP acotada y observable
**Description:** El backend puede ejecutar sincronización durable y reconciliación como proceso in-process observable, sin introducir infraestructura distribuida no requerida.
**Acceptance Criteria:**
- [ ] Un smoke/integration test inicia API, application services, SDK fake y worker in-process con límites configurables y cierre limpio.
- [ ] El worker expone métricas/logs sanitizados para último cursor, backlog, retries, divergencias, tombstones y transacciones por estado, sin private keys/tokens.
- [ ] Una configuración inválida o una dependencia blockchain no disponible impide declarar listo el worker y produce un error accionable, no un catch silencioso.
- [ ] La documentación identifica explícitamente los límites MVP y no presenta el worker como indexer general ni garantía de disponibilidad distribuida.
**Dependencies:** R5–R7; R11; `06-reproducibility.md` R1–R4.

## Security Gates
- [ ] **Boundary:** secret scan y dependency-architecture check no encuentran Anchor/RPC usado por API, application services, terminal/TUI, listener o reconciliador fuera de `trust-escrow-sdk`.
- [ ] **Data authority:** tests negativos prueban que DB alterada, stale o divergente no autoriza mutaciones ni reemplaza ownership, balances, estados o finality Solana.
- [ ] **Input/injection:** API, SDK, listener y reconciliador validan formato, límites, cluster/program ID, cursor, firma y idempotency key; no hay SQL/shell crudo ni interpolación insegura.
- [ ] **Secrets/signers:** no private keys, tokens o material sensible en DB, logs, fixtures, errores, reportes o documentación; user-signed y server-signed tienen permisos y pruebas negativas separadas.
- [ ] **Replay/atomicity:** retries, duplicados, concurrencia, eventos repetidos, transacciones pendientes y cierres no producen doble payout, doble efecto ni mutación parcial no documentada.
- [ ] **Finality/reorg:** ningún componente afirma `confirmed`/`finalized` sin evidencia del nivel correspondiente; divergencias, reorgs y tombstones quedan auditados.
- [ ] **Evidence truthfulness:** ningún hash se presenta como on-chain si el contrato/IDL no lo almacena o expone; hashes externos se etiquetan como tales.
- [ ] **Documentation/release:** los gates reutilizados de `07-docs-idl-sync.md` y `08-final-validation.md` bloquean release ante drift, evidencia stale o claims no demostrados.

## Verification Plan
- `yarn build`
- `yarn test`
- `anchor test --provider.cluster localnet`
- `cargo test --workspace`
- `cargo clippy --all-targets --all-features -- -D warnings`
- Test de arquitectura/dependencias para prohibir API/application/worker → Anchor/RPC directo.
- Tests unitarios de SDK fake, application services, idempotencia, estados de finality y validación de evidencia.
- Tests de integración con DB temporal para proyección, cursor, reinicio, deduplicación, reconciliación y tombstones.
- Smoke test del worker in-process con desconexión/retry y cierre limpio.
- Check de drift de docs/IDL/API/SDK y revisión del reporte de validación con evidencia sanitizada.

## Definition of Done (Result Contract)
- [ ] Los 12 requirements tienen criterios automatizables y cada criterio está asignado a un gate.
- [ ] API y terminal/TUI comparten el SDK como única frontera blockchain; no existen accesos directos Anchor/RPC.
- [ ] Solana es canónica para contrato; DB está limitada a proyección, metadata, auditoría y sync, con divergencias reparables y auditadas.
- [ ] Transacciones, eventos, retries, finality, cierres y modos de signer tienen estados y pruebas negativas explícitas.
- [ ] La evidencia y documentación no sobredeclaran hashes, finality, implementación ni validación.
- [ ] No se introduce indexer genérico, microservicio distribuido o event-sourcing fuera del alcance MVP.

## Out of Scope
- Implementar o refactorizar código de producción en Sketch.
- Crear un indexer genérico, microservicios separados, event-sourcing general, bus distribuido o alta disponibilidad multi-región.
- Cambiar el contrato Anchor, su economía, estados contractuales, ownership o reglas de payout para acomodar la proyección.
- Custodia de wallets de usuario, diseño de UX de wallet o proveedor concreto de firma.
- Nuevos endpoints/features de producto, notificaciones, analytics, búsqueda global o migración histórica de fondos.
- Pretender que una DB local, un hash externo, un log o una evidencia stale sea compromiso on-chain.

## Cross-References
- **Reutiliza:** `context/refs/reuse-report.md` (veredicto PARTIAL), [01-config-bootstrap.md](01-config-bootstrap.md), [04-deploy-runbook.md](04-deploy-runbook.md), [05-security-tests.md](05-security-tests.md), [07-docs-idl-sync.md](07-docs-idl-sync.md), [08-final-validation.md](08-final-validation.md).
- **Depende de:** [03-deadlines-auto-approval.md](03-deadlines-auto-approval.md) R1–R5 para estados y autoridad contractual; [06-reproducibility.md](06-reproducibility.md) R1–R4 para toolchain/evidencia reproducible.
- **Extiende:** [07-docs-idl-sync.md](07-docs-idl-sync.md) R1–R4 con bridge, proyección, finality y evidencia; [08-final-validation.md](08-final-validation.md) R1–R4 con gates backend.
- **Relacionado:** [05-security-tests.md](05-security-tests.md) R1–R5 para pruebas negativas, replay, cleanup y conservación.
