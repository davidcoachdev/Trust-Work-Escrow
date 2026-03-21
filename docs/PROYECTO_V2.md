# Trust Work Escrow v2 - Proyecto Completo

> Sistema de freelancing con escrow en Solana - Versión 2
>Hackathon WayLearn 2026

---

## 📋 Estado del Proyecto

| Phase | Estado | Fecha |
|-------|--------|-------|
| Phase 1: Foundation | ✅ Completado | 2026-03-21 |
| Phase 2: Core Implementation | ✅ Completado | 2026-03-21 |
| Phase 3: Testing | ⏳ Pendiente | - |
| Phase 4: Deployment | ⏳ Pendiente | - |

---

## 🎯 Objetivo

Crear un sistema de freelancing con escrow en Solana que permita:
- **Multi-wallet por usuario** - Como multiboot, múltiples wallets asociadas a una cuenta
- **Roles no encasillados** - Un usuario puede ser client, freelancer y arbiter
- **Pool de árbitros** - Árbitros registrados on-chain para disputas
- **Gobernanza multisig** - Admin y tesorero con 2-de-3 firmas

---

## 🏗️ Arquitectura

```
┌─────────────────────────────────────────────────────────────┐
│                    Frontend (Futuro)                        │
│   Web (Next.js) + CLI (Ratatui)                            │
└─────────────────────┬───────────────────────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────────────────────┐
│              Smart Contract v2 (Anchor/Rust)               │
├─────────────────────────────────────────────────────────────┤
│  Instructions (17)                                         │
│  ├── User: create_user, add_wallet, set_active_wallet    │
│  ├── Job: create_job, deposit, accept, submit, approve    │
│  ├── Arbiter: register_arbiters, raise_dispute            │
│  └── Config: initialize, pause, unpause, withdraw         │
├─────────────────────────────────────────────────────────────┤
│  State (Accounts)                                          │
│  ├── User (PDA) - wallet_principal, wallets_asociadas     │
│  ├── Job (PDA) - client, freelancer, amount, status       │
│  ├── Config (PDA) - admin, treasury, fee_percent          │
│  └── ArbiterPool (PDA) - registered arbiters              │
└─────────────────────────────────────────────────────────────┘
```

---

## 📁 Estructura del Proyecto

```
Trust-Work-Escrow/
├── trust-escrow-v2/              ← Smart Contract v2
│   ├── Anchor.toml
│   ├── programs/
│   │   └── trust-escrow-v2/
│   │       ├── Cargo.toml
│   │       └── src/
│   │           ├── lib.rs           ← Entry point
│   │           ├── error.rs         ← 30 custom errors
│   │           ├── state/           ← Account structs
│   │           │   ├── mod.rs
│   │           │   ├── config.rs
│   │           │   ├── user.rs
│   │           │   ├── job.rs
│   │           │   └── arbiter_pool.rs
│   │           └── instructions/    ← 17 instructions
│   │               ├── mod.rs
│   │               ├── user/
│   │               ├── job/
│   │               ├── arbiter/
│   │               └── config/
│   └── tests/
│       └── trust-escrow-v2.ts      ← Integration tests
│
├── docs/                         ← Documentación
│   ├── phases/
│   │   ├── PHASE1.md
│   │   ├── PHASE2.md
│   │   ├── PHASE3.md
│   │   └── PHASE4.md (pendiente)
│   ├── architecture/
│   ├── contracts/
│   ├── guides/
│   ├── hackathon/
│   ├── reference/
│   ├── smart-contract/
│   └── INDEX.md
│
└── trust-escrow/                 ← v1 (desplegada, no tocar)
```

---

## ⛓️ Instrucciones del Smart Contract

### User Instructions (4)

| Instrución | Descripción | Estado |
|------------|-------------|--------|
| `create_user` | Crea cuenta de usuario PDA | ✅ |
| `add_wallet` | Agrega wallet secundaria | ✅ |
| `set_active_wallet` | Cambia wallet activa | ✅ |
| `update_user` | Actualiza perfil (bio) | ✅ |

### Job Instructions (7)

| Instrución | Descripción | Estado |
|------------|-------------|--------|
| `create_job` | Crea trabajo/escrow | ✅ |
| `deposit_funds` | Deposita fondos | ✅ |
| `accept_job` | Freelancer acepta | ✅ |
| `submit_work` | Freelancer envía trabajo | ✅ |
| `approve_work` | Cliente aprueba + paga | ✅ |
| `reject_work` | Cliente rechaza + disputa | ✅ |
| `cancel_job` | Cliente cancela | ✅ |

### Arbiter Instructions (3)

| Instrución | Descripción | Estado |
|------------|-------------|--------|
| `register_arbiters` | Admin registra árbitros | ✅ |
| `raise_dispute` | Freelancer eleva disputa | ✅ |
| `resolve_dispute` | Arbiter resuelve (70-30) | ✅ |

### Config Instructions (4)

| Instrución | Descripción | Estado |
|------------|-------------|--------|
| `initialize_config` | Inicializa config global | ✅ |
| `pause` | Pausa el programa | ✅ |
| `unpause` | Reanuda el programa | ✅ |
| `withdraw_treasury` | Retira fees | ✅ |

**Total: 17 instrucciones**

---

## 🗂️ Cuentas (State)

### User Account

```rust
pub struct User {
    pub wallet_principal: Pubkey,
    pub wallets_asociadas: Vec<Pubkey>,  // Max 10
    pub active_wallet: Pubkey,
    pub username: String,               // Max 32
    pub bio: Option<String>,            // Max 500
    pub created_at: i64,
    pub bump: u8,
}
// Seeds: [b"user", wallet.as_ref()]
```

### Job Account

```rust
pub struct Job {
    pub client: Pubkey,
    pub freelancer: Option<Pubkey>,
    pub arbiter: Option<Pubkey>,
    pub amount: u64,
    pub fee_percent: u8,
    pub fee_amount: u64,
    pub status: JobStatus,
    pub title: String,                  // Max 100
    pub description: String,            // Max 500
    pub deadline: i64,
    pub created_at: i64,
    pub updated_at: i64,
    pub dispute_reason: String,         // Max 200
    pub bump: u8,
}
// Seeds: [b"job", client.as_ref(), job_id.to_le_bytes()]
```

### Config Account

```rust
pub struct Config {
    pub admin: Pubkey,
    pub treasury: Pubkey,
    pub multisig_owners: Vec<Pubkey>,    // Max 5
    pub multisig_threshold: u8,          // 2 default
    pub fee_percent: u8,                 // 5 default
    pub paused: bool,
    pub bump: u8,
}
// Seeds: [b"config"]
```

### ArbiterPool Account

```rust
pub struct ArbiterPool {
    pub authority: Pubkey,
    pub arbiters: Vec<Pubkey>,          // Max 50
    pub bump: u8,
}
// Seeds: [b"arbiter_pool"]
```

---

## 🔐 Errores Custom (30)

| Código | Descripción |
|--------|-------------|
| `UserAlreadyExists` | Usuario ya existe |
| `WalletAlreadyAssociated` | Wallet ya asociada |
| `WalletNotAssociated` | Wallet no asociada |
| `MaxWalletsReached` | Máximo de wallets (10) |
| `MaxArbitersReached` | Máximo de árbitros (50) |
| `NotAuthorized` | No autorizado |
| `NotAdmin` | No es admin |
| `NotArbiter` | No es árbitro |
| `CannotAcceptOwnJob` | No puedes aceptar tu propio job |
| `ProgramPaused` | Programa pausado |
| `InvalidJobStatus` | Estado inválido |
| ... (20 más) | |

---

## 🧪 Testing

### Tests Creados

Ubicación: `trust-escrow-v2/tests/trust-escrow-v2.ts`

| Suite | Tests |
|-------|-------|
| Config | initialize_config |
| User | create_user, add_wallet, set_active_wallet |
| Job | create_job, accept_job, submit_work, approve_work |

### Ejecutar Tests

```bash
# Instalar Anchor CLI
avm install latest
avm use latest

# Compilar
cd trust-escrow-v2
anchor build

# Ejecutar tests
anchor test
```

---

## 🚀 Deployment

### Pasos

1. `anchor build` - Compilar el programa
2. `anchor deploy` - Desplegar a devnet
3. Verificar IDL en `target/idl/`
4. Documentar Program ID

### Program ID (placeholder)

```
TRUST2XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX
```

---

## 📚 Documentación

| Documento | Descripción |
|----------|-------------|
| `docs/INDEX.md` | Índice principal |
| `docs/phases/PHASE1.md` | Foundation |
| `docs/phases/PHASE2.md` | Implementación |
| `docs/phases/PHASE3.md` | Testing |
| `docs/contracts/DESIGN.md` | Diseño v2 |
| `docs/hackathon/HACKATHON.md` | Info hackathon |

---

## 🔗 Recursos

- [Anchor Documentation](https://book.anchor-lang.com/)
- [Solana Documentation](https://docs.solana.com/)
- [WayLearn Hackathon](https://dorahacks.io/hackathon/solana-waylearn-2026/detail)

---

## 📅 Fechas Hackathon

| Evento | Fecha |
|--------|-------|
| Inicio desarrollo | 20 marzo 2026 |
| Periodo construcción | 20-23 marzo |
| Entrega | 23 marzo 23:59 |
| Premios | $2,500 / $1,500 / $1,000 USDC |

---

## 🤝 Contribuidores

- Proyecto para Hackathon WayLearn 2026
- Categoría: DAOs / Social

---

_Last updated: 2026-03-21_