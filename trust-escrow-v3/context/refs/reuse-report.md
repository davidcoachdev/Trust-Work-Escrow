# Reuse Report: shared `trust-escrow-sdk` bridge and Solana-authoritative v3 architecture

- Índice escaneado: **9 kits / 0 entradas de `context/refs/kit-index.json`** (`kit-index.json` no existe; se usó el inventario local como fallback).
- Alcance leído: `context/kits/`, `context/impl/`, `context/validation/`, `docs/contract/`, `docs/escenarios/` y patrones documentales/código no mutados de `../trust-escrow-v2/`.
- Términos incluidos: API scaffold, DB persistence, durable listener, reconciliation, idempotency, finality states, signer model, evidence hash contradiction, arbitration fee stale docs, validation evidence stale.

## Kits reutilizables (top-K = 5)

Scores deterministas sobre 0–1: solapamiento con el feature (50%), complementariedad entre dominios (30%) y evidencia/validación existente (20%).

1. **`context/kits/07-docs-idl-sync.md` — 0.94**
   - Reutilizar como gate de contrato documental: IDL, docs, escenarios y hallazgos de auditoría deben coincidir.
   - Aporta el patrón de no aceptar drift silencioso y de clasificar cada hallazgo como remediado, aceptado o gap abierto.
   - Extenderlo para incluir el contrato público del bridge SDK, límites de autoridad Solana/DB y estados de finality.
2. **`context/kits/08-final-validation.md` — 0.92**
   - Reutilizar los gates de build/tests, matriz de estados, replay, deploy reproducible, hashes, upgrade authority, signers y reporte PASS/FAIL/BLOCKED/ACCEPTED.
   - Es el mejor patrón existente para evitar que evidencia histórica o stale se presente como validación actual.
   - Extender el gate de release para bloquear contradicciones entre hash/evidencia y entre estado on-chain y proyección DB.
3. **`context/kits/05-security-tests.md` — 0.86**
   - Reutilizar criterios de autorización, signer/account constraints, atomicidad, replay/idempotency, estados terminales, cleanup de Evidence PDA y conservación económica.
   - Aporta pruebas negativas y de no-mutación parcial aplicables a comandos SDK y endpoints API.
   - Requiere nuevas pruebas específicas para deduplicación de eventos, reintentos del listener y reconciliación.
4. **`context/kits/04-deploy-runbook.md` — 0.81**
   - Reutilizar preflight de endpoint/cluster/program ID, hash, upgrade authority, signer público, pasos idempotentes y evidencia sin secretos.
   - Aporta el modelo de autoridad contractual y la separación explícita de treasury/arbitration treasury.
   - Adaptar el runbook para despliegue/operación del listener durable y para validar que DB nunca sea autoridad de mutaciones.
5. **`context/kits/01-config-bootstrap.md` — 0.73**
   - Reutilizar autoridad inicial fija, inicialización única, anti-frontrun, multisig como autoridad efectiva, rotación controlada y validación de fees/destinos.
   - Sirve como base del signer model del bridge y de sus precondiciones de configuración.
   - No cubre todavía separación de signer de lectura, signer de envío, signer multisig ni políticas de custodia del backend.

## Docs y patrones reutilizables

### v3

- `docs/contract/01-overview.md`, `03-estado.md`, `04-config.md`, `06-disputes.md`, `08-arbiter-pool.md`: autoridad contractual, estados Job/Dispute, Evidence PDA individual, `arbitration_treasury` separado y resolutor que solo firma/autoriza.
- `docs/contract/09-auditoria.md`: fuente de decisiones de remediación sobre resolver, payout, evidencia y treasury; debe tratarse como baseline histórico, no como prueba vigente.
- `context/validation/coverage-matrix.md` y `context/validation/final-report.md`: reutilizar formato de evidencia y estados, pero corregir la contradicción entre `APPROVE` histórico y evidencia local actual `BLOCKED`.
- `context/impl/trust-escrow-v3.md`: reutilizar la lista de blockers conocidos: replay/idempotency parcial, runtime bloqueado, autoridad/advisor persistente no disponible y expiración/carga de Surfpool.

### v2

- `../trust-escrow-v2/sdk/src/client.rs`, `events.rs`, `types.rs`, `lib.rs`: patrón de cliente de alto nivel typed sobre Solana, `Arc<RpcClient>`, `Arc<dyn Signer + Send + Sync>`, commitment configurable, retry/cache y tipos/eventos compartidos. Es la referencia principal para el contrato del `trust-escrow-sdk`, no para copiar implementación.
- `../trust-escrow-v2/sdk/tests/unit/*` y `sdk/tests/integration/escrow_flow_test.rs`: reutilizar cobertura de operaciones, firmas válidas, errores sin panic, workflows completos, recuperación y escenarios concurrentes como acceptance criteria.
- `../trust-escrow-v2/docs/planning/epic-cli-tui/design.md`: patrón explícito de **shared core module** entre CLI/TUI, wrapper del SDK, configuración jerárquica, separación presentación/lógica y flujo CLI/TUI → core → SDK → Solana.
- `../trust-escrow-v2/docs/planning/epic-cli-tui/specs.md`: requisitos de comandos, dashboards, actualizaciones en tiempo real, navegación, wallet ownership, errores accionables y acceso común a operaciones SDK.
- `../trust-escrow-v2/docs/planning/epic-cli-tui/phases/01-foundation.md` y `05-integration-testing.md`: patrón de wrapper compartido, runtime async con canales para TUI y pruebas unitarias con mock SDK más integración/E2E, retries, timeouts, signing y confirmation.
- `../trust-escrow-v2/docs/architecture/DATABASE_SCHEMA.md`: patrón útil de DB como cache/sync de User/Job PDA, más metadata, audit logs (`tx_signature` único), ledger y API logs. Debe reutilizarse solo como proyección/auditoría; `wallet_principal`/on-chain y estados contractuales requieren una definición de precedencia explícita.

## Gaps que deben crearse o resolverse

- **Bridge contractual ausente:** no existe una especificación v3 de `trust-escrow-sdk` compartida por terminal/TUI y backend API: operaciones, tipos, errores, eventos, versionado y límites del bridge.
- **API scaffold ausente:** no hay contrato API v3 que consuma el mismo bridge ni matriz endpoint → operación SDK → transacción Solana → proyección DB.
- **DB persistence/projection boundary ausente:** falta definir tablas/colecciones, claves on-chain, metadata mutable, audit trail, retention, índices y regla formal: DB no puede autorizar ni reemplazar estado contractual Solana.
- **Durable listener ausente:** faltan cursor/checkpoint, reanudación, backoff, deduplicación, ordenamiento, durable event store y manejo de forks/reorgs o pérdida de conexión.
- **Reconciliation ausente:** falta un proceso verificable para comparar Solana contra DB, detectar divergencias, reparar proyecciones sin mutar el contrato y emitir evidencia auditable.
- **Idempotency incompleta:** el baseline v3 marca replay/idempotency como `PARTIAL`; faltan claves idempotentes para requests API, transacciones, eventos y reintentos del listener.
- **Finality states ausentes:** falta un vocabulario único (por ejemplo submitted/processed/confirmed/finalized/failed/reorged) y su mapeo en SDK, API, TUI, listener, DB y documentación.
- **Signer model incompleto:** los kits cubren autoridad on-chain y signers de deploy, pero no roles/credenciales del SDK, backend signer, wallet interactiva, multisig, read-only RPC y prohibición de custodiar secretos en DB/logs.
- **Evidence hash contradiction:** el runbook exige hash SHA-256 y evidencia reproducible, mientras `final-report.md`/`impl` mezclan PASS histórico, validator ausente y `APPROVE`; falta una fuente única que distinga evidencia histórica, actual y bloqueada.
- **Arbitration fee stale docs:** hay múltiples documentos que fijan 5% y `arbitration_treasury`, pero el reporte debe validar que no quede wording antiguo de resolver fee/destino y que la definición vigente sea única en contrato, IDL, SDK, API y escenarios.
- **Validation evidence stale:** `coverage-matrix.md` y `final-report.md` contienen estados distintos entre runtime actual bloqueado y evidencia histórica aprobada; falta freshness/commit/hash/fecha obligatoria y bloqueo de release ante evidencia no reproducible.
- **Kit registry gap:** `context/refs/kit-index.json` no existe; el siguiente ciclo debería regenerarlo para que Retrieve sea index-driven y reproducible.

## Security gates para el siguiente Sketch

- [ ] No secretos, private keys ni tokens en índice, docs, fixtures, DB projection o logs.
- [ ] Toda entrada de API/listener/reconciliation valida formato, límites, cursor y firmas; no shell/SQL crudo.
- [ ] Solana/IDL/commit/hash/endpoint son evidencia de autoridad; DB solo proyección, metadata y auditoría.
- [ ] Cada requisito nuevo tiene acceptance criteria automatizable para API, SDK, listener, reconciliación, idempotencia, finality, signer y documentación.

## Veredicto

**PARTIAL** — hay reutilización fuerte para autoridad Solana, signers administrativos, estados contractuales, Evidence/fees, validación y el patrón v2 de shared core SDK para CLI/TUI. No existe todavía el puente v3 ni la capa API/listener/projection/reconciliation/finality que el feature exige, y la evidencia de validación actual contiene estados históricos stale que deben resolverse antes de declarar alineamiento.
