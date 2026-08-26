# Tasks: mvp-dynamic-roles-jobs

## Review Workload Forecast

| Field | Value |
|-------|-------|
| Estimated changed lines | 1800–2400 |
| 400-line budget risk | High |
| Chained PRs recommended | Yes |
| Suggested split | 5 PRs (2 specs/PR) → 10 slices (1 spec/commit) |
| Delivery strategy | ask-on-risk |
| Chain strategy | pending |

Decision needed before apply: Yes
Chained PRs recommended: Yes
Chain strategy: pending
400-line budget risk: High

### Suggested Work Units — 1 spec → 1 commit → live test

| # | Spec | Goal | PR | Test | Harness | Rollback |
|---|------|------|----|------|---------|----------|
|1|audit-trail|Audit cols+soft-delete|PR1|`cargo test is_active_filter`|`GET /wallets` hides inactive|cols nullable|
|2|permissions-menu|has(p) wildcard+Sidebar Vec|PR1|`cargo test has_wildcard`|`roles=[client,admin]` merged menu|flag off→single role|
|3|multi-wallet|user_wallets 1..N+bs58|PR2|`cargo test bs58_32B`|`POST /wallets`+picker|table drop|
|4|dynamic-roles|job_participants+self-apply|PR2|`cargo test self_apply`|`apply own job`→400|table drop|
|5|jobs-navigation|Jobs hierarchy+kanban+detail|PR3|`cargo test guard_403`|`/jobs/published` click→detail|routes flag|
|6|job-history|Historial AND filters|PR3|`cargo test history_and`|`GET /history?estado&disputa`|endpoint flag|
|7|disputes-scoped|Scoped open/history+metrics|PR4|`cargo test scoped`|`GET /disputes?scope=open`|filter flag|
|8|support-tickets|job_pda:Option bandeja|PR4|`cargo test ticket_option`|`POST /jobs/:id/support` vs `/support`|table flag|
|9|arbitration-role|Pool conditional+reject≥20|PR5|`cargo test pool_pool`|`reject`→PendingReassign|pool flag|
|10|admin-console|/admin 7 rutas+fee_bps|PR5|`cargo test admin_403`|`PATCH /admin/config`|routes flag|

Chain: `stacked-to-main` (PR→main) vs `feature-branch-chain` (PR1→tracker, PR2→PR1…). Orquestador pregunta.

## Wave 0: Foundation — PR1 (no deps)

- [ ] 0.1 `backend/api/src/models.rs` — `User{roles:Vec,perms:Vec,audit}` + mixin jobs/disputes. Acc: Given create When insert Then `created_by==actor&&is_active`. Commit: `feat(audit): add audit cols all tables`
- [ ] 0.2 `backend/api/src/repository.rs` — `WHERE is_active=true` default, no hard DELETE. Acc: Given 1 soft-deleted When list Then hidden. Commit: `feat(audit): soft-delete filter`
- [ ] 0.3 `backend/api/src/metadata.rs` — `UserMetadata Vec` + allowlist + alias legacy. Acc: Given `role="client"` Then `roles=["client"]`. Commit: `feat(permissions): Vec roles allowlist alias`
- [ ] 0.4 `backend/api/src/routes.rs` — `DELETE /wallets/:pubkey` →400 `WalletHasActiveJob` if InProgress/Submitted|Active dispute. Acc: Given InProgress When DELETE Then 400. Commit: `feat(audit): wallet delete guard`
- [ ] 0.5 `app/src/ui/dashboard_layout.rs,sidebar.rs` — `MenuConfig.has(p)` wildcard `admin:*`, `Sidebar(Vec)`. Acc: Given `[client,admin]` Then Jobs+Admin visibles sin toggle. Commit: `feat(menu): MenuConfig+Sidebar Vec`
- [ ] 0.6 `app/src/route.rs` — guards `has(required)`→403 + drift test `frontend⊆allowlist`. Acc: Given no `admin:users` When `/admin/users` Then 403. Commit: `feat(menu): route guards`

## Wave 1: Identity — PR2 (dep Wave0)

- [ ] 1.1 `backend/api/src/models.rs,repository.rs` — `UserWallet{pubkey bs58,purpose}` migrate `wallet_pubkey→publish`. Acc: Given legacy 1 wallet Then `purpose=publish`. Commit: `feat(wallet): user_wallets table`
- [ ] 1.2 `backend/api/src/routes.rs` — `GET/POST/DELETE /wallets`, `x-pubkey==signer_purpose`, `getBalance` before relay. Acc: Given 2 wallets When apply sin `signer_purpose` Then 400. Commit: `feat(wallet): CRUD+signer+funds`
- [ ] 1.3 `backend/api/src/models.rs` — `JobParticipant{job_pda,email,role_per_job}` creator auto `client`. Acc: Given alice crea A+aplica B Then `client`+`freelancer`. Commit: `feat(roles): job_participants`
- [ ] 1.4 `backend/api/src/routes.rs` — self-apply `CannotWorkOnOwnJob` (job.rs:385) + `ArbiterCannotBeParty` (dispute.rs:435). Acc: Given same email client When apply otra wallet Then 400. Commit: `feat(roles): self-apply guard`
- [ ] 1.5 `app/src/features/dashboard/config.rs` — picker auto(1)/select(2+), insufficient-funds banner. Acc: Given balance 0.01 When create 1 SOL Then blocked pre-relay. Commit: `feat(wallet): picker UX`

## Wave 2: Jobs — PR3 (dep Wave1)

- [ ] 2.1 `app/src/features/jobs/**,route.rs` — Jobs `Crear/Borradores/Publicados`, kanban 6 cols sin drag, click→`JobDetailPage{desc,chat,evidencias,estado}`. Acc: Given client When canvas Then 6 cols por status. Commit: `feat(jobs): hierarchy+kanban+detail`
- [ ] 2.2 `backend/api/src/routes.rs` — `GET /jobs?estado&rol&fecha&titulo&monto&disputa` AND. Acc: Given 10 When `estado=Terminado&disputa=true` Then solo Terminado+disputed. Commit: `feat(history): filters AND`
- [ ] 2.3 `backend/api/src/routes.rs` — `GET /jobs/history?email` via `job_participants`, detail read-only. Acc: Given Terminado When detail Then chat read-only. Commit: `feat(history): aggregation read-only`
- [ ] 2.4 `tests` — E2E canvas read-only, deep-link, empty `Sin trabajos`. Acc: Given historial When click Then badge `Disputa resuelta`. Commit: `test(jobs): history E2E`

## Wave 3: Disputes & Support — PR4 (dep Wave2)

- [ ] 3.1 `backend/api/src/routes.rs` — `GET /disputes?scope=open|history` scoped, metrics `3 abiertas·4.5 SOL`, canvas `Solicitada/Rechazada/En curso/Resuelta`. Acc: Given freelancer con dispute Active When Abiertas Then aparece; otro user no. Commit: `feat(disputes): scoped+metrics`
- [ ] 3.2 `backend/api/src/routes.rs` — `raise_dispute` guard `InProgress|Submitted`+`ticket.is_none()` else `CaseAlreadyOpen` (dispute.rs:263). Acc: Given Funded When raise Then 400. Commit: `feat(disputes): raise guard`
- [ ] 3.3 `backend/api/src/models.rs,routes.rs` — `SupportTicket{job_pda:Option}` + `POST /jobs/:id/support` + `POST /support` + `POST /support/:id/resolve` advisor-only. Acc: Given `job_pda=None` Then técnico Open. Commit: `feat(support): tickets job-bound+tech`
- [ ] 3.4 `app/src/features/dashboard/**` — bandeja `/admin/support` resolve. Acc: Given advisor When resolve 42 Then `Resolved+resolved_by`; freelancer→403. Commit: `feat(support): bandeja resolve`

## Wave 4: Arbitration & Admin — PR5 (dep Wave3)

- [ ] 4.1 `backend/api/src/routes.rs` — `GET /arbiter-pool`, `isArbiter=roles⊇arbiter∨pool.contains(pubkey)` SWR, `POST /disputes/:id/reject {reason≥20}`→PendingReassign. Acc: Given pool member Then Arbitraje visible; <20→400. Commit: `feat(arbitration): pool+reject`
- [ ] 4.2 `app/src/route.rs,backend/api/src/routes.rs` — `/admin` 7 subrutas guards `admin:*|support:view|accountant`→403. Acc: Given freelancer When `/admin/users` Then 403. Commit: `feat(admin): guards+7 subroutes`
- [ ] 4.3 `backend/api/src/routes.rs` — `PATCH /admin/users/:email {roles,perms}` audited. Acc: Given admin añade `arbiter` a bob Then bob ve Arbitraje. Commit: `feat(admin): user PATCH`
- [ ] 4.4 `backend/api/src/routes.rs` — `PATCH /admin/config {fee_bps:250}` `admin:wallets`, `/admin/accounting` escrow sum. Acc: Given 300 When new job Then 300 bps; old retaining. Commit: `feat(admin): fee_bps+accounting`
- [ ] 4.5 `tests` — E2E combined `roles=[client,freelancer,admin]` menu, 6 cols, detail read-only, /admin 403, bandeja, DELETE 400, reject. Acc: All Success Criteria proposal pass. Commit: `test(e2e): mvp verification`

## Dependency Graph

```
Wave0 ─→ Wave1 ─→ Wave2 ─→ Wave3 ─→ Wave4
Threat matrix: N/A — RED tests not applicable (design §Threat Matrix)
```

## Estimates

| Slice | Lines | Risk |
|-------|-------|------|
|audit-trail|250|Med|
|permissions-menu|300|High|
|multi-wallet|350|High|
|dynamic-roles|280|High|
|jobs-navigation|400|Med|
|job-history|220|Low|
|disputes-scoped|260|Med|
|support-tickets|250|Low|
|arbitration-role|280|Med|
|admin-console|450|High|
|Total|~3040|High|

Notes: 1 spec=1 commit→live proof, no time assumptions. Flag `permissions-menu` off→single role. Unit `has(p)`,Vec alias,`is_active`,bs58; integration self-apply dual-wallet, arbiter party, delete guard, AND filters; E2E menú combinado, canvas 6 cols, click→detail, /admin 403.
