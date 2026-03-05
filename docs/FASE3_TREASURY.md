# Fase 3: Treasury y Seguridad Avanzada

> Para cuando el proyecto llegue a hackatón o investor day.

---

## 🏦 Modelo de Negocio

### Flujo de Pagos (Propuesto)

```
┌─────────────────────────────────────────────────────────────────────────┐
│                      FLUJO DE PAGOS                                      │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  1. CLIENTE PUBLICA JOB                                                 │
│     ┌─────────────────┐                                                 │
│     │ Job: App Web    │                                                 │
│     │ Monto: 5 USDC  │                                                 │
│     │ Comisión: 5%   │ → 0.25 USDC                                      │
│     └────────┬────────┘                                                 │
│              │                                                          │
│              ▼                                                          │
│  2. CLIENTE DEPOSITA                                                    │
│     ┌─────────────────┐     ┌─────────────────┐                        │
│     │ Cliente Wallet  │────►│  Wallet App     │                        │
│     │ -5 USDC        │     │  (Treasury)     │                        │
│     └─────────────────┘     │  +5 USDC        │                        │
│                             │  Estado: OK    │                        │
│                             └────────┬────────┘                        │
│                                    │                                   │
│                                    ▼                                   │
│  3. FREELANCER TRABAJA                                              │
│     ┌─────────────────┐     ┌─────────────────┐                        │
│     │ Acepta          │     │ Job: InProgress │                        │
│     │ Entrega trabajo │────►│ Entregado       │                        │
│     └─────────────────┘     └────────┬────────┘                        │
│                                      │                                  │
│                                      ▼                                  │
│  4. APP VERIFICA Y PAGA                                              │
│     ┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐ │
│     │ Cliente aprueba │────►│ Wallet App      │────►│ Freelancer     │ │
│     │                 │     │ -4.75 USDC     │     │ +4.75 USDC     │ │
│     │                 │     │ -0.25 USDC(fee)│     │                 │ │
│     └─────────────────┘     └─────────────────┘     └─────────────────┘ │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

### Comisión de la Plataforma

| Modelo | Comisión | Ejemplo (5 USDC) |
|--------|----------|------------------|
| **Propuesto** | **5%** | 0.25 USDC |
| Upwork/Fiverr | 10-20% | 0.50-1.00 USDC |
| Traditional Escrow | 3-10% | 0.15-0.50 USDC |
| DeFi Escrow | 0.1-1% | 0.005-0.05 USDC |

**Comisión sugerida: 5%**
- ✅ Suficiente para mantener la plataforma
- ✅ Competitivo vs escrow tradicionales
- ✅ Menor que Upwork/Fiverr (10-20%)
- ✅ Incentiva uso vs plataformas centralizadas

---

## 💰 Wallet de la App (Treasury)

### Concepto

La **Wallet de la App** es una wallet real (tipo Ledger o cold wallet) que solo el admin puede controlar. Es como una "bóveda" donde se depositan los fondos de los clientes.

### Características

| Característica | Descripción |
|---------------|-------------|
| **Tipo** | Wallet hardware (Ledger) o cold wallet |
| **Acceso** | Solo admin (multi-sig opcional) |
| **Tokens** | SOL, USDC, y otros SPL tokens |
| **Seguridad** | Offline/cold storage |
| **Verificación** | On-chain (el smart contract verifica depósitos) |

### Diferencia con Modelo Tradicional

| Aspecto | Modelo Tradicional | Nuestro Modelo |
|---------|-------------------|----------------|
| Control de fondos | Tercer banco | Admin de la app |
| Retiros | Bancarios | Wallet crypto |
| Verificación | Confirma banco | Confirma blockchain |
| Transparencia | Baja | Alta (on-chain) |

---

## 🏗️ Arquitectura Híbrida

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    ARQUITECTURA HÍBRIDA                                 │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│   ┌──────────────┐         ┌──────────────┐         ┌──────────────┐   │
│   │   Cliente   │         │    App       │         │  Freelancer  │   │
│   │  (On-chain) │         │  (Off-chain) │         │  (On-chain)  │   │
│   └──────┬───────┘         └──────┬───────┘         └──────┬───────┘   │
│          │                        │                        │            │
│          │  create_job            │                        │            │
│          │──────────────────────►│                        │            │
│          │                        │                        │            │
│          │  deposita (off-chain)  │                        │            │
│          │──────────────────────►│  verifica             │            │
│          │                        │────┐                  │            │
│          │                        │◄───┘                  │            │
│          │                        │                        │            │
│          │  accept/submit         │                        │            │
│          │◄───────────────────────│                        │            │
│          │                        │                        │            │
│          │  approve               │  paga (off-chain)     │            │
│          │───────────────────────│───────────────────────►│            │
│          │                        │  -fee                 │            │
│          │                        │────┐                  │            │
│          │                        │◄───┘                  │            │
│                                                                          │
│   ┌──────────────────────────────────────────────────────────────────┐  │
│   │                    SMART CONTRACT                                │  │
│   │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐             │  │
│   │  │    Job      │  │   Config    │  │   Events    │             │  │
│   │  │  (State)   │  │  (Settings) │  │  (On-chain) │             │  │
│   │  └─────────────┘  └─────────────┘  └─────────────┘             │  │
│   │                                                                  │  │
│   │  - Estados del job                                             │  │
│   │  - Verificación de depósitos                                    │  │
│   │  - Registro de disputas                                        │  │
│   │  - Emitir eventos                                              │  │
│   └──────────────────────────────────────────────────────────────────┘  │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## 📋 Cuentas del Programa

### 1. Job (Estado del Trabajo)

```rust
#[account]
#[derive(InitSpace)]
pub struct Job {
    pub client: Pubkey,              // Cliente
    pub freelancer: Option<Pubkey>,  // Freelancer (None si no aceptado)
    pub arbiter: Pubkey,             // Árbitro
    pub amount: u64,                // Monto a pagar (sin fee)
    pub fee_percent: u8,            // Porcentaje de fee (5%)
    pub fee_amount: u8,             // Monto del fee calculado
    pub status: JobStatus,          // Estado actual
    pub title: String,             // Título
    pub description: String,       // Descripción
    pub deposit_confirmed: bool,   // ¿Fondos confirmados?
    pub deadline: i64,             // Fecha límite
    pub created_at: i64ado
    pub,           // Cre updated_at: i64,           // Actualizado
    pub bump: u8,                 // Bump PDA
}
```

### 2. Config (Configuración)

```rust
#[account]
#[derive(InitSpace)]
pub struct Config {
    pub authority: Pubkey,         // Admin de la app
    pub treasury_wallet: Pubkey,   // Wallet de la app (depositos)
    pub default_fee_percent: u8,  // Fee por defecto (5%)
    pub min_amount: u64,           // Monto mínimo
    pub max_amount: u64,          // Monto máximo
    pub pause: bool,              // Pausar programa
    pub bump: u8,                 // Bump PDA
}
```

### 3. Estado: JobStatus

```rust
pub enum JobStatus {
    Created,       // Creado, esperando depósito
    Funded,        // Depósito confirmado
    InProgress,    // Freelancer trabajando
    Submitted,     // Trabajo entregado
    Approved,      // Cliente aprobó → pagar
    Disputed,      // En disputa
    Resolved,      // Resuelto por árbitro
    Cancelled,     // Cancelado
}
```

---

## 🔐 Flujo de Verificación de Depósitos

### Paso 1: Cliente crea job (on-chain)

```rust
pub fn create_job(
    ctx: Context<CreateJob>,
    title: String,
    description: String,
    amount: u64,
    deadline: i64,
    arbiter: Pubkey,
) -> Result<()> {
    let job = &mut ctx.accounts.job;
    let config = &ctx.accounts.config;
    
    // Calcular fee (5%)
    let fee_percent = config.default_fee_percent;
    let fee_amount = amount * fee_percent as u64 / 100;
    
    job.client = ctx.accounts.client.key();
    job.freelancer = None;
    job.arbiter = arbiter;
    job.amount = amount;
    job.fee_percent = fee_percent;
    job.fee_amount = fee_amount;
    job.status = JobStatus::Created;
    job.deposit_confirmed = false;  // Esperando depósito
    job.title = title;
    job.description = description;
    job.deadline = deadline;
    job.created_at = Clock::get()?.unix_timestamp;
    job.updated_at = Clock::get()?.unix_timestamp;
    job.bump = ctx.bumps.job;
    
    emit!(JobCreated {
        job_id: job.key(),
        client: job.client,
        amount,
        fee_amount,
    });
    
    Ok(())
}
```

### Paso 2: Cliente deposita (off-chain)

```
El cliente transfiere manualmente:
- amount + fee a la treasury wallet de la app
- Ejemplo: 5 USDC + 0.25 USDC = 5.25 USDC
```

### Paso 3: App verifica y confirma (off-chain)

```
La app:
1. Verifica el depósito en blockchain
2. Llama a confirm_deposit (on-chain)
3. Actualiza el estado del job
```

### Paso 4: Confirmar depósito (on-chain)

```rust
pub fn confirm_deposit(
    ctx: Context<ConfirmDeposit>,
    job_id: u64,
) -> Result<()> {
    let job = &mut ctx.accounts.job;
    
    // Solo el admin puede confirmar
    require!(
        ctx.accounts.authority.key() == ctx.accounts.config.authority,
        ErrorCode::NotAuthorized
    );
    
    require!(
        job.status == JobStatus::Created,
        ErrorCode::InvalidStatus
    );
    
    job.status = JobStatus::Funded;
    job.deposit_confirmed = true;
    job.updated_at = Clock::get()?.unix_timestamp;
    
    emit!(DepositConfirmed { job_id: job.key() });
    
    Ok(())
}
```

---

## 💸释放 Pagos (Off-chain)

### Proceso de Pago

```typescript
// Frontend/Backend de la app
async function releasePayment(jobId: string) {
    const job = await program.account.job.fetch(jobId);
    
    // 1. Verificar que está aprobado
    if (job.status !== 'Approved') {
        throw new Error('Job no aprobado');
    }
    
    // 2. Calcular montos
    const fee = job.amount * job.feePercent / 100;
    const freelancerPayment = job.amount - fee;
    
    // 3. Pagar al freelancer (off-chain)
    await transferTokens(
        treasuryWallet,      // De: wallet app
        freelancerWallet,    // A: freelancer
        freelancerPayment   // Monto
    );
    
    // 4. Registrar fee
    await recordFee(fee);
    
    // 5. Actualizar estado on-chain
    await program.methods.releaseJob(jobId).accounts({...}).rpc();
}
```

---

## 👤 Roles y Permisos

| Función | Cliente | Freelancer | Árbitro | Admin |
|---------|---------|------------|---------|-------|
| create_job | ✅ | ❌ | ❌ | ❌ |
| confirm_deposit | ❌ | ❌ | ❌ | ✅ |
| accept_job | ❌ | ✅ | ❌ | ❌ |
| submit_work | ❌ | ✅ | ❌ | ❌ |
| approve_work | ✅ | ❌ | ❌ | ❌ |
| reject_work | ✅ | ❌ | ❌ | ❌ |
| raise_dispute | ❌ | ✅ | ❌ | ❌ |
| resolve_dispute | ❌ | ❌ | ✅ | ❌ |
| pause_program | ❌ | ❌ | ❌ | ✅ |
| update_config | ❌ | ❌ | ❌ | ✅ |
| withdraw_fees | ❌ | ❌ | ❌ | ✅ |

---

## 🔒 Medidas de Seguridad

### 1. Verificación de Depósito

```rust
// Siempre verificar que el depósito fue hecho antes de iniciar
require!(job.deposit_confirmed, ErrorCode::DepositNotConfirmed);
```

### 2. Pause Mechanism

```rust
// El admin puede pausar en emergencias
pub fn pause(ctx: Context<Pause>) -> Result<()> {
    let config = &mut ctx.accounts.config;
    config.pause = true;
    emit!(ProgramPaused);
    Ok(())
}
```

### 3. Límites de Monto

```rust
// Prevenir errores grandes
require!(
    amount >= config.min_amount && amount <= config.max_amount,
    ErrorCode::AmountOutOfRange
);
```

### 4. Rate Limiting

```rust
// Prevenir spam
#[account]
pub struct UserRateLimit {
    pub user: Pubkey,
    pub tx_count: u32,
    pub window_start: i64,
}
```

### 5. Multisig (Opcional)

```rust
// Múltiples admins para aprobar acciones sensibles
pub struct MultiSig {
    pub owners: Vec<Pubkey>,
    pub required: u8,
}
```

---

## 📊 Dashboard de Treasury

### Métricas

| Métrica | Descripción |
|---------|-------------|
| Total Deposits | Suma de todos los depósitos |
| Total Released | Pagado a freelancers |
| Total Fees | Comisiones cobradas |
| Active Jobs | Jobs activos |
| Disputed Jobs | Jobs en disputa |

---

## 🧪 Checklist para Hackatón/Investor Day

- [ ] Smart contract con estados correctos
- [ ] CLI para gestionar jobs
- [ ] Función de confirmar depósitos (admin)
- [ ] Verificación off-chain de pagos
- [ ] Sistema de fees (5%)
- [ ] Pause mechanism
- [ ] Dashboard de métricas
- [ ] Demo de flujo completo

---

## 📚 Recursos

- [Solana Security Best Practices](https://docs.solana.com/developing/security-best-practices)
- [Anchor Security Guidelines](https://www.anchor-lang.com/docs/security)
- [SPL Token Documentation](https://spl.solana.com/token)
