# Proposal: mvp-dynamic-roles-jobs
## Intent
One email = client own jobs + freelancer others + admin/arbiter simultáneo si permitido. Job.client immutable (job.rs:385). Menú único dinámico por `permissions`.

## Scope
### In Scope
- Jobs Crear/Publicados/Aceptados/Historial/Disputas/Arbitraje/Soporte/Config/Saldo; `job_participants`; `user_wallets {publish,apply}`
- Menú único `MenuConfig`+`Sidebar(Vec<Role>)`+canvas kanban+`JobDetailPage` (desc/chat/evidencias/estado)
- Support POST /jobs/:job_id/support + POST /support; ArbiterPool
- Admin `/admin` 7 subrutas + auditoría RFC2119 soft delete; wallet delete block si activo
### Out of Scope
- Anchor rewrite, tokenomics, auto-assign arbiter, drag kanban

## Capabilities
### New Capabilities
- `jobs-navigation`, `dynamic-roles`, `permissions-menu`, `multi-wallet`, `job-history`, `disputes-scoped`, `support-tickets`, `arbitration-role`, `admin-console`, `audit-trail`
### Modified Capabilities
- None

## Approach
Hybrid Explore#3. `User.role`->`User{roles:Vec,permissions:Vec}` e.g.`["admin:users","admin:support","jobs:create","jobs:view:own","jobs:apply","disputes:view","arbitration:assigned","config:wallet"]`. Add `user_wallets`+`job_participants`. `DashboardLayout`->`MenuConfig`->`Sidebar`.

## Navegación / Menú Único Dinámico
Verificado: sidebar DashboardRole single; metadata allowlist 5 sin Vec.
Modelo: `UserMetadata{roles:Vec,permissions:Vec}` multi-rol.
MenuConfig `has(p)`; Sidebar recibe `Vec<Role>` no `DashboardRole` único.
| Item | Si `has` | Submenú |
|------|----------|---------|
| Administración | `admin:*` | Usuarios/Permisos/Asignaciones/Wallets |
| Soporte | `support:view` | Tickets |
| Jobs | `jobs:view` | Crear/Borradores/Publicados+Canvas |
| Disputas | `disputes:view` | Métricas+canvas |
| Configuración | auth | App/Wallet/Saldos/Tickets |
Canvas sin drag, click->`JobDetailPage{desc,chat,evidencias,estado}` read-only historial:
- Cliente: Borrador,Publicado,En curso,Disputado,Cancelado,Terminado
- Freelancer: Solicitado,Aceptado,Rechazado,En curso,En disputas,Terminado
- Disputas: Solicitada,Rechazada,En curso,Resuelta
Guards route.rs.

## Support & Arbitration
Support InProgress|Submitted o técnico; Arbitraje Asignadas/Historial/Saldo/Rechazar.

## Administración & Auditoría
`/admin` guards `admin|support_tech|accountant`: métricas, users PATCH rol/permiso, jobs global, support bandeja, disputas, wallets fee_bps(250), accounting. Ticket `SupportTicket{job_pda:Option,opened_by,status:Open}`. Wallet DELETE 400 `WalletHasActiveJob` si InProgress/Submitted o Dispute Active; libre soft delete. Tablas MUST `created_at,updated_at,created_by,updated_by,is_active,deleted_at`; MUST filtrar `is_active`; MUST NOT hard DELETE.

## Affected Areas
| Area | Imp | Desc |
|------|-----|------|
| `app/src/ui/sidebar.rs` | Mod | Sidebar{roles,perms} condicional+kanban |
| `app/src/ui/dashboard_layout.rs` | Mod | deriva MenuConfig |
| `app/src/route.rs` | Mod | rutas permiso+guards |
| `app/src/features/jobs/**` | New | Canvas+JobDetailPage |
| `backend/api/src/metadata.rs` | Mod | UserMetadata Vec+AuditFields |
| `backend/api/src/repository.rs` | Mod | get_permissions+soft delete |
| `backend/api/src/routes.rs` | Mod | /admin/*, DELETE wallet |

## Risks
| Risk | Like | Mit |
|------|------|-----|
| Allowlist Vec break | High | alias flag |
| is_active olvido | Med | default filter |
| Permiso drift | Med | single service |

## Rollback Plan
Flag `permissions-menu` off -> `DashboardRole` single, `roles`->`role`, audit nullable.

## Dependencies
metadata.rs allowlist, repository.rs, routes.rs /support /arbiter-pool get_config, sidebar+layout

## Success Criteria
- [ ] Admin ve todo; freelancer no ve Administración
- [ ] `roles=[client,freelancer,admin]` submenús combinados sin toggle
- [ ] Canvas cliente 6 cols y freelancer 6 cols; click->detalle read-only historial
- [ ] `/admin/users` sin permiso ->403
- [ ] Ticket bandeja resolver ok; DELETE InProgress->400 soft delete ok

## Open Questions
- Nombres permisos y treasury editable?
