# Technical Design Document (TDD)
## Trust Work Escrow v2

---

## 1. Stack Tecnológico

### 1.1 Blockchain
- **Red**: Solana 3.0
- **Framework**: Anchor 0.32+
- **Lenguaje**: Rust 1.89+

### 1.2 Backend
- **Framework**: Rust (Axum)
- **Runtime**: Tokio
- **ORM**: SQLx (PostgreSQL)

### 1.3 Frontend
- **Framework**: Next.js 14+ (App Router)
- **UI**: React 19
- **Estado**: Zustand / TanStack Query
- **Estilos**: Tailwind CSS 4

### 1.4 Databases
- **Relacional**: PostgreSQL (datos estructurados, users, jobs)
- **NoSQL**: MongoDB (chat E2EE, logs de eventos)
- **Cache**: Redis (sesiones, cacheo)

### 1.5 Infrastructure
- **Hosting**: Vercel (frontend), Railway/Render (backend)
- **RPC**: Helius / Triton (webhooks para eventos)
- **Monitoreo**: Prometheus + Grafana

---

## 2. Arquitectura General

```
┌─────────────────────────────────────────────────────────────────┐
│                        FRONTEND (Next.js)                         │
│              Web + Wallet Connect + Dashboard                      │
└─────────────────────────────┬───────────────────────────────────────┘
                              │ HTTPS / WSS
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                     BACKEND (Rust Axum)                           │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────────────┐   │
│  │ Auth    │  │ Jobs    │  │ Users   │  │ Notifications   │   │
│  └─────────┘  └─────────┘  └─────────┘  └─────────────────┘   │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐                       │
│  │ Teams   │  │ Disputes│  │ AI      │                       │
│  └─────────┘  └─────────┘  └─────────┘                       │
└─────────────────────────────┬───────────────────────────────────────┘
                              │ gRPC / REST
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                      SDK (Rust escrow-core)                        │
│              Lógica compartida: PDAs, helpers, types             │
└─────────────────────────────┬───────────────────────────────────────┘
                              │ JSON-RPC
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                   SMART CONTRACT (Anchor/Rust)                     │
│           Program ID: [A DEFINIR EN DEPLOY]                       │
│                  17 instrucciones + 4 cuentas                     │
└─────────────────────────────┬───────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                          TESORERÍA                                │
│                   Wallet Multisig (Squads)                        │
└─────────────────────────────────────────────────────────────────┘
```

---

## 3. Estructura del Proyecto

```
trust-escrow-v2/
├── programs/                      # Smart contracts (Anchor)
│   └── trust-escrow-v2/
│       ├── src/
│       │   ├── lib.rs           # Entry point
│       │   ├── error.rs         # Custom errors
│       │   ├── state/           # Account structs
│       │   │   ├── mod.rs
│       │   │   ├── config.rs
│       │   │   ├── user.rs
│       │   │   ├── job.rs
│       │   │   ├── team.rs
│       │   │   ├── milestone.rs
│       │   │   └── dispute.rs
│       │   └── instructions/     # Instruction handlers
│       │       ├── mod.rs
│       │       ├── user.rs
│       │       ├── job.rs
│       │       ├── team.rs
│       │       └── config.rs
│       └── Cargo.toml
│
├── sdk/                          # Rust SDK (crate reutilizable)
│   ├── src/
│   │   ├── lib.rs
│   │   ├── client.rs            # RPC client
│   │   ├── pda.rs              # PDA derivation
│   │   ├── instructions.rs      # Transaction builders
│   │   └── types.rs             # Shared types
│   └── Cargo.toml
│
├── backend/                      # API Backend (Rust)
│   ├── src/
│   │   ├── main.rs
│   │   ├── routes/             # API routes
│   │   │   ├── mod.rs
│   │   │   ├── jobs.rs
│   │   │   ├── users.rs
│   │   │   ├── auth.rs
│   │   │   └── notifications.rs
│   │   ├── services/           # Business logic
│   │   ├── db/                 # Database
│   │   └── ws/                 # WebSocket server
│   ├── Cargo.toml
│   └── Dockerfile
│
├── frontend/                    # Next.js Frontend
│   ├── app/                   # App Router
│   │   ├── page.tsx
│   │   ├── jobs/
│   │   ├── dashboard/
│   │   └── api/
│   ├── components/
│   ├── lib/
│   ├── package.json
│   └── tailwind.config.ts
│
├── cli/                         # CLI (Rust + Clap)
│   └── src/main.rs
│
├── tui/                         # TUI (Rust + Ratatui)
│   └── src/
│
└── docs/                        # Documentación
```

---

## 4. Smart Contract Design

### 4.1 PDA Seeds

```rust
// Config global
config_pda = [b"config"]              // [global:config]

// User account
user_pda = [b"user", wallet.as_ref()] // [user:wallet_address]

// Job account
job_pda = [b"job", client.as_ref(), job_id.to_le_bytes()]
// [job:client_wallet:job_id]

// Team account
team_pda = [b"team", owner.as_ref(), team_id.to_le_bytes()]
// [team:owner_wallet:team_id]

// Milestone account
milestone_pda = [b"milestone", job.as_ref(), milestone_id.to_le_bytes()]
// [milestone:job_address:milestone_id]

// Dispute account
dispute_pda = [b"dispute", job.as_ref()]
// [dispute:job_address]

// Arbiter pool
arbiter_pool_pda = [b"arbiter_pool"]
```

### 4.2 Account Structs

#### Config Account
```rust
#[account]
#[derive(InitSpace)]
pub struct Config {
    pub admin: Pubkey,                 // Admin de la plataforma
    pub treasury: Pubkey,             // Wallet de treasury (multisig)
    pub multisig_owners: Vec<Pubkey>, // Owners del multisig (max 5)
    pub multisig_threshold: u8,       // Firmas requeridas (default 2)
    pub fee_percent: u8,               // Fee porcentual (default 5)
    pub dispute_stake_percent: u8,    // Stake de disputa (default 5)
    pub paused: bool,                  // Programa pausado
    pub bump: u8,
}
```

#### User Account
```rust
#[account]
#[derive(InitSpace)]
pub struct User {
    pub wallet_principal: Pubkey,           // Wallet principal
    pub wallets_asociadas: Vec<Pubkey>,   // Wallets secundarias (max 10)
    pub active_wallet: Pubkey,             // Wallet activa para contexto
    pub username: String,                  // Username (max 32)
    pub bio: Option<String>,               // Bio (max 500)
    pub created_at: i64,                   // Timestamp creación
    pub updated_at: i64,                   // Timestamp actualización
    pub bump: u8,
}
```

#### Job Account
```rust
#[account]
#[derive(InitSpace)]
pub struct Job {
    pub client: Pubkey,                    // Cliente
    pub freelancer: Option<Pubkey>,        // Freelancer (None si no aceptado)
    pub team_id: Option<Pubkey>,           // Equipo (None si individual)
    pub arbiter: Pubkey,                   // Árbitro asignado
    pub amount: u64,                       // Monto total (lamports)
    pub fee_amount: u64,                   // Fee calculado
    pub deposited_amount: u64,             // Monto depositado
    pub released_amount: u64,              // Monto liberado
    pub status: JobStatus,                  // Estado actual
    pub title: String,                     // Título (max 100)
    pub description: String,                // Descripción (max 1000)
    pub deadline: i64,                     // Fecha límite
    pub created_at: i64,
    pub updated_at: i64,
    pub bump: u8,
}

#[derive(Clone, Copy, PartialEq, Eq, AnchorSerialize, AnchorDeserialize, InitSpace)]
#[repr(u8)]
pub enum JobStatus {
    Draft,
    Created,
    Funded,
    InProgress,
    Submitted,
    Approved,
    Disputed,
    Resolved,
    Cancelled,
}
```

#### Team Account
```rust
#[account]
#[derive(InitSpace)]
pub struct Team {
    pub owner: Pubkey,                    // Owner del equipo
    pub name: String,                     // Nombre del equipo
    pub members: Vec<TeamMember>,         // Miembros
    pub created_at: i64,
    pub updated_at: i64,
    pub bump: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace)]
pub struct TeamMember {
    pub user: Pubkey,                     // Usuario
    pub role: TeamRole,                    // Rol en el equipo
    pub department: Option<String>,         // Departamento
    pub payout_percentage: u8,             // Porcentaje de pago (0-100)
    pub is_active: bool,
}
```

#### Milestone Account
```rust
#[account]
#[derive(InitSpace)]
pub struct Milestone {
    pub job: Pubkey,                      // Job padre
    pub index: u8,                        // Índice del hito
    pub title: String,                     // Título
    pub description: String,                // Descripción
    pub amount: u64,                       // Monto del hito
    pub status: MilestoneStatus,           // Estado
    pub deadline: i64,                     // Fecha límite
    pub created_at: i64,
    pub bump: u8,
}
```

#### Dispute Account
```rust
#[account]
#[derive(InitSpace)]
pub struct Dispute {
    pub job: Pubkey,                      // Job en disputa
    pub opened_by: Pubkey,                 // Quién abrió
    pub reason: String,                    // Razón
    pub evidence_hashes: Vec<String>,     // Hashes de evidencia
    pub status: DisputeStatus,             // Estado
    pub ai_summary: Option<String>,       // Resumen de IA
    pub resolution: Option<DisputeResolution>, // Resolución
    pub created_at: i64,
    pub resolved_at: Option<i64>,
    pub bump: u8,
}
```

### 4.3 Instrucciones

#### User Instructions (4)
| Instrución | Descripción |
|------------|-------------|
| `create_user` | Crea cuenta de usuario PDA |
| `add_wallet` | Agrega wallet secundaria |
| `set_active_wallet` | Cambia wallet activa |
| `update_user` | Actualiza perfil (bio, username) |

#### Team Instructions (5)
| Instrución | Descripción |
|------------|-------------|
| `create_team` | Crea equipo |
| `add_member` | Agrega miembro |
| `remove_member` | Elimina miembro |
| `update_member` | Actualiza rol/porcentaje |
| `update_team` | Actualiza info del equipo |

#### Job Instructions (7)
| Instrución | Descripción |
|------------|-------------|
| `create_job` | Crea trabajo (DRAFT) |
| `publish_job` | Publica y fondea (105%) |
| `accept_job` | Freelancer/equipo acepta |
| `submit_work` | Freelancer entrega |
| `approve_work` | Cliente aprueba |
| `reject_work` | Cliente rechaza |
| `cancel_job` | Cancela (sin freelancer) |

#### Milestone Instructions (3)
| Instrución | Descripción |
|------------|-------------|
| `add_milestone` | Agrega hito al job |
| `approve_milestone` | Aprueba hito (pago parcial) |
| `reject_milestone` | Rechaza hito |

#### Dispute Instructions (3)
| Instrución | Descripción |
|------------|-------------|
| `raise_dispute` | Abre disputa |
| `submit_evidence` | Submit evidencia |
| `resolve_dispute` | Resuelve (árbitro) |

#### Config Instructions (4)
| Instrución | Descripción |
|------------|-------------|
| `initialize` | Inicializa config global |
| `update_config` | Actualiza config |
| `pause` | Pausa programa |
| `unpause` | Reanuda programa |

### 4.4 Validaciones On-Chain

```rust
// No self-hiring
require!(
    ctx.accounts.user.key() != ctx.accounts.job.client,
    EscrowError::SelfHiringNotAllowed
);

// Estado válido
require!(
    job.status == JobStatus::Funded,
    EscrowError::InvalidJobStatus
);

// Fondos suficientes
require!(
    deposited_amount >= required_amount,
    EscrowError::InsufficientFunds
);

// No double release
require!(
    released_amount == 0,
    EscrowError::AlreadyReleased
);

// Payouts suman 100%
let total: u8 = members.iter().map(|m| m.payout_percentage).sum();
require!(total == 100, EscrowError::InvalidPayoutPercentage);
```

---

## 5. API Specification

### 5.1 REST Endpoints

#### Auth
```
POST   /api/auth/verify        # Verificar firma de wallet
POST   /api/auth/disconnect     # Desconectar wallet
```

#### Users
```
GET    /api/users/me           # Usuario actual
POST   /api/users              # Crear usuario
PUT    /api/users/me           # Actualizar usuario
POST   /api/users/wallets      # Agregar wallet
PUT    /api/users/wallets/:id  # Actualizar wallet
DELETE /api/users/wallets/:id  # Eliminar wallet
```

#### Jobs
```
GET    /api/jobs               # Listar jobs (filtros: status, category)
POST   /api/jobs               # Crear job
GET    /api/jobs/:id           # Ver job
PUT    /api/jobs/:id           # Actualizar job
DELETE /api/jobs/:id           # Eliminar (solo DRAFT)
POST   /api/jobs/:id/fund      # Publicar y fondear
POST   /api/jobs/:id/accept    # Aceptar job
POST   /api/jobs/:id/submit    # Entregar trabajo
POST   /api/jobs/:id/approve   # Aprobar trabajo
POST   /api/jobs/:id/reject    # Rechazar trabajo
POST   /api/jobs/:id/cancel    # Cancelar job
```

#### Milestones
```
GET    /api/jobs/:id/milestones    # Listar hitos
POST   /api/jobs/:id/milestones    # Crear hito
PUT    /api/milestones/:id         # Actualizar hito
POST   /api/milestones/:id/approve # Aprobar hito
POST   /api/milestones/:id/reject  # Rechazar hito
```

#### Disputes
```
GET    /api/disputes           # Listar disputas (admin/arbiter)
GET    /api/jobs/:id/dispute   # Ver disputa de job
POST   /api/jobs/:id/dispute   # Abrir disputa
POST   /api/disputes/:id/evidence # Submit evidencia
POST   /api/disputes/:id/resolve # Resolver disputa
GET    /api/disputes/:id/summary  # Obtener resumen IA
```

#### Teams
```
GET    /api/teams              # Listar equipos del usuario
POST   /api/teams              # Crear equipo
GET    /api/teams/:id          # Ver equipo
PUT    /api/teams/:id          # Actualizar equipo
POST   /api/teams/:id/members # Agregar miembro
PUT    /api/teams/:id/members/:uid # Actualizar miembro
DELETE /api/teams/:id/members/:uid # Eliminar miembro
```

#### Notifications
```
GET    /api/notifications       # Listar notificaciones
PUT    /api/notifications/:id/read # Marcar como leída
WS     /api/ws                 # WebSocket para real-time
```

### 5.2 Request/Response Examples

#### POST /api/jobs
```json
// Request
{
  "title": "Landing Page para SaaS",
  "description": "Diseño y desarrollo de landing page...",
  "amount": 5000000000,
  "deadline": 1714560000,
  "category": "web_development",
  "milestones": [
    {"title": "Wireframes", "amount": 1000000000},
    {"title": "Desarrollo", "amount": 3000000000},
    {"title": "Entrega final", "amount": 1000000000}
  ]
}

// Response
{
  "id": "job_abc123",
  "pda_address": "7xXxXxXxXxXxXxXxXxXxXxXxXxXxXxXxXxXxXxXxXxXxXx",
  "status": "draft",
  "created_at": 1712000000
}
```

#### POST /api/jobs/:id/fund
```json
// Request
{
  "wallet_address": "XxxXxxXxxXxxXxxXxxXxxXxxXxxXxxXxxXxxX"
}

// Response
{
  "tx_signature": "5xXxXxXxXxXxXxXxXxXxXxXxXxXxXxXxXxXxXxXxXxXxXxXxXxXx",
  "status": "funded",
  "deposited_amount": 5250000000,
  "fee_amount": 250000000
}
```

---

## 6. Database Schema (PostgreSQL)

### 6.1 Tables

```sql
-- Users cache (sincronizado con on-chain)
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    wallet_principal TEXT UNIQUE NOT NULL,
    username TEXT,
    bio TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Teams
CREATE TABLE teams (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    pda_address TEXT UNIQUE,
    owner_id UUID REFERENCES users(id),
    name TEXT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Team members
CREATE TABLE team_members (
    team_id UUID REFERENCES teams(id) ON DELETE CASCADE,
    user_id UUID REFERENCES users(id),
    role TEXT NOT NULL,
    department TEXT,
    payout_percentage INTEGER CHECK (payout_percentage >= 0 AND payout_percentage <= 100),
    is_active BOOLEAN DEFAULT true,
    PRIMARY KEY (team_id, user_id)
);

-- Jobs cache
CREATE TABLE jobs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    pda_address TEXT UNIQUE,
    client_id UUID REFERENCES users(id),
    freelancer_id UUID REFERENCES users(id),
    team_id UUID REFERENCES teams(id),
    status TEXT NOT NULL,
    amount BIGINT NOT NULL,
    fee_amount BIGINT,
    title TEXT NOT NULL,
    description TEXT,
    category TEXT,
    deadline TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Milestones
CREATE TABLE milestones (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    job_id UUID REFERENCES jobs(id) ON DELETE CASCADE,
    index_num INTEGER NOT NULL,
    title TEXT NOT NULL,
    description TEXT,
    amount BIGINT NOT NULL,
    status TEXT DEFAULT 'pending',
    deadline TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Disputes
CREATE TABLE disputes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    job_id UUID REFERENCES jobs(id),
    pda_address TEXT,
    opened_by UUID REFERENCES users(id),
    reason TEXT,
    status TEXT DEFAULT 'open',
    ai_summary TEXT,
    resolution JSONB,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    resolved_at TIMESTAMPTZ
);

-- AI Reports
CREATE TABLE ai_reports (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    dispute_id UUID REFERENCES disputes(id),
    output_json JSONB NOT NULL,
    model_version TEXT,
    confidence FLOAT,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Notifications
CREATE TABLE notifications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id),
    type TEXT NOT NULL,
    title TEXT NOT NULL,
    message TEXT,
    data JSONB,
    read BOOLEAN DEFAULT false,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Audit logs
CREATE TABLE audit_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tx_signature TEXT UNIQUE,
    action TEXT NOT NULL,
    actor_wallet TEXT,
    job_id UUID,
    metadata JSONB,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Financial ledger
CREATE TABLE financial_ledger (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    job_id UUID REFERENCES jobs(id),
    user_id UUID REFERENCES users(id),
    tx_signature TEXT UNIQUE,
    type TEXT NOT NULL,
    amount BIGINT NOT NULL,
    from_wallet TEXT,
    to_wallet TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW()
);
```

---

## 7. Flujo de Eventos

### 7.1 Sync Loop

```
1. Usuario firma transacción en Frontend
2. Frontend llama a API
3. API construye transacción via SDK
4. Transacción enviada a Solana
5. Solana procesa (finalized)
6. Helius detecta evento
7. Webhook a /api/webhooks/solana
8. API actualiza PostgreSQL
9. WebSocket emite evento
10. Frontend recibe update en tiempo real
```

### 7.2 Webhook Handler

```rust
#[post("/webhooks/solana")]
async fn solana_webhook(
    State(state): State<AppState>,
    Json(payload): Json<SolanaWebhookPayload>,
) -> Result<Json<WebhookResponse>, Status> {
    // Verificar firma del webhook
    verify_webhook_signature(&payload)?;
    
    // Ignorar duplicados (idempotencia)
    if state.db.exists(&payload.signature).await {
        return Ok(Json(WebhookResponse { processed: false }));
    }
    
    // Parsear evento
    match payload.event_type {
        "JobCreated" => handle_job_created(&state, &payload).await?,
        "JobFunded" => handle_job_funded(&state, &payload).await?,
        "WorkSubmitted" => handle_work_submitted(&state, &payload).await?,
        "DisputeRaised" => handle_dispute_raised(&state, &payload).await?,
        "DisputeResolved" => handle_dispute_resolved(&state, &payload).await?,
        _ => {}
    }
    
    // Guardar para idempotencia
    state.db.insert_signature(&payload.signature).await?;
    
    // Emitir WebSocket
    state.ws.broadcast(&payload).await?;
    
    Ok(Json(WebhookResponse { processed: true }))
}
```

---

## 8. Seguridad

### 8.1 Autenticación

- Wallet-based authentication (sign-message)
- JWT para sesiones
- Verificación de firma on-chain

### 8.2 Autorización

- RBAC en endpoints
- Validación on-chain de roles
- Rate limiting por wallet

### 8.3 Cifrado

- E2EE para mensajes de chat
- TLS para todas las comunicaciones
- Secrets en environment variables

---

_Last updated: 2026-03-22_
