# Proposal: mvp-dynamic-roles-jobs
## Intent
One account acts as client on own jobs and freelancer on others, optionally arbiter. Same email publishes Job A (publish wallet) and works Job B (apply wallet). Job.client immutable (job.rs:385 CannotWorkOnOwnJob). Reuse SupportTicketMetadata + POST /jobs/:job_id/support in repository.rs/routes.rs.
## Scope
### In Scope
- Jobs: Crear, Publicados (client==me), Aceptados (freelancer==me), Historial, Disputas->Abiertas/Historial, Arbitraje (conditional), Soporte, Config, Saldo 3 buckets
- job_participants per-job role; user_wallets {publish,apply}; 1 wallet auto, 2+ picker per action
- Support ticket job-related/tecnico -> POST /support, resolve POST /support/resolve
- Arbiter menu visible if role==arbiter OR ArbiterPool.contains(pubkey) (dispute.rs:435 ArbiterCannotBeParty, GET /arbiter-pool): Asignadas/Historial/Saldo/Rechazar(reason->admin)
- Historial filters estado/rol/fecha/titulo/monto/con-sin disputa; funds getBalance before relay
- Rules client!=applicant, wallet_client!=wallet_freelancer, arbiter!=client&&freelancer (dispute.rs:263 raise_dispute InProgress|Submitted + CaseAlreadyOpen)
### Out of Scope
- Anchor rewrite, tokenomics, auto-assign arbiter
## Capabilities
### New Capabilities
- jobs-navigation, dynamic-roles, multi-wallet (default-vs-picker), job-history, disputes-scoped, support-tickets, arbitration-role
### Modified Capabilities
- None
## Approach
Hybrid Explore#3. Keep User.role for admin/arbiter/guest, alias wallet_pubkey->publish. Add user_wallets+job_participants. Drop DashboardRole; single Jobs accordion. Enforce CannotWorkOnOwnJob + ArbiterCannotBeParty; support reuses off-chain; arbiter via GET /arbiter-pool; fees ARBITER_FEE_BPS_PER_PARTY*2 for Saldo.
## Support & Arbitration
- Support: OpenSupportTicket needs InProgress|Submitted, blocked if Dispute (CaseAlreadyOpen). PDA b"support"; advisor resolves -> cancelled+refund. Soporte top-level.
- Arbitraje (isArbiter=role==arbiter||pool.contains): Asignadas (arbiter==me&&ArbiterAssigned), Historial (Resolved&&me), Saldo (sum ArbitrationEscrow), Rechazar (reason pending authority).
- Saldo: Publicados(gastado) client==me, Realizados(ganado) freelancer==me, Arbitrajes fee sum; 3 cards + delta.
## Affected Areas
route.rs (Mod /jobs/*,/disputes/*,/support/*,/arbitraje/*), sidebar.rs (Mod single Jobs+Arbitraje+Soporte), features/jobs/** (New), guest.rs (wallets Vec), backend/api/src/* (UserWallet/JobParticipant+Saldo), phantom.rs (SIWS picker), routes.rs (Reuse /support,/arbiter-pool)
## Risks
Diverge Med->on-chain final; Self-apply Med->400+CannotWorkOnOwnJob; Arbiter==party Low->pool+ArbiterCannotBeParty; SIWS Med->JWT; Clash Low->CaseAlreadyOpen; Reject Med->reason+admin
## Rollback Plan
Flag jobs-navigation revert /dashboard/client|freelancer; keep legacy role/wallet; drop tables restores single wallet; hide Support/Arbitraje by flag.
## Dependencies
dispute.rs:263 raise_dispute, :435 ArbiterCannotBeParty, ArbiterPool, OpenSupportTicket; MetadataRepository additive
## Success Criteria
- [ ] Same email creates+applies; 1 auto 2+ picker
- [ ] Self-apply 400 (job.rs:385)
- [ ] Publicados/Aceptados by participant.role
- [ ] Historial filters + Soporte works
- [ ] Arbitraje only if pool.contains; Saldo 3 buckets
## Open Questions
- Personal app = advisor vs authority vs support_staff?
- Rechazar flow needs POST /disputes/reject + admin reassign
- Fee source on-chain vs off-chain?
