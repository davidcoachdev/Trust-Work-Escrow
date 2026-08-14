# Backend v3 route matrix

**Estado:** planificado; la API, terminal/TUI, application service y SDK v3 no
están implementados. Las instrucciones del contrato sí están presentes en el
source revisado, pero eso no prueba runtime, deploy ni sincronización.

Cada fila conserva la frontera obligatoria:
`endpoint/comando → application service → trust-escrow-sdk → Solana → DB projection/status`.
Las lecturas usan `read-only`; las mutaciones declaran `user-signed` o
`server-signed` explícitamente. `finality` es observada por SDK y nunca inventada
por DB.

| Endpoint / command | Application service | SDK operation | On-chain authority | DB projection / status |
|---|---|---|---|---|
| `POST /v1/config` / `config init` | `ConfigService.initialize` | `initialize_config` | bootstrap authority | `config`; planned/current |
| `POST /v1/config/pause` / `config pause` | `ConfigService.pause` | `pause_program` | `config.authority` | `config`; planned/current |
| `POST /v1/config/unpause` / `config unpause` | `ConfigService.unpause` | `unpause_program` | `config.authority` | `config`; planned/current |
| `PATCH /v1/config/treasury` / `config treasury set` | `ConfigService.updateTreasury` | `update_treasury` | `config.authority` | `config`; planned/current |
| `PATCH /v1/config/arbitration-treasury` / `config arbitration-treasury set` | `ConfigService.updateArbitrationTreasury` | `update_arbitration_treasury` | `config.authority` | `config`; planned/current |
| `POST /v1/treasury/withdraw` / `treasury withdraw` | `TreasuryService.withdraw` | `withdraw_treasury` | treasury signer | `transaction_intents`, audit; planned/finality |
| `POST /v1/arbitration-treasury/withdraw` / `arbitration withdraw` | `TreasuryService.withdrawArbitration` | `withdraw_arbitration` | arbitration treasury signer | `transaction_intents`, audit; planned/finality |
| `POST /v1/jobs` / `job create` | `JobService.create` | `create_job` | `job.client` | `job`; planned/current |
| `POST /v1/jobs/:id/fund` / `job fund` | `JobService.depositFunds` | `deposit_funds` | `job.client` | `job`, intent; planned/finality |
| `POST /v1/jobs/:id/applications` / `job apply` | `ApplicationService.apply` | `apply_to_job` | applicant | `application`, `job`; planned/current |
| `POST /v1/jobs/:id/applications/:index/accept` / `job application accept` | `ApplicationService.accept` | `accept_application` | `job.client` | `job`, application; planned/current |
| `POST /v1/jobs/:id/applications/cleanup` / `job applications cleanup` | `ApplicationService.cleanup` | `cleanup_applications` | permissionless, validated accounts | application tombstones; planned/current |
| `GET /v1/config` / `config get` | `ConfigQuery.get` | `read_config` | none; read-only RPC | `config`; planned/current/stale |
| `GET /v1/arbiters/pool` / `arbiter-pool get` | `ArbiterQuery.getPool` | `read_arbiter_pool` | none; read-only RPC | arbiter pool; planned/current/stale |
| `GET /v1/jobs/:id` / `job get` | `JobQuery.get` | `read_job` | none; read-only RPC | `job`; planned/current/stale/closed |
| `GET /v1/jobs/:id/applications/:index` / `job application get` | `ApplicationQuery.get` | `read_application` | none; read-only RPC | `application`; planned/current/stale/closed |
| `POST /v1/jobs/:id/submit` / `job submit` | `JobService.submitWork` | `submit_work` | `job.freelancer` | `job`, intent; planned/finality |
| `POST /v1/jobs/:id/auto-approve` / `job auto-approve` | `JobService.autoApprove` | `auto_approve_work` | permissionless after deadline | `job`, intent; planned/finality |
| `POST /v1/jobs/:id/approve` / `job approve` | `JobService.approveWork` | `approve_work` | `job.client` | `job`, intent; planned/finality |
| `POST /v1/jobs/:id/reject` / `job reject` | `JobService.rejectWork` | `reject_work` | `job.client` | `job`, audit; planned/current |
| `POST /v1/jobs/:id/cancel` / `job cancel` | `JobService.cancel` | `cancel_job` | `job.client` | `job`, tombstone; planned/closed |
| `POST /v1/jobs/:id/pause` / `job pause` | `JobService.pause` | `pause_job` | `job.client` | `job`; planned/current |
| `POST /v1/jobs/:id/unpause` / `job unpause` | `JobService.unpause` | `unpause_job` | `job.client` | `job`; planned/current |
| `POST /v1/jobs/:id/expire-pause` / `job expire-pause` | `JobService.expirePaused` | `expire_paused_job` | `job.client` | `job`, tombstone; planned/closed |
| `POST /v1/arbiters/pool` / `arbiter-pool create` | `ArbiterService.createPool` | `create_arbiter_pool` | `config.authority` | arbiter pool; planned/current |
| `POST /v1/arbiters` / `arbiter add` | `ArbiterService.add` | `add_arbiter` | `config.authority` | arbiter pool; planned/current |
| `DELETE /v1/arbiters/:key` / `arbiter remove` | `ArbiterService.remove` | `remove_arbiter` | `config.authority` | arbiter pool; planned/current |
| `POST /v1/jobs/:id/dispute` / `dispute raise` | `DisputeService.raise` | `raise_dispute` | client or freelancer | dispute, escrow, job; planned/current |
| `POST /v1/jobs/:id/dispute/accept` / `dispute accept` | `DisputeService.accept` | `accept_dispute` | other job party | dispute, escrow; planned/current |
| `POST /v1/jobs/:id/dispute/evidence` / `dispute evidence add` | `DisputeService.submitEvidence` | `submit_evidence` | client or freelancer | evidence, dispute; planned/current |
| `POST /v1/jobs/:id/dispute/intervention` / `dispute intervene` | `DisputeService.requestIntervention` | `request_platform_intervention` | client or freelancer | dispute, audit; planned/current |
| `GET /v1/jobs/:id/dispute` / `dispute get` | `DisputeQuery.get` | `read_dispute_and_evidence` | none; read-only RPC | dispute/evidence; planned/current/stale |
| `POST /v1/jobs/:id/dispute/arbiter` / `dispute assign-arbiter` | `DisputeService.assignArbiter` | `assign_arbiter` | config authority + pool authority | dispute; planned/current |
| `POST /v1/jobs/:id/dispute/resolve` / `dispute resolve` | `DisputeService.resolve` | `resolve_dispute` | assigned arbiter | dispute; planned/current |
| `POST /v1/jobs/:id/dispute/platform-resolve` / `dispute platform-resolve` | `DisputeService.resolvePlatform` | `resolve_platform_case` | `config.advisor` | dispute; planned/current |
| `POST /v1/jobs/:id/dispute/finalize` / `dispute finalize` | `DisputeService.finalizePayouts` | `finalize_dispute_payouts` | arbiter or advisor, client account context | job, escrow, dispute, tombstone; planned/finality |
| `POST /v1/jobs/:id/dispute/evidence/cleanup` / `dispute evidence cleanup` | `DisputeService.cleanupEvidence` | `cleanup_dispute_evidence` | arbiter or advisor | evidence tombstones; planned/closed |
| `POST /v1/jobs/:id/support` / `support open` | `SupportService.open` | `open_support_ticket` | client or freelancer | support ticket; planned/current |
| `POST /v1/jobs/:id/support/resolve` / `support resolve` | `SupportService.resolve` | `resolve_support_ticket` | `config.advisor` plus client account context | ticket, job, tombstone; planned/closed |
| `POST /v1/jobs/:id/milestones` / `milestone create` | `MilestoneService.create` | `create_milestone` | job client account context | milestone, job; planned/current |
| `POST /v1/jobs/:id/milestones/:index/submit` / `milestone submit` | `MilestoneService.submit` | `submit_milestone` | `job.freelancer` | milestone; planned/current |
| `POST /v1/jobs/:id/milestones/:index/approve` / `milestone approve` | `MilestoneService.approve` | `approve_milestone` | `job.client` | milestone, job, intent; planned/finality |
| `POST /v1/jobs/:id/milestones/:index/reject` / `milestone reject` | `MilestoneService.reject` | `reject_milestone` | `job.client` | milestone; planned/current |
| `GET /v1/jobs/:id/milestones/:index` / `milestone get` | `MilestoneQuery.get` | `read_milestone` | none; read-only RPC | milestone; planned/current/stale |
| `GET /v1/jobs/:id/support` / `support get` | `SupportQuery.get` | `read_support_ticket` | none; read-only RPC | support ticket; planned/current/stale |

## Estado y límites

- Todas las filas son contratos de ruta planificados; ninguna fila prueba que la
  API, TUI, SDK, worker o DB exista en runtime.
- `current`, `stale`, `divergent` y `closed` son estados de proyección. El estado
  contractual se vuelve a leer desde Solana mediante el SDK cuando hay conflicto.
- `submitted → processed → confirmed → finalized` solo se registra con evidencia
  de la transacción/commitment. `failed` y `reorged` no se inventan.
- Evidence PDA y digest externo siguen siendo conceptos separados; no hay claim de
  hash de evidencia on-chain.
