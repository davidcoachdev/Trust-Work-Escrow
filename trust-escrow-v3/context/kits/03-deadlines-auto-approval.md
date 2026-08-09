# Cavekit R3: Submitted, deadlines y auto-aprobación

## Goal
Definir `Submitted` como la entrega realizada por `submit_work` y aplicar on-chain una ventana determinista de 7 días desde `submitted_at`, protegiendo la revisión del cliente y evitando fondos bloqueados.

## Constraints
- Calidad: reglas basadas en `Clock`, límites exactos y transiciones atómicas.
- Seguridad: una disputa abierta bloquea auto-aprobación; `pause_job` solo opera antes de asignar freelancer y nunca detiene el timer de `Submitted`; no hay replay ni doble payout.
- Strict TDD: pruebas de boundary clock y estados RED antes del cambio.

## Requirements

### R1: Registro y cálculo del deadline
**Description:** `submit_work` registra una entrega y transiciona el Job a `Submitted`; no existe un estado separado `Received`. El deadline efectivo es `submitted_at + 7 días`, sin depender de la hora de una transacción posterior.
**Acceptance Criteria:**
- [ ] Tras `submit_work`, el Job queda en `Submitted`, `submitted_at` es observable y el deadline calculado equivale exactamente a 604800 segundos después.
- [ ] No existe una transición ni un estado separado `Received` entre `submit_work` y `Submitted`.
- [ ] Antes del deadline, una llamada de auto-aprobación falla y no paga ni cierra el job.
- [ ] En el instante límite se prueba y documenta la convención inclusiva/exclusiva elegida; el resultado es determinista.
- [ ] Después del deadline, con estado `Submitted` y sin excepción, la llamada válida puede completar el flujo una sola vez.
**Dependencies:** R1 R2–R3.

### R2: Ventana de acción del cliente
**Description:** Durante los 7 días el cliente conserva las acciones de aprobar, rechazar o abrir disputa.
**Acceptance Criteria:**
- [ ] Aprobar dentro de la ventana paga `amount` restante al freelancer, `fee_amount` a `treasury`, cierra `Job` y no puede repetirse.
- [ ] Rechazar dentro de la ventana vuelve al estado de trabajo permitido sin payout ni cierre indebido.
- [ ] Abrir disputa dentro de la ventana crea la disputa y bloquea la auto-aprobación posterior.
- [ ] Una acción del cliente que no corresponde al estado actual es rechazada sin transferencia parcial.
**Dependencies:** R1, R3.

### R3: Auto-aprobación con payout y cleanup
**Description:** Al vencer la ventana sin acción y sin disputa, el programa ejecuta el payout confirmado y cierra el job.
**Acceptance Criteria:**
- [ ] Con `Submitted`, deadline vencido y sin disputa, el freelancer recibe exactamente el monto restante y `treasury` exactamente `fee_amount`.
- [ ] El `Job` queda cerrado y su rent vuelve al destinatario definido; no quedan fondos del escrow atrapados.
- [ ] La auto-aprobación usa el destino de freelancer ligado al job y el treasury ligado a `Config`.
- [ ] Dos llamadas posteriores o concurrentes no generan doble payout ni reabren el PDA.
**Dependencies:** R1–R2, R1 R3, R7 R2.

### R4: Excepciones de disputa, cancelación y pausa
**Description:** Las disputas y terminales bloquean la auto-aprobación; `pause_job` solo puede ejecutarse antes de asignar freelancer y no puede detener una ventana ya iniciada.
**Acceptance Criteria:**
- [ ] Una disputa abierta, activa o con evidencia impide auto-aprobar aunque el deadline de submission haya vencido.
- [ ] Un job cancelado o resuelto por cancelación no puede auto-aprobar ni pagar dos veces.
- [ ] `pause_job` solo tiene éxito en `Created` o `Funded` cuando `job.freelancer == None`.
- [ ] `pause_job` falla sin mutación si el Job tiene freelancer asignado, está en progreso o está en `Submitted`; en particular, no puede detener ni extender el timer de auto-aprobación.
**Dependencies:** R3, R5 R3–R4, R7 R2.

### R5: Clock, overflow y replay
**Description:** Los cálculos temporales son seguros frente a clocks límite, timestamps inválidos y replay.
**Acceptance Criteria:**
- [ ] Timestamps en límite inferior, exacto y superior de la ventana tienen resultados esperados y documentados.
- [ ] Overflow al sumar 7 días falla sin mutar estado ni mover fondos.
- [ ] Una instrucción repetida con el mismo job/disputa devuelve error de estado y no duplica eventos/payouts.
- [ ] Los tests usan una fuente de tiempo controlable en localnet o un método reproducible equivalente, sin depender de sleeps frágiles.
**Dependencies:** R3–R4, R6 R3.

### R6: Postulaciones en PDAs individuales
**Description:** Cada Job puede recibir hasta 50 postulaciones independientes en total, y cada postulación se identifica de forma determinista y se valida durante `create_job`, `apply_to_job` y `accept_application` sin almacenar una colección inline de tamaño no acotado en el Job.
**Acceptance Criteria:**
- [ ] `create_job` crea un Job válido sin requerir una cuenta agregada de postulaciones sobredimensionada; el estado inicial indica cero postulaciones aceptadas/activas según el modelo documentado.
- [ ] Una `apply_to_job` válida crea exactamente una PDA individual determinista para el Job e índice dados; derivar nuevamente con los mismos datos produce la misma dirección y no crea una segunda postulación.
- [ ] El índice de postulación solo puede estar dentro del rango permitido para las 50 posiciones (`0..49`); un índice inválido, reutilizado o posterior al máximo falla sin mutar Job ni cuentas.
- [ ] La instrucción rechaza un `applicant` que no sea el signer o que coincida con el cliente del Job; también rechaza una PDA de aplicación asociada a otro Job o a otro índice.
- [ ] El texto de Job y de la propuesta cumple los límites de tamaño y las reglas de vacío definidas; entradas fuera de esos límites fallan antes de crear o mutar la PDA.
- [ ] Un applicant no puede crear dos postulaciones para el mismo Job aunque intente cambiar el índice o la PDA; el intento duplicado falla de forma determinista.
- [ ] `accept_application` solo acepta una postulación Pending perteneciente al Job y al índice solicitado, con permiso del cliente; una cuenta, applicant, Job, índice o estado incompatibles falla sin mutación parcial.
- [ ] Cada transición terminal de la postulación y cada cierre del Job aplica la política documentada de retención o cierre de su PDA: la rent vuelve únicamente al destinatario definido, no quedan PDAs huérfanas y no se convierte rent en payout.
**Dependencies:** R1, R2, R3, [07-docs-idl-sync.md](07-docs-idl-sync.md) R5.

## Security Gates
- [ ] No se acepta `submitted_at` provisto libremente por el usuario.
- [ ] Todo payout valida cliente, freelancer, treasury, estado y PDA antes de transferir.
- [ ] Disputa y cancelación son bloqueos explícitos, no solo convenciones de cliente.
- [ ] No hay doble cierre, doble transferencia ni `saturating` que oculte déficit económico.
- [ ] Seeds, bump, ownership, signer, Job, índice y applicant de cada Application PDA se validan antes de crear, aceptar o cerrar la cuenta.
- [ ] El máximo 50 y los límites de texto se validan on-chain; no dependen de filtros del cliente.

## Verification Plan
- `yarn test`
- `anchor test --provider.cluster localnet`
- Tests de boundary clock, disputa, cancelación, pausa, auto-payout, replay y ciclo completo de Application PDA.
- Inspección de balances y cuentas antes/después de cada escenario.

## Out of Scope
- Cambiar la duración confirmada de 7 días.
- Recordatorios, notificaciones, cron externo o keeper propietario como requisito de producto.
- Pausar un Job con freelancer asignado o pausar `Submitted` para detener/extender el timer de auto-aprobación.
- Diseñar expiración, recordatorios o reanudación de pausas más allá de la regla de autorización de `pause_job`.
- Agregar más de 50 postulaciones, una colección inline de postulaciones en el Job o un flujo de ranking/selección automática.

## Cross-References
- **Depende de:** [01-config-bootstrap.md](01-config-bootstrap.md) R3; [07-docs-idl-sync.md](07-docs-idl-sync.md) R1–R2.
- **Relacionado:** [05-security-tests.md](05-security-tests.md) R2–R5.
- **Verificado por:** [08-final-validation.md](08-final-validation.md) R2.
- **Relacionado:** [05-security-tests.md](05-security-tests.md) R6; [07-docs-idl-sync.md](07-docs-idl-sync.md) R5.
