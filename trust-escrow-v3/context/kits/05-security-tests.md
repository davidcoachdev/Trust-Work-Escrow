# Cavekit R5: Tests de seguridad y cobertura de estados

## Goal
Convertir cada hallazgo de seguridad y cada transición crítica en pruebas negativas, de integración y de invariantes económicas.

## Constraints
- Calidad: cobertura determinista de estados, errores, cleanup, replay y balances.
- Seguridad: probar rechazo, no solo happy path; conservar fondos y autoridad.
- Strict TDD: cada regresión empieza con test fallido.

## Requirements

### R1: Autorización y destinos
**Description:** Las pruebas demuestran que ningún signer o cuenta sustituta puede ejecutar administración o desviar payouts.
**Acceptance Criteria:**
- [ ] Tests negativos cubren Config, ArbiterPool, advisor, cliente, freelancer, resolver y treasury incorrectos.
- [ ] `approve_work`, milestone, auto-aprobación y dispute payout rechazan freelancer/destino no ligado al job.
- [ ] Las cuentas de fee incorrectas no reciben fondos y la transacción no deja mutaciones parciales.
- [ ] Cada error negativo verifica código/causa y balances/estado sin cambios.
**Dependencies:** R1 R1–R4, R2 R1–R4, R3 R3.

### R2: Máquina de estados y disputas
**Description:** Cada transición válida e inválida del Job, Milestone, Dispute, SupportTicket y Evidence queda cubierta.
**Acceptance Criteria:**
- [ ] Tests cubren `Created`, `Funded`, `InProgress`, `Submitted`, `Released`, `Cancelled` y estados de disputa actuales.
- [ ] El flujo `submit_work` deja el Job en `Submitted` sin introducir ni aceptar un estado separado `Received`.
- [ ] Cada instrucción crítica falla fuera de su estado permitido y conserva los campos previos.
- [ ] `pause_job` tiene éxito solo en `Created` o `Funded` con `job.freelancer == None`; falla sin mutación con freelancer asignado, en progreso o en `Submitted`.
- [ ] Un intento de `pause_job` sobre `Submitted` no detiene ni extiende el timer de auto-aprobación.
- [ ] Disputa abierta bloquea auto-aprobación y milestone payout; resolución/cancelación permite solo el terminal previsto.
- [ ] No se puede asignar árbitro a partes ni reabrir una disputa resuelta.
**Dependencies:** R2 R3, R3 R2–R4, R7 R2.

### R3: Replay, concurrencia y clock
**Description:** Los tests prueban idempotencia práctica y carreras entre acciones del cliente, keeper, disputa y cancelación.
**Acceptance Criteria:**
- [ ] Repetir cada instrucción terminal no duplica payout, cierre, evidencia ni evento lógico.
- [ ] Ejecutar acciones competidoras en orden alternado produce un único estado terminal válido o errores explícitos.
- [ ] Se prueban deadline antes, exacto y después de 7 días, incluyendo overflow y timestamp negativo/limítrofe.
- [ ] La suite no depende de sleeps arbitrarios ni de una RPC externa.
**Dependencies:** R3 R1–R5, R6 R3.

### R4: Evidence PDAs y cleanup
**Description:** Las evidencias tienen límites, ownership, asociación al job/dispute y cierre determinista.
**Acceptance Criteria:**
- [ ] Crear evidencia acepta solo payload no vacío dentro del límite vigente de 2.048 bytes y rechaza overflow.
- [ ] El máximo vigente de 10 evidencias se aplica por disputa/job y el intento número 11 falla sin mutación.
- [ ] Evidencia de otra disputa/job o signer no autorizado es rechazada.
- [ ] Todos los caminos terminales y cleanup verifican que no queden Evidence PDAs huérfanas, salvo retención explícitamente especificada.
**Dependencies:** R3 R4, R7 R2.

### R5: Payouts y conservación
**Description:** Las pruebas verifican montos exactos, fees separadas, shortfall y cierre de cuentas.
**Acceptance Criteria:**
- [ ] Happy paths y fallos parciales verifican balances exactos del cliente, freelancer, treasury y arbitration treasury.
- [ ] Auto-aprobación paga amount restante + fee de protocolo; disputa conserva `fee + cliente + freelancer + shortfall = amount + fee`.
- [ ] Milestones ya pagados no se pagan nuevamente en approve, cancelación o disputa.
- [ ] Cerrar una cuenta no permite que la rent o lamports reaparezcan como payout no especificado.
**Dependencies:** R3 R3–R4, R2 R4, R7 R2.

### R6: Seguridad y cobertura de Applications PDAs
**Description:** La suite prueba que el modelo de postulaciones individuales mantiene aislamiento entre Jobs, límites estrictos, permisos, unicidad, máximo 50 y cleanup/rent durante todo el ciclo de vida.
**Acceptance Criteria:**
- [ ] `create_job` se prueba con el modelo sin cuenta `Applications` inline sobredimensionada y queda verificable que el Job no reserva espacio para 50 objetos completos.
- [ ] Una aplicación válida crea la PDA determinista esperada; una PDA con Job, índice, applicant, seed o bump incorrectos es rechazada y no deja cuenta utilizable.
- [ ] Se rechazan applicant ausente/no signer, applicant igual al cliente, Job inexistente, Job en estado no postulable y permiso de cliente inválido.
- [ ] Se rechazan índice negativo representado, índice fuera de `0..49`, índice repetido y cualquier intento de superar exactamente 50 postulaciones; cada caso verifica estado, cuentas y balances sin cambios.
- [ ] Se rechazan propuesta vacía o mayor al límite, y texto de Job fuera de los límites; los tests verifican que la validación usa tamaño determinista y no trunca silenciosamente.
- [ ] Un applicant duplicado para el mismo Job es rechazado aunque use otro índice; la postulación original permanece intacta.
- [ ] `accept_application` rechaza una Application de otro Job, índice incorrecto, estado no Pending o signer distinto del cliente, sin asignar freelancer ni cambiar el Job.
- [ ] Los caminos accepted/rejected/withdrawn y el cierre terminal del Job verifican la política de cleanup: PDA cerrada o retenida explícitamente, rent al destinatario correcto y ninguna PDA huérfana ni payout de rent.
**Dependencies:** R3 R6, R7 R5.

## Security Gates
- [ ] Cobertura negativa existe para cada autoridad/destino/account constraint crítico.
- [ ] Hay aserciones de balances y estado antes/después, no solo `expect(tx).not.be.null`.
- [ ] No hay tests que oculten errores, usen secretos reales o dependan de devnet.
- [ ] Las pruebas de Applications comprueban derivación determinista, ownership, límites 50, duplicados, permisos y rent/cleanup, no solo el estado del Job.
- [ ] Los tests de replay/concurrencia son repetibles en localnet.

## Verification Plan
- `yarn test`
- `anchor test --provider.cluster localnet`
- Reporte de cobertura de tests y matriz de estados/errores.
- `cargo clippy --all-targets --all-features -- -D warnings`

## Out of Scope
- Fuzzing formal, auditoría externa o pruebas de carga de producción.
- Cambiar el modelo económico para hacer pasar un test.
- Tests de UI o indexador no presentes en este programa.
- Cambiar la política de retención de postulaciones o agregar ranking, matching automático o más de 50 postulaciones.

## Cross-References
- **Cubre:** todos los kits R1–R4 y R6–R7.
- **Relacionado:** [08-final-validation.md](08-final-validation.md) R1–R4.
- **Relacionado:** [03-deadlines-auto-approval.md](03-deadlines-auto-approval.md) R6; [07-docs-idl-sync.md](07-docs-idl-sync.md) R5.
