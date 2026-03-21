# Explicación del Contrato Inteligente Trust Work Escrow

Este documento explica el archivo `lib.rs` del contrato inteligente Trust Work Escrow, parte por parte. El contrato está construido usando Anchor para Solana e implementa un sistema de escrow para interacciones entre freelancers y clientes.

## Resumen

El contrato gestiona trabajos donde los clientes depositan fondos en una cuenta PDA (Program-Derived Address) de escrow. Los freelancers pueden aceptar trabajos, enviar trabajo, y los clientes pueden aprobar o disputar. Un árbitro resuelve disputas. El protocolo cobra una tarifa del 5% en transacciones exitosas.

## Parte 1: Importaciones y Declaraciones

```rust
use anchor_lang::prelude::*;
use anchor_lang::system_program;

declare_id!("5gu5JCSpB8MKyJzhXpGaCt8SruAMnRD6cTPbwPX6JTYo");
```

- **Importaciones**: Trae el preludio de Anchor (tipos comunes como `Account`, `Signer`) y el programa del sistema para transferencias.
- **declare_id!**: Define la dirección en cadena del programa. Debe coincidir con el keypair usado para el despliegue.

## Parte 2: Constantes

```rust
const FEE_PERCENT: u8 = 5;
const MAX_TITLE_LENGTH: usize = 100;
const MAX_DESCRIPTION_LENGTH: usize = 500;
const MIN_JOB_AMOUNT: u64 = 100_000; // 0.0001 SOL mínimo
```

- **FEE_PERCENT**: Tarifa del protocolo (5% del monto del trabajo).
- **MAX_TITLE_LENGTH/MAX_DESCRIPTION_LENGTH**: Límites en campos de cadena para controlar el tamaño de la cuenta.
- **MIN_JOB_AMOUNT**: Monto mínimo de trabajo en lamports (previene transacciones de polvo).

## Parte 3: Módulo del Programa e Instrucciones

La macro `#[program]` define el módulo. Cada `pub fn` es una instrucción llamable vía RPC.

### initialize_config

```rust
pub fn initialize_config(ctx: Context<InitializeConfig>) -> Result<()> {
    let config = &mut ctx.accounts.config;
    config.authority = ctx.accounts.authority.key();
    config.treasury = ctx.accounts.treasury.key();
    config.fee_percent = FEE_PERCENT;
    config.paused = false;
    config.bump = ctx.bumps.config;
    msg!("Config initialized by: {}", config.authority);
    Ok(())
}
```

- **Propósito**: Configura el PDA global de config con autoridad, tesorería y ajustes.
- **Lógica**: Inicializa campos; bump para derivación de PDA.
- **Seguridad**: Solo se llama una vez; la autoridad controla la pausa.

### create_job

```rust
pub fn create_job(
    ctx: Context<CreateJob>,
    _job_id: u64,
    title: String,
    description: String,
    amount: u64,
    deadline: i64,
) -> Result<()> {
    // Validaciones...
    let job = &mut ctx.accounts.job;
    // Inicializar campos del job...
    msg!("Job created: {} - Amount: {} lamports", job.key(), amount);
    Ok(())
}
```

- **Propósito**: El cliente crea un PDA de job con detalles.
- **Validaciones**: Verifica que el programa no esté pausado, monto >= mínimo, cadenas dentro de límites.
- **Lógica**: Establece estado del job a `Created`, calcula tarifa, almacena metadatos.
- **Semilla PDA**: `[b"job", client.key().as_ref(), &job_id.to_le_bytes()]` asegura jobs únicos por cliente.

### deposit_funds

```rust
pub fn deposit_funds(ctx: Context<DepositFunds>, _job_id: u64) -> Result<()> {
    // Chequeos...
    let transfer_amount = job.amount + job.fee_amount;
    system_program::transfer(/* CPI */)?;
    job.status = JobStatus::Funded;
    // ...
}
```

- **Propósito**: El cliente deposita fondos en el escrow del PDA del job.
- **Lógica**: Usa CPI (Cross-Program Invocation) para transferir SOL de forma segura. Estado cambia a `Funded`.

### accept_job

```rust
pub fn accept_job(ctx: Context<AcceptJob>, _job_id: u64) -> Result<()> {
    // Chequeos...
    job.freelancer = Some(ctx.accounts.freelancer.key());
    job.status = JobStatus::InProgress;
    // ...
}
```

- **Propósito**: El freelancer acepta el job financiado.
- **Validaciones**: El job debe estar financiado; freelancer != cliente.
- **Lógica**: Asigna freelancer, actualiza estado.

### submit_work

```rust
pub fn submit_work(ctx: Context<SubmitWork>, _job_id: u64) -> Result<()> {
    // Chequeos...
    job.status = JobStatus::Submitted;
    // ...
}
```

- **Propósito**: El freelancer marca el trabajo como enviado.
- **Validaciones**: Solo el freelancer asignado puede enviar.

### approve_work

```rust
pub fn approve_work(ctx: Context<ApproveWork>, _job_id: u64) -> Result<()> {
    // Chequeos...
    // Pagar al freelancer y tesorería...
    job.status = JobStatus::Released;
    // ...
}
```

- **Propósito**: El cliente aprueba el trabajo, liberando fondos.
- **Lógica**: Transfiere pago al freelancer, tarifa a tesorería. Cierra PDA, reembolsando rent al cliente.
- **Seguridad**: Usa borrows de lamports para manipulación directa de SOL.

### reject_work / raise_dispute

```rust
pub fn reject_work(ctx: Context<RejectWork>, _job_id: u64, reason: String) -> Result<()> {
    // Chequeos...
    job.status = JobStatus::Disputed;
    job.dispute_reason = reason;
    // ...
}
```

- **Propósito**: El cliente rechaza el trabajo o el freelancer levanta disputa, abriendo arbitraje.
- **Validaciones**: Requiere razón; estado debe ser `Submitted`.

### resolve_dispute

```rust
pub fn resolve_dispute(
    ctx: Context<ResolveDispute>,
    _job_id: u64,
    freelancer_percent: u8,
) -> Result<()> {
    // Chequeos...
    // Dividir fondos basado en porcentaje...
    job.status = JobStatus::Resolved;
    // ...
}
```

- **Propósito**: El árbitro resuelve disputa dividiendo fondos.
- **Lógica**: Paga al freelancer su porcentaje, tarifa a tesorería, resto al cliente. Cierra PDA.

### cancel_job

```rust
pub fn cancel_job(ctx: Context<CancelJob>, _job_id: u64) -> Result<()> {
    // Chequeos...
    if job.status == JobStatus::Funded {
        // Reembolsar fondos...
    }
    job.status = JobStatus::Cancelled;
    // ...
}
```

- **Propósito**: El cliente cancela job no financiado o financiado.
- **Lógica**: Reembolsa fondos depositados si aplica; cierra PDA.

### pause_program / unpause_program

```rust
pub fn pause_program(ctx: Context<PauseProgram>) -> Result<()> {
    // Chequeos...
    config.paused = true;
    // ...
}
```

- **Propósito**: La autoridad puede pausar/despausar el programa (parada de emergencia).

## Parte 4: Estructuras de Cuentas

Cada `#[derive(Accounts)]` define cuentas requeridas para una instrucción, con restricciones.

- **InitializeConfig**: Crea PDA de config.
- **CreateJob**: Crea PDA de job, valida config.
- **DepositFunds/AcceptJob/etc.**: Mutan PDA de job, chequean semillas y restricciones.
- **ApproveWork/ResolveDispute**: Incluyen restricciones de cierre para reembolsar rent.

## Parte 5: Estructuras de Datos

### Config

```rust
#[account]
#[derive(InitSpace)]
pub struct Config {
    pub authority: Pubkey,
    pub treasury: Pubkey,
    pub fee_percent: u8,
    pub paused: bool,
    pub bump: u8,
}
```

- Almacena ajustes globales. PDA sembrado con `[b"config"]`.

### Job

```rust
#[account]
#[derive(InitSpace)]
pub struct Job {
    pub client: Pubkey,
    pub freelancer: Option<Pubkey>,
    pub arbiter: Pubkey,
    pub amount: u64,
    pub fee_percent: u8,
    pub fee_amount: u64,
    pub status: JobStatus,
    #[max_len(100)]
    pub title: String,
    // ... otros campos
    pub bump: u8,
}
```

- Representa un job. Usa `InitSpace` para cálculo automático de espacio.

### JobStatus

```rust
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Debug, InitSpace)]
pub enum JobStatus {
    Created,
    Funded,
    InProgress,
    Submitted,
    Released,
    Disputed,
    Resolved,
    Cancelled,
}
```

- Enum para estados del job, serializable para cuentas.

## Parte 6: Códigos de Error

```rust
#[error_code]
pub enum ErrorCode {
    #[msg("El programa está pausado")]
    ProgramPaused,
    // ... otros errores
}
```

- Errores personalizados con mensajes en español para retroalimentación amigable al usuario.

## Notas de Seguridad

- **PDAs**: Todas las cuentas usan semillas para direcciones determinísticas.
- **Validaciones**: Chequeos extensivos con `require!` previenen estados inválidos.
- **CPIs**: Transferencias seguras vía programa del sistema.
- **Cierres**: PDAs se cierran para reembolsar rent cuando los jobs terminan.
- **Autoridad**: Autoridad de config controla pausa; árbitros por job.

Este contrato asegura escrow sin confianza con resolución de disputas. Prueba exhaustivamente antes del despliegue en mainnet.