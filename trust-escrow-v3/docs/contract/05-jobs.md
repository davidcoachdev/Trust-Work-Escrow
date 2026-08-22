# 05 · Módulo Jobs

Estado: ✅ implementado completo (create_job, deposit_funds, apply_to_job,
accept_application, submit_work, approve_work, reject_work, cancel_job,
pause_job, unpause_job, expire_paused_job, auto_approve_work).

> **Pausa por job:** `deposit_funds` y `accept_application` quedan bloqueados si el job
> está pausado (`check_not_paused`). La pausa solo la puede poner el cliente y
> **solo si no hay freelancer asignado** (`Created`/`Funded`). No puede durar
> para siempre: tras `MAX_PAUSE_DURATION` (30 días) cualquiera puede llamar
> `expire_paused_job` para reembolsar y cerrar. Ver `09-auditoria.md`.

## `create_job`

**Qué hace**
Crea el PDA `Job` (seed `[b"job", client, job_id]`) en estado `Created`. Calcula
la fee de plataforma con `compute_fee(amount, fee_bps)` (aritmética chequeada) y
la guarda en `fee_amount`. Inicializa los contadores de milestones en 0 y
`applicants` vacío. No reserva una cuenta de colección de postulaciones.

**Por qué**
- La fee se calcula **una sola vez** aquí y se guarda, evitando recalcular y
  desincronizar (bug de v2 donde se recalculaba en pagos con base inconsistente).
- `deadline` debe ser futuro para que el auto-aprueba por grace tenga sentido.
- No asigna árbitro (corregido por diseño: el árbitro es neutral y se asigna solo
  al abrir disputa).

**Validaciones**
- `!config.paused` → `ProgramPaused`
- `amount >= MIN_JOB_AMOUNT` → `AmountTooSmall`
- `!title.is_empty()` → `EmptyTitle`
- `title.len() <= MAX_TITLE_LENGTH` → `TitleTooLong`
- `description.len() <= MAX_DESCRIPTION_LENGTH` → `DescriptionTooLong`
- `deadline > now` → `DeadlineMustBeFuture`

**Cuentas**: `client` (Signer, paga), `job` (init PDA), `config`, `system_program`.

## `apply_to_job`

**Qué hace**
Un freelancer se postula al job fondeado creando una PDA individual
`[b"application", job, application_index, applicant]` con
`Application { job, index, applicant, proposal, applied_at, status: Pending }`.
El Job solo conserva `applicants` para contar hasta `MAX_APPLICATIONS` = 50 y
rechazar duplicados; el job queda en `Funded`.

**Por qué**
Modelo de postulaciones múltiples: el cliente elige luego con `accept_application`
(en vez del auto-aceptar directo). Separar cada postulación mantiene
`create_job` por debajo del límite de asignación de cuentas de Anchor/Solana.

**Validaciones**
- `job.status == Funded` → `InvalidJobStatus`
- `applicant.key() != job.client` → `CannotWorkOnOwnJob`
- `!proposal.is_empty()` → `EmptyProposal`
- `proposal.len() <= MAX_PROPOSAL_LENGTH` → `ProposalTooLong`
- `application_index == applicants.len()` y longitud menor que 50
  → `ApplicationIndexMismatch` / `InvalidApplicationIndex`
- no postularse dos veces (`applicant` ya en `Job.applicants`) → `AlreadyApplied`

**Cuentas**: `applicant` (Signer, paga), `client` (UncheckedAccount), `job`
(mut PDA), `application` (init PDA individual), `system_program`.

## `accept_application`

**Qué hace**
El cliente elige una postulación por índice y applicant: valida la PDA
individual, marca `Application` como `Accepted`, setea
`job.freelancer = Some(applicant)` y pasa el job a `InProgress`. La cuenta de
Application se cierra en la misma transacción y su rent vuelve al applicant.

**Por qué**
Asigna quién entregará el trabajo (requisito para `submit_work`/`approve_work` y
para abrir disputas). Reemplaza al auto-aceptar directo: el cliente elige entre
las postulaciones recibidas.

**Validaciones**
- `job.status == Funded` → `InvalidJobStatus`
- `check_not_paused` → `JobPaused` / `JobPausedExpired`
- `job.client == firmante` → `NotJobClient`
- seeds `[job, application_index, applicant]`, `application.job`, `index`,
  `status == Pending` y correspondencia con `Job.applicants` → errores de PDA,
  índice o estado

**Cuentas**: `client` (Signer), `job` (mut PDA), `applicant` (SystemAccount),
`application` (mut PDA, `close = applicant`).

## `deposit_funds`

**Qué hace**
El cliente transfiere `amount + fee_amount` desde su wallet al PDA `job`. Pasa el
job a `Funded`.

**Por qué**
- Es una transferencia normal firmada por el cliente (el PDA `job` es el
  destino, no el origen → no necesita `new_with_signer`).
- El principal queda custodiado; de aquí se pagará al freelancer y la fee a
  `treasury` al aprobar (o se reparte en disputa).
- `checked_add` evita desbordamiento silencioso en el total.

**Validaciones**
- `job.status == Created` → `InvalidJobStatus`
- `job.client == firmante` → `NotJobClient`

**Cuentas**: `client` (Signer), `job` (mut PDA), `config`, `system_program`.

## `submit_work`

**Qué hace**
El freelancer marca el trabajo como entregado (`Submitted`) y registra
`submitted_at`.

**Por qué**
Requiere `job.freelancer == firmante` y estado `InProgress` (asignado y en
curso). Transición clara hacia la etapa de revisión del cliente.

**Validaciones**
- `job.freelancer == firmante` → `NotJobFreelancer`
- `job.status == InProgress` → `InvalidJobStatus`

**Cuentas**: `freelancer` (Signer), `client` (UncheckedAccount, validado por PDA), `job` (mut PDA).

## `approve_work`

**Qué hace**
El PDA `job` firma (`new_with_signer`) y paga el **resto** (`amount -
milestones_amount_total`) al freelancer y `fee_amount` a `treasury`. Job →
`Released` y se cierra (`close = client`, renta devuelta).

**Por qué**
- Corrige el bug de v2: aquí el PDA `job` **firma** la transferencia de salida
  (v2 usaba `CpiContext::new` y fallaba en runtime).
- La fee de plataforma se enruta a `treasury` (v2 la mandaba al PDA `config`).
- `close = client` devuelve la renta; no quedan fondos atrapados.
- Si el job tiene milestones, paga solo el resto y exige que estén todos
  aprobados (evita pagar dos veces lo ya liberado por milestones).

**Validaciones**
- `job.client == firmante` → `NotJobClient`
- `job.status == Submitted` → `InvalidJobStatus`
- `job.freelancer.is_some()` → `NoFreelancerAssigned`
- `treasury.key() == config.treasury` → `InvalidTreasury`
- `freelancer.key() == job.freelancer` → `NotJobFreelancer` (evita desviar el pago)

**Cuentas**: `client` (Signer), `job` (mut PDA, close), `freelancer` (SystemAccount),
`treasury` (UncheckedAccount + constraint), `config`, `system_program`.

## `auto_approve_work`

Un keeper cualquiera puede ejecutar el payout cuando `now >= submitted_at +
604800` y el Job sigue en `Submitted`. Paga exactamente el principal restante al
freelancer ligado al Job, `fee_amount` a `Config.treasury` y devuelve la rent al
cliente antes de cerrar el PDA. La existencia de cualquier `Dispute` PDA bloquea
la operación. Repetirla falla por estado y no puede producir doble payout.

## `reject_work`

**Qué hace**
El cliente rechaza el trabajo entregado; job vuelve a `InProgress` para que el
freelancer corrija y reenvíe.

**Por qué**
Permite iteración sin abrir disputa. (En v3 no guarda razón; el parámetro
`reason` se ignora por ahora; se puede añadir campo luego.)

**Validaciones**
- `job.client == firmante` → `NotJobClient`
- `job.status == Submitted` → `InvalidJobStatus`

**Cuentas**: `client` (Signer), `job` (mut PDA).

## `cancel_job`

**Qué hace**
El cliente cancela el job (solo antes de asignar freelancer):
- `Created`: solo cierra (nada depositado).
- `Funded`: el PDA `job` firma y reembolsa `amount + fee_amount` al cliente.

Job → `Cancelled` y cierra (`close = client`, renta devuelta).

**En `InProgress` no se cancela solo.** Si el freelancer no entregó / no cumplió, el
cliente (o el freelancer) abre un **`SupportTicket`** (ver `06-disputes.md`) y el
asesor de plataforma resuelve cancelando y reembolsando lo no devengado. Así:
- el cliente no paga bono (a diferencia de una disputa), y
- el cliente no se autoreembolsa (lo decide el asesor neutral), y
- el freelancer se queda lo que ya cobró en milestones aprobados.

**Por qué**
- Reembolso con `new_with_signer` (corrige v2, que también lo necesitaba).
- `close = client` devuelve la renta. No se cobra comisión (servicio no consumido).
- La cancelación en `InProgress` queda en manos del asesor para proteger a ambas
  partes (el cliente no se queda el trabajo hecho, ni paga por el incumplimiento ajeno).

**Validaciones**
- `job.client == firmante` → `NotJobClient`
- `job.status == Created || Funded` → `InvalidJobStatus`
- `checked_add` evita desbordamiento en el total.

**Cuentas**: `client` (Signer, mut), `job` (mut PDA, close), `system_program`.

## `pause_job` / `unpause_job` / `expire_paused_job`

**`pause_job`** — Cliente pausa el job. Requiere `status == Created || Funded`,
`job.freelancer == None` y que no esté ya pausado. Guarda `paused=true`,
`paused_at=now`. Mientras está pausado, `deposit_funds` y `accept_application` se bloquean.

**`unpause_job`** — Cliente reanuda (`paused=false`).

**`expire_paused_job`** — Cualquiera puede llamarlo si `now - paused_at >
MAX_PAUSE_DURATION` (30 días). Reembolsa `amount+fee` (si `Funded`) y cierra el
job. Evita fondos atrapados para siempre.

**Validaciones**
- `job.client == firmante` → `NotJobClient`
- pausar solo en `Created`/`Funded` → `CannotPauseWithFreelancer`
- `check_not_paused` bloquea `deposit_funds`/`accept_application` → `JobPaused` /
  `JobPausedExpired`

**Cuentas**: `client` (Signer), `job` (mut PDA). `expire_paused_job` usa
`caller` (Signer) + `client` (UncheckedAccount, destinatario del reembolso) +
`job` (`close = client`).

## Diagrama

```mermaid
flowchart LR
    C([Cliente]) -->|create_job| J[(PDA job: Created)]
    C -->|deposit_funds: amount+fee| J
    J -->|Funded| F([Freelancer acepta + submit_work])
    F -->|Submitted| A[Cliente approve_work]
    A -->|paga amount+fee, cierra| P([Freelancer + Treasury])
    A -.reject_work.-> F
    C -->|cancel_job| R([Reembolso + cierra])
    C -.->|pause_job (solo Created/Funded)| J
    J -.->|expire_paused_job tras 30d| R
```
