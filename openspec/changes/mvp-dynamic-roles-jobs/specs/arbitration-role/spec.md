# arbitration-role Specification

## Purpose
Conditional arbiter role: visible if `User.role==arbiter` OR `ArbiterPool.contains(pubkey)` (GET /arbiter-pool). Submenu: Asignadas, Historial resueltas, Saldo arbitraje, Rechazar con justificación pendiente de admin.

## Requirements

### Requirement: Conditional Arbiter Visibility
The system SHALL show Arbitraje menu iff `isArbiter = (user.roles contains arbiter) OR (GET /arbiter-pool).arbiters contains wallet_pubkey`. Polling or SWR SHALL revalidate pool membership.

#### Scenario: Pool member sees menu
- GIVEN wallet `ARBI...` in ArbiterPool, user role is freelancer
- WHEN MenuConfig evaluates `isArbiter`
- THEN Arbitraje submenu appears (Asignadas/Historial/Saldo)

#### Scenario: Non-arbiter hidden
- GIVEN wallet not in pool and role != arbiter
- WHEN rendering Sidebar
- THEN Arbitraje menu is absent, direct `/arbitraje/*` returns 403

### Requirement: Assigned and History Lists
The system SHALL list `Asignadas` as disputes where `dispute.arbiter == me && status == ArbiterAssigned|EvidenceSubmitted`; `Historial` as `status == Resolved && arbiter == me`. Rows SHALL show job title, parties, monto, deadline.

#### Scenario: Assigned appears after assign
- GIVEN dispute raised on Job 9, arbiter `ARBI` assigned via `assign_arbiter` (dispute.rs:435 checks `ArbiterCannotBeParty`)
- WHEN arbiter opens Asignadas
- THEN dispute appears with status ArbiterAssigned

#### Scenario: Arbiter cannot be party blocked at assign
- GIVEN arbiter pubkey equals job.client
- WHEN `assign_arbiter` called
- THEN on-chain fails `ArbiterCannotBeParty` (dispute.rs:435) and API returns 400

### Requirement: Arbitration Balance
The system SHALL compute Saldo arbitraje as `sum(ARBITER_FEE_BPS_PER_PARTY *2 escrow bonds)` for resolved disputes where `arbiter == me`. Display SHALL show cards similar to job-history Saldo but scoped to arbitration fees.

#### Scenario: Saldo sums fees
- GIVEN arbiter resolved 2 disputes, each bond 0.025 SOL (fee_bps 250)
- WHEN arbiter opens Saldo
- THEN total shows 0.05 SOL, breakdown per dispute

### Requirement: Reject with Justification (Pending Admin)
The system SHALL allow arbiter to reject assignment via `POST /disputes/:id/reject {reason}` (reason REQUIRED, min 20 chars). Rejection SHALL set `status=PendingReassign` and require admin authority to reassign; dispute SHALL remain without arbiter until admin action.

#### Scenario: Reject happy path
- GIVEN arbiter assigned to dispute 5
- WHEN arbiter rejects with reason "Conflicto de interés: conozco al cliente"
- THEN dispute moves to PendingReassign, visible in admin Disputas queue

#### Scenario: Reject without reason blocked
- GIVEN arbiter tries reject without reason or <20 chars
- WHEN POST /disputes/5/reject
- THEN API returns 400 `ReasonRequired`

#### Scenario: Reject by non-assigned arbiter blocked
- GIVEN another arbiter not assigned
- WHEN trying to reject same dispute
- THEN API returns 403 NotAuthorized
