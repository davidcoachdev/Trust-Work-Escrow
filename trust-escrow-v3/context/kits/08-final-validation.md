# Cavekit R8: Validación final y security gates

## Goal
Impedir que una remediación de auditoría se declare lista sin evidencia técnica completa, reproducible y revisable.

## Constraints
- Calidad: gates ordenados, evidencia trazable y reporte honesto.
- Seguridad: cualquier fallo crítico bloquea release; no se ignoran errores ambientales sin clasificarlos.
- Strict TDD: los gates incluyen regresiones automatizadas y no reemplazan tests por inspección manual.

## Requirements

### R1: Gate de compilación, lint y tests
**Description:** La suite completa pasa en orden y los resultados quedan registrados.
**Acceptance Criteria:**
- [ ] `yarn build` termina con exit code 0.
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` termina con exit code 0.
- [ ] `yarn test` termina sin fallos ni tests omitidos no justificados.
- [ ] `anchor test --provider.cluster localnet` termina con la suite funcional y negativa completa.
**Dependencies:** R5 R1–R5, R6 R1–R4.

### R2: Gate funcional de invariantes
**Description:** La evidencia final demuestra autoridad, deadlines, disputas, cleanup, evidencias y payouts correctos.
**Acceptance Criteria:**
- [ ] La matriz de estados muestra todos los caminos válidos e inválidos y referencia su test.
- [ ] El caso de 7 días verifica `submit_work → Submitted` sin `Received`, window actions, disputa bloqueante, rechazo de `pause_job` fuera de `Created`/`Funded` sin freelancer, auto-payout exacto y cierre.
- [ ] La matriz de cuentas demuestra que no hay PDA huérfana ni doble payout después de caminos terminales/replay.
- [ ] La conservación económica de protocolo y arbitraje pasa con milestones, disputa unilateral/mutua y shortfall.
**Dependencies:** R3 R1–R5, R5 R2–R5, R7 R1–R3.

### R3: Gate de deploy y reproducibilidad
**Description:** El artefacto final puede ser desplegado/verificado por un operador distinto en el entorno elegido.
**Acceptance Criteria:**
- [ ] El runbook pasa preflight y verifica endpoint, program ID, hash, upgrade authority y signers.
- [ ] `Config` se inicializa/verifica con la autoridad/advisor/treasuries aprobados y no se re-inicializa.
- [ ] La ejecución localnet es repetible desde estado limpio y no usa secretos versionados.
- [ ] Toda diferencia entre artefacto local y on-chain bloquea el resultado como FAIL.
**Dependencies:** R1 R1–R4, R4 R1–R4, R6 R2–R3.

### R4: Reporte de gaps y decisión de release
**Description:** El reporte final clasifica cada hallazgo y requirement sin afirmar remediación no probada.
**Acceptance Criteria:**
- [ ] Cada hallazgo de auditoría tiene estado PASS, FAIL, BLOCKED o ACCEPTED explícito con evidencia y responsable siguiente.
- [ ] Los gaps de especificación permanecen visibles y no se transforman silenciosamente en requisitos inventados.
- [ ] Un FAIL de seguridad, autoridad, payout, replay, deadline, secreto o reproducibilidad bloquea release.
- [ ] El reporte lista archivos, comandos, versiones, hashes y fecha de ejecución suficientes para repetir la validación.
**Dependencies:** R1–R3, todos los kits.

### R5: Gate de postulaciones y ciclo de vida de PDAs
**Description:** La evidencia final demuestra que el Job soporta exactamente el modelo aprobado de hasta 50 postulaciones individuales deterministas, con validaciones completas y cleanup/rent seguro.
**Acceptance Criteria:**
- [ ] La matriz funcional prueba `create_job`, `apply_to_job` y `accept_application` con PDA individual, índice, applicant, permisos y relación correcta con Job.
- [ ] La suite demuestra cero, una y exactamente 50 postulaciones válidas; la número 51, índices inválidos y duplicados fallan sin mutación parcial.
- [ ] La suite demuestra rechazo de texto vacío/excesivo y de cualquier cuenta, signer, Job o PDA cruzada, con códigos de error y balances sin cambios.
- [ ] La evidencia de ciclo de vida demuestra cleanup/retención explícita, destinatario de rent y ausencia de PDAs huérfanas o payout de rent no especificado.
- [ ] El IDL, docs, tests y escenarios coinciden en seeds/constraints, instrucciones, límites y estados; cualquier referencia al modelo inline anterior bloquea el gate.
**Dependencies:** R3 R6, R5 R6, R7 R5.

### R6: Gate de arquitectura backend v3 y sincronización
**Description:** La validación final demuestra que el backend respeta la frontera SDK, la autoridad Solana y la proyección durable sin claims de evidencia no probados.
**Acceptance Criteria:**
- [ ] El análisis de dependencias no encuentra API, application service, terminal/TUI, listener o reconciliador llamando Anchor/RPC fuera de `trust-escrow-sdk`.
- [ ] La matriz de ownership demuestra que DB solo proyecta/metadata/auditoría/sync y que una divergencia no autoriza mutaciones ni reemplaza estado, ownership, balances o finality on-chain.
- [ ] Tests de reinicio, cursor, duplicados, retries, idempotency keys, finality y tombstones pasan sin doble payout, doble efecto o recreación de cuentas cerradas.
- [ ] Tests de signer cubren user-signed/server-signed sin secretos persistidos y con autorización explícita.
- [ ] El check documental falla si evidencia externa se etiqueta como hash on-chain o si evidencia stale/bloqueada se presenta como validación vigente.
**Dependencies:** `backend-v3-sketch.md` R1–R12, R7 R6.

## Security Gates
- [ ] SAST/secret scan sin findings críticos o altos no aceptados explícitamente.
- [ ] Authorization/account constraints revisados con tests negativos.
- [ ] Input bounds, overflow, replay, cleanup y conservation checks PASS.
- [ ] Application PDA seeds, ownership, applicant/Job/index constraints, máximo 50 y rent/cleanup checks PASS.
- [ ] Upgrade authority, program ID, hash, endpoint y Config authority verificados desde fuente confiable.
- [ ] No se declara DONE con tests bloqueados por ambiente sin reportar BLOCKED.
- [ ] Boundary SDK, DB authority, finality/reconciliation, tombstones, signer modes y evidence truthfulness están enlazados a evidencia por requirement.

## Verification Plan
- Ejecutar, en orden, los comandos de R1.
- `surfpool ls` y `surfpool run deployment` con evidencia sanitizada.
- Revisión manual de matrices, diffs de IDL/docs y hallazgos de auditoría.
- Generar reporte final enlazando tests y artefactos por requirement.

## Out of Scope
- Aprobar excepciones de riesgo sin autoridad humana explícita.
- Crear un proceso de certificación externo.
- Hacer commit, merge o release durante Sketch.

## Cross-References
- **Consolida:** [01-config-bootstrap.md](01-config-bootstrap.md) a [07-docs-idl-sync.md](07-docs-idl-sync.md).
- **Siguiente fase:** `/sdd-cavekit map` debe derivar tasks una por cada requirement y security gate aplicable.
- **Relacionado:** [backend-v3-sketch.md](backend-v3-sketch.md) R1–R12; [07-docs-idl-sync.md](07-docs-idl-sync.md) R6.
