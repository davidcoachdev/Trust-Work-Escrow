# Smart Contract - Trust Work Escrow

## 📋 Descripción — v3 vigente (`trust-escrow-v3` `7a2YhCd7iivXfyySkp1pf5jjijGqpjNqwQCUS912q5Vh`, Anchor 0.32.1)

El smart contract vigente es `trust-escrow-v3` (program id `7a2YhCd7iivXfyySkp1pf5jjijGqpjNqwQCUS912q5Vh`).
El contrato histórico v1/v2 se conserva abajo como referencia. **El modelo vigente para Applications es PDA individual, no colección inline.**

### Modelo Applications PDA individual (T21-T26) — IDL vigente

**Job** es compacto: `Vec<Pubkey>` con capacidad reservada `MAX_APPLICATIONS = 50`, no `[Application; 50]` inline.
`Job::INIT_SPACE < 10 KiB` (< `28 KiB` de 50 Applications inline), `delta 50*32 bytes`. Ver `trust-escrow-v3/programs/trust-escrow-v3/src/lib.rs` y `backend/sdk/src/types.rs`.

```rust
#[account]
#[derive(InitSpace)]
pub struct Job {
    pub client: Pubkey,
    pub freelancer: Option<Pubkey>,
    pub amount: u64,
    pub fee_amount: u64,
    pub status: JobStatus,
    pub paused: bool,
    pub paused_at: i64,
    pub deadline: i64,
    pub submitted_at: Option<i64>,
    pub milestones_total: u8,
    pub milestones_approved: u8,
    pub milestones_amount_total: u64,
    #[max_len(MAX_APPLICATIONS)] // 50 — Vec<Pubkey> compacto, sin inline
    pub applicants: Vec<Pubkey>,
    pub bump: u8, // seeds [b"job", client, job_id.le_bytes()]
}

#[account]
#[derive(InitSpace)]
pub struct Application {
    pub job: Pubkey,              // Job PDA padre
    pub index: u8,                // 0..49, debe ser == job.applicants.len() al crear
    pub applicant: Pubkey,        // wallet del postulante — parte de la seed
    pub proposal_hash: [u8; 32],  // SHA256 off-chain del texto (1..512 chars), nunca [0;32]
    pub status: ApplicationStatus, // Pending/Accepted/Rejected/Withdrawn
    pub bump: u8,                 // seeds [b"application", job, &[index], applicant]
}
pub enum ApplicationStatus { Pending, Accepted, Rejected, Withdrawn }
const MAX_APPLICATIONS: usize = 50; // límite duro, Vec 50, IDL types Job.applicants: Vec<Pubkey>
```

**Seeds / ownership / bump (IDL y código alineados, validados en `backend/sdk/tests/t26_idl_docs.rs`):**

- `Job`: `seeds = [b"job", client.key().as_ref(), &job_id.to_le_bytes()]`, `bump = job.bump`, owner `crate::ID` (7a2Y...), PDA off-curve.
- `Application` (cuenta individual por postulante): `seeds = [b"application", job.key().as_ref(), &[application_index], applicant.key().as_ref()]`, `bump = application.bump`, owner `crate::ID`, PDA off-curve y determinista por `(job, index, applicant)`.
- SDK `pda::derive_application_pda` y `derive_job_pda` usan `Pubkey::find_program_address` con las mismas seeds; validado IDL vs código (address `7a2Y...`, types `Application`/`Job`, args y cuentas).

**Args / cuentas (IDL `trust-escrow-v3/target/idl/escrow.json` vs `lib.rs`):**

- `apply_to_job(ctx: Context<ApplyToJob>, _job_id: u64, application_index: u8, proposal_hash: [u8; 32])` — cuentas: `applicant (Signer, mut, payer)`, `client (UncheckedAccount, validado por PDA job)`, `job (mut, PDA job)`, `application (init, payer applicant, space Application::INIT_SPACE+8, seeds application)`, `system_program`. `application_index` debe ser `u8` y `== job.applicants.len()`.
- `accept_application(ctx: Context<AcceptApplication>, _job_id: u64, application_index: u8)` — `client (Signer, mut)`, `job (mut, PDA job)`, `applicant (SystemAccount)`, `application (mut, PDA application)`. Valida `application.index == index`, `status == Pending`, `job.applicants[index] == applicant`, asigna `job.freelancer = Some(applicant)`, `status = InProgress`, `application.status = Accepted`.
- `reject_application` / `withdraw_application` — mismos seeds; `application` con `close = applicant` (rent refund al postulante). Solo `Pending`.
- `cleanup_applications(ctx: Context<CleanupApplications>, _job_id: u64, start_index: u8)` — `client (Signer)`, `job (mut, PDA job, constraint client == job.client)`, `remaining_accounts: [application, applicant] * N` con `start_index`, batch hasta 50, rent refund de cada `Application` no-accepted al `applicant` (cerrada `assign SYSTEM_PROGRAM_ID, resize 0`).

**Límites texto:**

- On-chain: `proposal_hash != [0u8; 32]` (`EmptyProposal`), el texto off-chain se hashea con SHA256; el contrato solo ve el hash y rechaza hash nulo (propuesta vacía). Hash siempre 32 bytes.
- Off-chain / SDK / API: `1..=512` chars (`ProposalTooLong` / `EmptyProposal` en `backend/sdk`, `validation.rs`, `metadata.rs`). Hasheo determinista `hash(proposal.as_bytes())`.

**Unicidad:**

- `AlreadyApplied` si `job.applicants` ya contiene `applicant`, aunque cambie `index`.
- `CannotWorkOnOwnJob` si `applicant == job.client`.
- `ApplicationIndexMismatch` si `index != job.applicants.len()`, `InvalidApplicationIndex` si `len >= 50` o `index >= 50`. Índice válido `0..49`.

**Cleanup / rent:**

- `Reject` y `Withdraw` cierran `Application` con `close = applicant` — rent vuelve al postulante, cuenta cerrada queda `owner = SYSTEM_PROGRAM_ID, data_len 0`.
- `cleanup_applications` itera `remaining_accounts` pares `(application, applicant)` desde `start_index`, valida PDA determinista y `stored.index/applicant/job`, y para cada `Pending/Rejected/Withdrawn` (no `Accepted` ni ya cerrada con `allow_closed=false`) hace `transfer rent → applicant` + `close`. `Accepted` se retiene. Batch con `allow_closed=true` tolera cuentas ya cerradas. Errores: `InvalidApplicationCleanupAccounts`.

**Sin modelo inline:** El contrato vigente no almacena `Vec<Application>` ni `[Application; 50]` dentro de `Job`; Job solo guarda `Vec<Pubkey>` de 50. La IDL (`types.Job.applicants: vec pubkey`) y `Job::INIT_SPACE` lo prueban (no `28 KiB` inline). Tests `job_compact_init_space_under_10kib_and_vec_50_compact` y `t26_*` lo blindan. Docs no deben referenciar modelo inline como vigente.

**Validación IDL vs código:** `backend/sdk/tests/t26_idl_docs.rs` valida en cada `cargo test --features solana` que program id, seeds, bump, ownership, args/cuentas, límites, unicidad, cleanup/rent y no-inline coinciden entre `lib.rs`, `types.rs`, `pda.rs` y `target/idl/escrow.json`. La IDL se regenera con `anchor build` y se compara en CI (`scripts/final-gate.sh`).

---

## 📋 Descripción (v1 histórico)

## 🏗️ Estructura del Programa

### Cuentas (Accounts)

#### 1. Config (Configuración Global)

```rust
#[account]
#[derive(InitSpace)]
pub struct Config {
    pub authority: Pubkey,      // Administrador del programa
    pub treasury: Pubkey,       // Wallet donde se envían las comisiones
    pub fee_percent: u8,        // Porcentaje de comisión (5 = 5%)
    pub paused: bool,           // Programa pausado
    pub bump: u8,               // Bump para PDA
}
```

**Espacio:** ~50 bytes

#### 2. Job (Trabajo/Escrow)

```rust
#[account]
#[derive(InitSpace)]
pub struct Job {
    pub client: Pubkey,           // Cliente que crea el trabajo
    pub freelancer: Option<Pubkey>, // Freelancer asignado (None si no aceptado)
    pub arbiter: Pubkey,         // Árbitro designado para disputas
    pub amount: u64,            // Monto total en lamports
    pub fee_percent: u8,        // Porcentaje de comisión
    pub fee_amount: u64,        // Comisión calculada
    pub status: JobStatus,      // Estado actual del trabajo
    pub title: String,          // Título del trabajo (max 100)
    pub description: String,    // Descripción detallada (max 500)
    pub deadline: i64,          // Fecha límite (timestamp)
    pub created_at: i64,        // Fecha de creación
    pub updated_at: i64,        // Última actualización
    pub dispute_reason: String, // Razón de la disputa (max 200)
    pub bump: u8,               // Bump para PDA
}
```

**Espacio:** ~50 bytes base + title + description + dispute_reason

#### 3. JobStatus (Enum)

```rust
pub enum JobStatus {
    Created,       // Creado, esperando depósito/accept
    Funded,        // Fondos depositados, esperando freelancer
    InProgress,    // Aceptado, en progreso
    Submitted,     // Entregado, esperando aprobación
    Released,      // Completado, fondos liberados
    Disputed,      // En disputa
    Resolved,      // Resuelto por árbitro
    Cancelled,     // Cancelado, refund al cliente
}
```

## 📝 Instrucciones (Instructions)

### 1. initialize_config

Inicializa la configuración global del programa. Solo puede ejecutarse una vez.

```rust
pub fn initialize_config(ctx: Context<InitializeConfig>) -> Result<()>
```

**Cuentas requeridas:**

- `authority` - Administrador (Signer, mutable)
- `treasury` - Wallet para recibir fees (UncheckedAccount)
- `config` - Cuenta de configuración (PDA, init)
- `system_program` - Programa del sistema

**Validaciones:**

- Solo puede ejecutarse una vez (PDA init falla si existe)
- Treasury se almacena en config para validación futura

---

### 2. create_job

Crea un nuevo trabajo (sin depositar fondos aún).

```rust
pub fn create_job(
    ctx: Context<CreateJob>,
    job_id: u64,
    title: String,
    description: String,
    amount: u64,
    deadline: i64,
) -> Result<()>
```

**Cuentas requeridas:**

- `client` - Cliente (Signer, mutable)
- `arbiter` - Árbitro designado (UncheckedAccount)
- `job` - Cuenta del trabajo (PDA, init)
- `config` - Configuración global
- `system_program` - Programa del sistema

**Validaciones:**

- Programa no pausado
- `amount` >= MIN_JOB_AMOUNT (100,000 lamports)
- `title` no vacío y <= 100 caracteres
- `description` <= 500 caracteres

**Flujo:**

1. Calcula la comisión (`amount * 5 / 100`)
2. Crea la cuenta Job con status = Created
3. Almacena arbiter en el Job para disputas futuras

---

### 3. deposit_funds

Deposita fondos (amount + fee) en el Job PDA.

```rust
pub fn deposit_funds(ctx: Context<DepositFunds>, job_id: u64) -> Result<()>
```

**Cuentas requeridas:**

- `client` - Cliente (Signer, mutable)
- `job` - Cuenta del trabajo (PDA)
- `config` - Configuración global
- `system_program` - Programa del sistema

**Validaciones:**

- Solo si status = Created
- Solo el cliente del job puede depositar

**Flujo:**

1. Calcula total = amount + fee_amount
2. CPI a System Program para transferir lamports
3. Cambia status a Funded

---

### 4. accept_job

Acepta un trabajo como freelancer.

```rust
pub fn accept_job(ctx: Context<AcceptJob>, job_id: u64) -> Result<()>
```

**Cuentas requeridas:**

- `freelancer` - Freelancer (Signer)
- `job` - Cuenta del trabajo (PDA)
- `config` - Configuración global

**Validaciones:**

- Solo puede ejecutarse si status = Funded
- No puede aceptarse uno mismo (client != freelancer)

**Flujo:**

1. Asigna `freelancer` a la cuenta Job
2. Cambia status a InProgress

---

### 5. submit_work

El freelancer marca el trabajo como completado.

```rust
pub fn submit_work(ctx: Context<SubmitWork>, job_id: u64) -> Result<()>
```

**Cuentas requeridas:**

- `freelancer` - Freelancer (Signer)
- `job` - Cuenta del trabajo (PDA)
- `config` - Configuración global

**Validaciones:**

- Solo el freelancer puede executar
- Solo si status = InProgress

**Flujo:**

1. Cambia status a Submitted

---

### 6. approve_work

El cliente aprueba el trabajo y libera los fondos.

```rust
pub fn approve_work(ctx: Context<ApproveWork>, job_id: u64) -> Result<()>
```

**Cuentas requeridas:**

- `client` - Cliente (Signer, mutable)
- `job` - Cuenta del trabajo (PDA, close → client)
- `freelancer` - Cuenta del freelancer (mutable, SystemAccount)
- `treasury` - Treasury del protocolo (mutable, validado contra config)
- `config` - Configuración global

**Validaciones:**

- Solo el cliente puede executar
- Solo si status = Submitted
- Freelancer asignado coincide con la cuenta
- Treasury coincide con config.treasury

**Flujo:**

1. Calcula `net_payment = amount - fee_amount` (el freelancer también aporta su 5%)
2. Transfiere `net_payment` al freelancer
3. Transfiere `fee_amount * 2` (doble comisión: cliente + freelancer) al treasury
4. Cierra la cuenta Job (rent → client)
5. Cambia status a Released

> **Nota de comisiones:** El cliente deposita `amount + fee_amount` (105%). Al completar, el freelancer recibe el 95% neto y el treasury acumula el 10% total (5% de cada parte).

---

### 7. reject_work

El cliente rechaza el trabajo y abre disputa.

```rust
pub fn reject_work(ctx: Context<RejectWork>, job_id: u64, reason: String) -> Result<()>
```

**Cuentas requeridas:**

- `client` - Cliente (Signer)
- `job` - Cuenta del trabajo (PDA)
- `config` - Configuración global

**Validaciones:**

- Solo el cliente puede executar
- Solo si status = Submitted
- Reason no vacío

**Flujo:**

1. Guarda la razón en `dispute_reason`
2. Cambia status a Disputed

---

### 8. raise_dispute

El freelancer abre una disputa (solo después de submit).

```rust
pub fn raise_dispute(ctx: Context<RaiseDispute>, job_id: u64, reason: String) -> Result<()>
```

**Cuentas requeridas:**

- `freelancer` - Freelancer (Signer)
- `job` - Cuenta del trabajo (PDA)
- `config` - Configuración global

**Validaciones:**

- Solo el freelancer puede executar
- Solo si status = Submitted
- Reason no vacío

**Flujo:**

1. Guarda la razón en `dispute_reason`
2. Cambia status a Disputed

---

### 9. resolve_dispute

El árbitro resuelve la disputa.

```rust
pub fn resolve_dispute(
    ctx: Context<ResolveDispute>,
    job_id: u64,
    freelancer_percent: u8,
) -> Result<()>
```

**Cuentas requeridas:**

- `arbiter` - Árbitro (Signer, validado con has_one)
- `client` - Cuenta del cliente (mutable, SystemAccount)
- `job` - Cuenta del trabajo (PDA, close → client)
- `freelancer` - Cuenta del freelancer (mutable, SystemAccount)
- `treasury` - Treasury del protocolo (mutable, validado contra config)
- `config` - Configuración global

**Validaciones:**

- Solo el árbitro asignado puede executar (has_one = arbiter)
- Freelancer coincide con job.freelancer
- Solo si status = Disputed
- `freelancer_percent` entre 0 y 100
- Treasury coincide con config.treasury

**Flujo:**

1. Calcula `net_amount = amount - fee_amount` (base para el reparto, excluyendo comisión del freelancer)
2. Calcula distribución: `freelancer_amount = net_amount * freelancer_percent / 100`
3. Transfiere `freelancer_amount` al freelancer
4. Transfiere `fee_amount * 2` (doble comisión: cliente + freelancer) al treasury
5. Cierra la cuenta Job (porción del cliente en net_amount + rent → client)
6. Cambia status a Resolved

> **Nota de comisiones:** Igual que `approve_work`, el treasury recibe el 10% total (5% de cada parte) sin importar el resultado de la disputa.

---

### 10. cancel_job

Cancela un trabajo (solo si Created o Funded).

```rust
pub fn cancel_job(ctx: Context<CancelJob>, job_id: u64) -> Result<()>
```

**Cuentas requeridas:**

- `client` - Cliente (Signer)
- `job` - Cuenta del trabajo (PDA, close → client)
- `config` - Configuración global

**Validaciones:**

- Solo el cliente puede executar
- Solo si status = Created o Funded

**Flujo:**

1. Si Funded: reembolsa fondos al cliente
2. Cierra la cuenta Job (rent → client)
3. Cambia status a Cancelled

---

## 🔐 Seguridad

### Validaciones Implementadas

1. **Signers**: Todas las instrucciones críticas requieren firma
2. **Owner checks**: Verificación de que el caller es el propietario
3. **Status checks**: Cada instrucción verifica el estado actual
4. **Amount checks**: Validación de montos positivos
5. **Deadline checks**: Verificación de fechas

### Errores Personalizados

```rust
#[error_code]
pub enum ErrorCode {
    #[msg("El programa está pausado")]
    ProgramPaused,
    #[msg("El monto es muy pequeño")]
    AmountTooSmall,
    #[msg("El título no puede estar vacío")]
    EmptyTitle,
    #[msg("El título excede el largo máximo")]
    TitleTooLong,
    #[msg("La descripción excede el largo máximo")]
    DescriptionTooLong,
    #[msg("Estado de job inválido para esta operación")]
    InvalidJobStatus,
    #[msg("No eres el cliente de este trabajo")]
    NotJobClient,
    #[msg("No eres el freelancer de este trabajo")]
    NotJobFreelancer,
    #[msg("No puedes trabajar en tu propio proyecto")]
    CannotWorkOnOwnJob,
    #[msg("No hay freelancer asignado")]
    NoFreelancerAssigned,
    #[msg("La razón de la disputa no puede estar vacía")]
    EmptyDisputeReason,
    #[msg("Porcentaje inválido")]
    InvalidPercent,
    #[msg("No autorizado")]
    NotAuthorized,
    #[msg("Treasury inválido")]
    InvalidTreasury,
}
```

## 📊 Constants

```rust
const FEE_PERCENT: u8 = 5;                   // 5% fee
const MAX_TITLE_LENGTH: usize = 100;         // Máximo título
const MAX_DESCRIPTION_LENGTH: usize = 500;   // Máximo descripción
const MIN_JOB_AMOUNT: u64 = 100_000;         // 0.0001 SOL mínimo
```

## 🔗 PDAs (Program Derived Addresses)

### Config PDA

```
seeds = [SEED_CONFIG]
bump = config.bump
```

### Job PDA

```
seeds = [SEED_JOB, client.as_ref(), job_id.to_le_bytes().as_ref()]
bump = job.bump
```

## 📈 Logging

El programa emite logs via `msg!()` para cada operación:

```
"Config initialized by: <authority>"
"Job created: <pda> - Amount: <lamports> lamports"
"Funds deposited: <lamports> lamports"
"Job accepted by: <freelancer>"
"Work submitted for job: <pda>"
"Work approved. Payment: <amount> to freelancer, <fee> fee to treasury"
"Work rejected, dispute opened for job: <pda>"
"Dispute raised for job: <pda>"
"Dispute resolved: <x>% freelancer, <y>% client"
"Job cancelled: <pda>"
"Program paused"
"Program unpaused"
```

## 🧪 Testing

### Tests de Integración

```bash
anchor test
```

### Tests Incluidos

1. **initialize_config** - Configuración inicial
2. **create_job** - Crear trabajo
3. **accept_job** - Aceptar trabajo
4. **submit_work** - Entregar trabajo
5. **approve_work** - Aprobar y liberar fondos
6. **reject_work** - Rechazar y abrir disputa
7. **raise_dispute** - Abrir disputa (freelancer)
8. **resolve_dispute** - Resolver disputa
9. **cancel_job** - Cancelar trabajo

## 📚 Referencias

- [Anchor Documentation](https://www.anchor-lang.com/)
- [Solana Program Tutorial](https://docs.solana.com/developing/on-chain-programs/overview)
- [Security Best Practices](https://docs.solana.com/developing/security-best-practices)
