# Design: mvp-dynamic-roles-jobs

## Context
Escrow v3: `DashboardRole` single + `UserMetadata{role:String, wallet_pubkey:Option}` (metadata.rs:655, sidebar.rs, dashboard_layout.rs) fuerza toggle. On-chain `Job.client` immutable; `apply_to_job` exige `applicant != job.client` → `CannotWorkOnOwnJob` (job.rs:385). Disputas: `ticket.is_none()` → `CaseAlreadyOpen` (dispute.rs:263) y `InProgress|Submitted`; `assign_arbiter` verifica `ArbiterCannotBeParty` (dispute.rs:435). Propuesta v4: un email = client en propios + freelancer en ajenos + admin/arbiter simultáneo, menú único dinámico por `permissions` Vec.

## Goals / Non-Goals
**Goals**: Vec<Role>/permissions, `user_wallets` publish/apply, `job_participants` per-job, canvas kanban sin drag + `JobDetailPage`, historial unificado con filtros, disputas scoped, support `job_pda:Option`, arbitration condicional (`roles`∨`ArbiterPool`), `/admin` 7 subrutas, audit soft-delete + guard `WalletHasActiveJob`.
**Non-Goals**: Anchor rewrite, tokenomics, auto-assign arbiter, drag kanban.

## Technical Approach
Hybrid Explore#3. Allowlist único `metadata.rs` → `User{roles:Vec,permissions:Vec,is_active,created/updated/by,deleted_at}` + `user_wallets{email,pubkey,purpose:publish|apply|general}` + `job_participants{job_pda,email,role_per_job}` + `support_tickets{job_pda:Option}`. Frontend `MenuConfig{has(p)}` wildcard `admin:*` → `Sidebar(Vec)`. `route.rs` guards 403. Slices incrementales: un spec → commit → prueba en vivo (requisito usuario); flag `permissions-menu` off = fallback `DashboardRole` single + `roles[0]`.

## Architecture Decisions

| Decisión | Opciones | Tradeoff | Elección |
|---|---|---|---|
| Vec roles | Vec vs enum toggle | Vec combina menú sin toggle; alias migración | `roles:Vec, perms:Vec` + alias `role` legacy |
| Permisos source | allowlist vs DB | allowlist=drift detectable simple | `metadata.rs` único + test `frontend⊆backend` |
| Multi-wallet | 1 vs N | N habilita split publish/apply; picker | `user_wallets` 1..N, auto si len==1 else picker `signer_purpose` |
| Authority | global role vs `job_participants` | global no soporta dual | `job_participants` + creator auto `client` |
| Soft-delete | hard vs `is_active+deleted_at` | soft=auditoría | soft-only, filtro default `is_active=true` |
| Canvas | drag vs click→detail | drag complejo/permisos | kanban sin drag, click → `JobDetailPage` |

## System Overview

```
Wallet(SIWS per pubkey) → Auth → User{roles,perms} → MenuConfig.has(p)
                                            │              │
                                     Sidebar(Vec)    route.rs 403
                                            │
                    ┌───────────────────────┼───────────────────────┐
              Jobs/Canvas          Disputas/Arbitraje          Admin(7)
                    │                       │                      │
        job_participants → JobDetailPage ← disputes/support_tickets
                    │
     on-chain Job.client immutable (job.rs:385)
     dispute guards (dispute.rs:263/435)
```

## Data Model

```rust
User { email PK, roles:Vec, permissions:Vec, is_guest,
       created_at,updated_at,created_by,updated_by,is_active,deleted_at }
UserWallet { email FK, pubkey PK(44 bs58), purpose:publish|apply|general,
             label, created_at, is_active, deleted_at }
JobParticipant { job_pda FK, email FK, role_per_job:client|freelancer, wallet_pubkey, joined_at, is_active }
SupportTicket { id PK, pda_address, job_pda:Option<PDA>, opened_by, reason,
                status:Open|Resolved, resolved_by/at, audit cols..., is_active, deleted_at }
// jobs/applications/disputes/evidence: añadir mismas audit cols
```

Guard: `DELETE /users/:email/wallets/:pubkey` → 400 `WalletHasActiveJob` si `JobStatus∈{InProgress,Submitted}` o dispute `Active|EvidenceSubmitted|ArbiterAssigned`.

## Interfaces / Contracts

**API** (routes.rs, repository.rs):
```
GET  /config → {fee_bps:250, allowlist}
GET  /users/:email, PATCH /admin/users/:email {roles,permissions}
GET  /users/:email/wallets, POST {pubkey,purpose}, DELETE /:pubkey
GET  /jobs?email=&scope=published|applied|history&estado=&rol=&fecha=&titulo=&monto=&disputa=
GET  /jobs/history?email=... // via job_participants
GET  /jobs/:pda, POST /jobs/:pda/support {reason}
GET  /disputes?email=&scope=open|history, POST /disputes/:id/reject {reason≥20}
POST /support {job_pda:Option, reason}, POST /support/:id/resolve
GET  /admin/{users,permisos,asignaciones,wallets,metricas,tickets,disputas,contabilidad}
PATCH /admin/config {fee_bps} // admin:wallets
GET  /arbiter-pool → {arbiters:Vec<pubkey>}
```

**Frontend** (sidebar.rs, dashboard_layout.rs, route.rs, features/jobs/**):
```rust
MenuConfig { roles:Vec, perms:Vec } // has(p) exact||wildcard admin:*
Sidebar { roles:Vec<Role>, perms:Vec } // no DashboardRole single
KanbanBoard { columns:Vec<Column>, onClick: |pda| -> JobDetailPage }
JobDetailPage { desc, chat:Vec<Msg>, evidencias, estado, readOnly:bool }
AdminConsole { guard: has("admin:*"|"support:view"|"accountant") else 403 }
```
Guards `route.rs`: `has(required)` → 403 sin loop.

## Security

- **Self-apply**: off-chain `participant.email` client → 400 `CannotWorkOnOwnJob`; on-chain `applicant != job.client` (job.rs:385); `wallet_client != wallet_freelancer`.
- **Arbiter**: `ArbiterCannotBeParty` en `assign_arbiter` (dispute.rs:435) + `CaseAlreadyOpen`/`CannotDisputeAtStage` (dispute.rs:263).
- **SIWS per wallet**: `x-pubkey` debe matchear `signer_purpose`; `getBalance` antes de `relay`; publish para apply → warning.
- **RBAC**: `MenuConfig` single source; test drift `frontend ⊆ allowlist`.

## Testing Strategy

| Layer | Qué | Cómo |
|-------|-----|------|
| Unit | `has(p)` wildcard, Vec alias, filtro `is_active`, bs58 32B | Rust unit + InMemory repo |
| Integration | self-apply dual-wallet, arbiter party, delete guard, filtros historial AND | axum test client InMemory |
| E2E | menú combinado roles, canvas 6 cols, click→detail read-only, /admin 403 | Dioxus WASM guards |

## Threat Matrix
N/A — no routing shell, subprocess, VCS/PR automation, ni clasificación de ejecutables. Diseño cambia routing Dioxus (`route.rs` guards) pero no invoca shell/git/gh.

| Boundary | Cases | Aplicabilidad | Respuesta | RED |
|---|---|---|---|---|
| Docs paths | requirements.txt, README.sh | N/A: sin ejecución desde docs | — | — |
| Git selection | git -C, paths | N/A: fuera scope | — | — |
| Commit/Push/PR | staged, tracking, --head | N/A: sin VCS automation | — | — |

## Migration / Rollout
Flag `permissions-menu` off = fallback `DashboardRole` single + `roles[0]`, audit nullable. **Chained PRs por spec (slices incrementales)**:
1. `permissions-menu+audit-trail`: `metadata.rs` Vec+audit, repo filtros `is_active`, alias legacy.
2. `multi-wallet+dynamic-roles`: `user_wallets`, `job_participants`, SIWS per wallet, self-apply.
3. `jobs-navigation+job-history`: `MenuConfig`/`Sidebar(Vec)`, canvas, historial+filtros, `JobDetailPage`.
4. `disputes-scoped+support-tickets`: scoping, `job_pda:Option`, bandeja.
5. `arbitration-role+admin-console`: `ArbiterPool` SWR, `/admin` 7 subrutas, `fee_bps`.

Rollback: flag off restaura single role; hard DELETE sigue prohibido.

## Open Questions
- [ ] Nombres finales permisos treasury (`admin:wallets` vs `admin:treasury`) y scope editable
- [ ] Límite N wallets por email
