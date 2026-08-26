# disputes-scoped Specification

## Purpose
Disputas scoped to participation: Abiertas / Historial (+ métricas). Funded jobs in `InProgress|Submitted` may raise dispute (dispute.rs:263 requires `CaseAlreadyOpen` guard); arbiter assignment respects `ArbiterCannotBeParty` (dispute.rs:435).

## Requirements

### Requirement: Disputes Scoped to Participation
The system SHALL scope `GET /disputes?email=&scope=open|history` to jobs where `participant.email == current` and dispute exists. `scope=open` SHALL return status `Active|EvidenceSubmitted|ArbiterAssigned`; `history` SHALL return `Resolved|Rejected`.

#### Scenario: Freelancer sees own disputes
- GIVEN freelancer participant on Job 9 with dispute Active
- WHEN opening Disputas → Abiertas
- THEN Job 9 appears; another user's dispute does not

#### Scenario: No disputes empty state
- GIVEN user has no disputes
- WHEN opening Abiertas
- THEN UI shows "Sin disputas abiertas"

### Requirement: Open vs History Partition
The system SHALL partition: Abiertas lists active disputes with metrics (días abierta, monto en escrow, bond); Historial lists resolved with outcome and `resolved_at`. Metrics SHALL aggregate count open, total escrow locked.

#### Scenario: Metrics header
- GIVEN 3 open disputes totaling 4.5 SOL escrow
- WHEN viewing Abiertas
- THEN header shows "3 abiertas · 4.5 SOL en custodia"

### Requirement: Dispute Creation Guard
The system SHALL allow `raise_dispute` only when `job.status ∈ {InProgress, Submitted}` and `ticket.is_none()` (dispute.rs:263 `CaseAlreadyOpen`). Raiser SHALL be `job.client` or `job.freelancer` (`NotAuthorized` otherwise). Wallet `ArbiterCannotBeParty` (dispute.rs:435) SHALL be enforced on arbiter assign, not on raise.

#### Scenario: Raise dispute happy path
- GIVEN job status Submitted, raiser is client, no existing ticket/dispute
- WHEN raiser calls `raise_dispute`
- THEN dispute created Active, bond `ARBITER_FEE_BPS_PER_PARTY` transferred to escrow

#### Scenario: Raise dispute blocked by wrong stage
- GIVEN job status Funded (no work submitted)
- WHEN user tries raise_dispute
- THEN API returns 400 `CannotDisputeAtStage` (dispute.rs:263 requires InProgress|Submitted)

#### Scenario: Duplicate dispute blocked
- GIVEN job already has dispute/ticket (`ticket.is_some()`)
- WHEN raiser retries
- THEN API returns 400 `CaseAlreadyOpen`

### Requirement: Dispute Canvas Columns
The system SHALL render dispute canvas columns `Solicitada, Rechazada, En curso, Resuelta`; click opens JobDetail read-only + dispute thread.

#### Scenario: Column mapping
- GIVEN disputes with 1 Active, 1 Resolved
- WHEN viewing canvas
- THEN Active appears in "En curso", Resolved in "Resuelta"
