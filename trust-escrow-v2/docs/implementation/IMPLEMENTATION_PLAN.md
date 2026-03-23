# Implementation Plan
## Trust Work Escrow v2

---

## 1. Resumen de Fases

| Fase | Descripción | Duración | Entregable |
|------|-------------|----------|------------|
| **0** | Setup & Planning | 1 día | Estructura de carpetas, docs completos |
| **1** | Smart Contract Core | 2-3 días | 17 instrucciones funcionando |
| **2** | Rust SDK | 1-2 días | SDK publicable en crates.io |
| **3** | Backend API | 2-3 días | API REST + WebSocket |
| **4** | Frontend MVP | 3-4 días | Dashboard funcional |
| **5** | Integración & Testing | 2 días | Sistema completo |
| **6** | Polish & Deploy | 1 día | Producción |

**Total estimado:** 12-16 días

**Deadline Hackathon:** 23 de marzo 2026, 23:30 UTC

**Protocolo de entrega por fase:**
1. Actualizar planning con checkmarks de lo realizado
2. Crear reporte de lo realizado en markdown
3. Commit después de cada fase completada

---

## 2. Fase 0: Setup & Planning

### 2.1 Objetivos
- [ ] Crear estructura de carpetas
- [ ] Completar documentación
- [ ] Responder preguntas
- [ ] Configurar workspace
- [ ] **Crear carpeta docs/ dentro de cada módulo** (sdk, cli, backend, etc.) con documentación educativa

### 2.2 Estructura de Carpetas

```
trust-escrow-v2/
├── programs/                 # Anchor workspace
├── sdk/                      # Rust SDK crate
├── backend/                  # Rust Axum API
├── frontend/                 # Next.js app
├── cli/                      # Rust CLI
├── tui/                      # Rust TUI
├── docs/                     # Documentación
└── scripts/                  # Deploy scripts
```

**Nota:** Cada carpeta (sdk, cli, backend, etc.) es un proyecto independiente con su propio README.md.

### 2.3 Configuración Inicial

```bash
# Anchor workspace
mkdir -p programs && cd programs
anchor init trust-escrow-v2

# Rust SDK
cargo new sdk && cd sdk
cargo add serde serde_json solana-sdk anchor-client

# Backend
cargo new backend && cd backend
cargo add axum tokio sqlx

# Frontend
npx create-next-app@latest frontend
```

---

## 3. Fase 1: Smart Contract

### 3.1 Duración: 2-3 días

### 3.2 Entregables

- [ ] Account structs (Config, User, Job, Team, Milestone, Dispute)
- [ ] 17 instrucciones implementadas
- [ ] Tests de integración
- [ ] IDL generado
- [ ] **Carpeta docs/** en el proyecto con explicación educativa de las funcionalidades
- [ ] **Compilar y testear** después de cada fase de código, luego commit

### 3.3 Instrucciones por Orden

#### Día 1: Config + User

```
□ initialize_config
□ update_config
□ pause
□ unpause
□ create_user
□ add_wallet
□ set_active_wallet
□ update_user
```

#### Día 2: Jobs + Teams

```
□ create_team
□ add_member
□ remove_member
□ update_team
□ create_job
□ publish_job
□ accept_job
□ submit_work
□ approve_work
□ reject_work
□ cancel_job
```

#### Día 3: Milestones + Disputes

```
□ add_milestone
□ approve_milestone
□ reject_milestone
□ raise_dispute
□ submit_evidence
□ resolve_dispute
□ withdraw_treasury
```

### 3.4 Tests

```typescript
describe('Config', () => {
  it('initialize config');
  it('update config');
  it('pause/unpause');
});

describe('User', () => {
  it('create user');
  it('add wallet');
  it('set active wallet');
});

describe('Job', () => {
  it('create job');
  it('publish job');
  it('accept job');
  it('submit work');
  it('approve work');
  it('reject work');
  it('cancel job');
});

describe('Disputes', () => {
  it('raise dispute');
  it('resolve dispute');
});
```

---

## 4. Fase 2: Rust SDK

### 4.1 Duración: 1-2 días

### 4.2 Entregables

- [ ] Crate publicable en crates.io
- [ ] Documentación con examples
- [ ] Tests unitarios

### 4.3 Estructura del SDK

```rust
// sdk/src/lib.rs
pub mod client;
pub mod instructions;
pub mod pda;
pub mod types;

pub use client::CofreClient;
pub use types::*;
```

### 4.4 Ejemplo de Uso

```rust
use escrow_sdk::CofreClient;

#[tokio::main]
async fn main() -> Result<()> {
    let client = CofreClient::new(
        "https://api.devnet.solana.com",
        "/path/to/keypair.json",
    )?;
    
    // Create job
    let job = client.create_job()
        .title("Landing Page")
        .amount(5_000_000_000) // 5 SOL
        .deadline(1714560000)
        .send()
        .await?;
    
    println!("Job created: {}", job.pda_address);
    Ok(())
}
```

---

## 5. Fase 3: Backend API

### 5.1 Duración: 2-3 días

### 5.2 Entregables

- [ ] REST API endpoints
- [ ] Helius Webhooks para eventos Solana
- [ ] Database integration
- [ ] Auth middleware
- [ ] README.md con documentación del API

### 5.3 Estructura del Backend

```
backend/
├── src/
│   ├── main.rs
│   ├── routes/
│   │   ├── mod.rs
│   │   ├── auth.rs
│   │   ├── jobs.rs
│   │   ├── users.rs
│   │   ├── teams.rs
│   │   ├── disputes.rs
│   │   └── notifications.rs
│   ├── services/
│   │   ├── mod.rs
│   │   ├── job_service.rs
│   │   ├── user_service.rs
│   │   └── notification_service.rs
│   ├── db/
│   │   ├── mod.rs
│   │   ├── postgres.rs
│   │   └── mongodb.rs
│   └── middleware/
│       ├── mod.rs
│       ├── auth.rs
│       └── rate_limit.rs
└── Cargo.toml
└── README.md
```

### 5.4 Endpoints Prioritarios

```rust
// Prioridad 1: Core functionality
POST   /api/jobs              // Crear job
GET    /api/jobs/:id          // Ver job
POST   /api/jobs/:id/fund     // Publicar y fondear
POST   /api/jobs/:id/accept   // Aceptar
POST   /api/jobs/:id/submit   // Entregar
POST   /api/jobs/:id/approve  // Aprobar

// Prioridad 2: Users & Teams
GET    /api/users/me
POST   /api/users/wallets
GET    /api/teams

// Prioridad 3: Disputes
POST   /api/disputes
POST   /api/disputes/:id/resolve

// Prioridad 4: Notifications
GET    /api/notifications
```

**Flujo de Aplicación a Jobs:**
```
POST   /api/jobs/:id/apply        // Freelancer aplica
GET    /api/jobs/:id/applications // Cliente ve aplicaciones
POST   /api/applications/:id/accept   // Cliente acepta
POST   /api/applications/:id/reject   // Cliente rechaza
```

---

## 6. Fase 4: Frontend MVP

### 6.1 Duración: 3-4 días

### 6.2 Entregables

- [ ] Wallet connect
- [ ] Dashboard
- [ ] Crear job
- [ ] Ver jobs
- [ ] Detalle de job
- [ ] Aprobar/Rechazar
- [ ] Sistema de disputas básico

### 6.3 Estructura del Frontend

```
frontend/
├── app/
│   ├── page.tsx                    # Landing
│   ├── layout.tsx                 # Root layout
│   ├── globals.css
│   ├── dashboard/
│   │   ├── page.tsx               # Dashboard
│   │   ├── jobs/
│   │   │   ├── page.tsx           # Lista de jobs
│   │   │   ├── new/page.tsx       # Crear job
│   │   │   └── [id]/page.tsx      # Detalle job
│   │   └── settings/page.tsx
│   ├── jobs/
│   │   ├── page.tsx               # Marketplace
│   │   └── [id]/page.tsx          # Detalle público
│   └── api/
│       └── [...trpc]/route.ts     # tRPC or REST
├── components/
│   ├── ui/                        # shadcn/ui components
│   ├── job-card.tsx
│   ├── wallet-button.tsx
│   ├── job-form.tsx
│   └── dispute-modal.tsx
├── lib/
│   ├── sdk.ts                     # API client
│   ├── SolanaProvider.tsx
│   └── utils.ts
└── hooks/
    ├── useWallet.ts
    ├── useJobs.ts
    └── useNotifications.ts
```

### 6.4 Pantallas Prioritarias

#### Día 1: Core UI + Wallet
- [ ] Setup Next.js + Tailwind + shadcn/ui
- [ ] Wallet Connect integration
- [ ] Auth flow
- [ ] Layout base

#### Día 2: Dashboard + Jobs List
- [ ] Dashboard page
- [ ] Jobs list
- [ ] Job cards
- [ ] Filters

#### Día 3: Create Job + Job Detail
- [ ] Create job form
- [ ] Job detail page
- [ ] Actions (fund, accept, submit)
- [ ] Status visualization

#### Día 4: Disputes + Polish
- [ ] Dispute flow
- [ ] Dispute resolution UI
- [ ] Perfil de freelancer
- [ ] Perfil de equipo con listado de miembros
- [ ] Ver perfil de cada miembro
- [ ] Loading states
- [ ] Error handling
- [ ] Responsive design

---

## 7. Fase 5: Integración & Testing

### 7.1 Duración: 2 días

### 7.2 Entregables

- [ ] Integración frontend-backend
- [ ] Tests E2E
- [ ] Bug fixes
- [ ] Performance optimization

### 7.3 Checklist de Integración

```
□ Wallet connect working
□ Create job → backend → blockchain
□ Fund job → blockchain event → UI update
□ Accept job → UI update
□ Submit work → UI update
□ Approve work → funds transferred
□ Reject work → dispute created
□ Resolve dispute → funds distributed
□ Notifications real-time
```

---

## 8. Fase 6: Polish & Deploy

### 8.1 Duración: 1 día

### 8.2 Entregables

- [ ] Frontend deploy (Vercel)
- [ ] Backend deploy (Railway/Render)
- [ ] Database setup (Supabase/Neon)
- [ ] Environment variables
- [ ] Monitoring setup
- [ ] Video demo

### 8.3 Deploy Checklist

```bash
# Frontend (Vercel)
vercel --prod

# Backend (Railway)
railway up

# Database migrations
sqlx migrate run

# Smart contract
anchor build
anchor deploy --provider.cluster devnet
```

---

## 9. Recursos Necesarios

### 9.1 APIs Externas

| Servicio | Uso | Costo |
|----------|-----|-------|
| Helius RPC | Blockchain access | Freemium |
| Helius Webhooks | Event sync | $29/mo |
| Supabase | PostgreSQL + Auth | Freemium |
| Vercel | Frontend hosting | Freemium |
| Railway | Backend hosting | Pay-as-you-go |

### 9.2 Herramientas

| Herramienta | Uso |
|-------------|-----|
| GitHub Actions | CI/CD |
| Cargo | Rust package manager |
| Anchor | Solana framework |
| Next.js | React framework |
| Tailwind | Styling |

---

## 10. Milestones

### 10.1 Milestone 1: Smart Contract
**Fecha objetivo:** Día 3
**Entregable:** Programa desplegado en devnet con IDL

### 10.2 Milestone 2: SDK + Backend
**Fecha objetivo:** Día 5
**Entregable:** API funcionando con jobs CRUD

### 10.3 Milestone 3: Frontend MVP
**Fecha objetivo:** Día 9
**Entregable:** Dashboard funcional con wallet connect

### 10.4 Milestone 4: Sistema Completo
**Fecha objetivo:** Día 12
**Entregable:** Flujo completo funcionando

### 10.5 Milestone 5: Hackathon Submission
**Fecha objetivo:** Día 14
**Entregable:** Video demo + repo público

---

## 11. Equipo (1 persona)

| Rol | Responsabilidades |
|-----|-------------------|
| **Full-stack Dev** | Todo 😄 |

### Time Management

```
08:00 - 10:00 │ Coding (fresh mind)
10:00 - 10:30 │ Break
10:30 - 13:00 │ Coding
13:00 - 14:00 │ Lunch
14:00 - 17:00 │ Coding
17:00 - 17:30 │ Review + Documentation
17:30 - 18:00 │ Daily standup
```

---

_Last updated: 2026-03-22_
