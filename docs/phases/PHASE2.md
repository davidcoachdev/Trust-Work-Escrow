# Phase 2: Jobs, Teams & Applications - Trust Work Escrow v2

## Descripción

Implementación de Jobs, Teams y sistema de aplicaciones.

## Fecha

2026-03-23

## Estado

✅ Completado

---

## Archivos Modificados

```
trust-escrow-v2/programs/trust-escrow-v2/src/
├── lib.rs                    # Consolidado - TODO el contrato
└── src/                      # Módulos eliminados (monolítico)
    ├── instructions/          # ❌ Eliminado - merge a lib.rs
    └── state/                # ❌ Eliminado - merge a lib.rs
```

---

## Instrucciones Implementadas

### User Instructions (4)

| Instrucción | Descripción |
|------------|-------------|
| `create_user` | Crea cuenta de usuario (username, multi-wallet) |
| `add_wallet` | Agrega wallet secundaria (max 5) |
| `set_active_wallet` | Cambia wallet activa |
| `update_user` | Actualiza bio del perfil |

### Team Instructions (2)

| Instrucción | Descripción |
|------------|-------------|
| `create_team` | Crea equipo de freelancers |
| `add_team_member` | Agrega miembro al equipo |

### Job Instructions (8)

| Instrucción | Descripción |
|------------|-------------|
| `create_job` | Crea job con título, descripción, monto y deadline |
| `deposit_funds` | Cliente deposita fondos + fee en escrow |
| `apply_to_job` | Freelancer/Team aplica con propuesta |
| `accept_application` | Cliente acepta aplicación y asigna freelancer |
| `submit_work` | Freelancer entrega trabajo completado |
| `approve_work` | Cliente aprueba y transfiere fondos + fee |
| `reject_work` | Cliente rechaza trabajo |
| `cancel_job` | Cliente cancela (refund si no hay freelancer) |

---

## Estructuras de Datos

### User
```rust
pub struct User {
    pub wallet_principal: Pubkey,
    pub wallets: Vec<Pubkey>,           // max 5
    pub active_wallet: Pubkey,
    pub username: String,               // max 32
    pub bio: Option<String>,            // max 500
    pub created_at: i64,
    pub bump: u8,
}
```

### Job
```rust
pub struct Job {
    pub client: Pubkey,
    pub freelancer: Option<Pubkey>,
    pub team: Option<Pubkey>,
    pub title: String,                  // max 64
    pub description: String,             // max 1024
    pub amount: u64,
    pub fee: u64,
    pub total_deposited: u64,
    pub deadline: i64,
    pub status: JobStatus,               // Created, ApplicationsOpen, InProgress, Submitted, Approved, Disputed, Cancelled
    pub applications: Vec<Application>,
    pub bump: u8,
    pub created_at: i64,
    pub updated_at: i64,
    pub submitted_at: Option<i64>,
}
```

### Application
```rust
pub struct Application {
    pub applicant: Pubkey,
    pub is_team: bool,
    pub proposal: String,                 // max 512
    pub applied_at: i64,
    pub status: ApplicationStatus,       // Pending, Accepted, Rejected, Withdrawn
}
```

---

## Flujo de un Job

```
CREATED → APPLICATIONS_OPEN → IN_PROGRESS → SUBMITTED → APPROVED
                       ↓                              ↓
                  CANCELLED                      DISPUTED → RESOLVED
                   (refund)                     (Phase 03)
```

---

## Seeds de PDAs

| Cuenta | Seed | Creador |
|--------|------|---------|
| Config | `b"config"` | initialize_config |
| User | `b"user", authority` | create_user |
| Team | `b"team", owner` | create_team |
| Job | `b"job", client, job_id` | create_job |

---

## Features de Seguridad

- ✅ Validación de estado del job
- ✅ Verificación de ownership (client, freelancer)
- ✅ Prevención de self-accept (freelancer ≠ client)
- ✅ Pause mechanism (admin puede pausar)
- ✅ Límites en longitudes de campos
- ✅ Validación de montos mínimos (100_000 lamports)
- ✅ Validación de deadlines (must be future)

---

## Bug Conocido: Anchor 0.32 `#[program]` Macro

**Problema:** Módulos anidados triggers bug #3690
**Solución:** Contrato monolítico en un solo `lib.rs`

---

## Siguiente

Phase 3: Disputes, Milestones & Treasury
