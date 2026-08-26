# support-tickets Specification

## Purpose
Client opens support ticket job-bound (`POST /jobs/:job_id/support`) or technical (`POST /support`); technical support staff sees queue in `/admin/support`. Reuses `SupportTicket {job_pda: Option, opened_by, status: Open}` off-chain; on-chain `OpenSupportTicket` requires InProgress|Submitted and blocked if Dispute Active (CaseAlreadyOpen).

## Requirements

### Requirement: Open Support Ticket (Job-Bound and Technical)
The system SHALL expose `POST /jobs/:job_id/support` for job-bound tickets and `POST /support` for technical tickets. Ticket SHALL store `{job_pda: Option<PDA>, opened_by: email, reason, status: Open, created_at}`. Job-bound SHALL require `job.status ∈ {InProgress, Submitted}`; technical SHALL allow `job_pda: None`.

#### Scenario: Job-bound ticket happy path
- GIVEN job 7 status InProgress, client alice
- WHEN alice POST /jobs/7/support {reason: "Freelancer no responde"}
- THEN ticket created with `job_pda=7` and appears in `/admin/support` queue

#### Scenario: Technical ticket without job
- GIVEN user bob with wallet issue
- WHEN bob POST /support {reason: "No veo mi wallet", job_pda: null}
- THEN ticket created with `job_pda=None`, type "Técnico"

#### Scenario: Ticket without job when required
- GIVEN job-bound flow but job_id missing or invalid PDA
- WHEN opening ticket
- THEN API returns 400 `InvalidJobPda`

### Requirement: Ticket Queue and Resolution
The system SHALL show bandeja in `/admin/support` for roles with `admin:support|support:view`. Resolve SHALL be `POST /support/:id/resolve` or `POST /support/resolve {id}` transitioning `Open → Resolved`. Only `advisor/support_staff` SHALL resolve; others 403.

#### Scenario: Support staff resolves ticket
- GIVEN ticket id 42 status Open
- WHEN advisor calls resolve
- THEN ticket status becomes Resolved, `resolved_by` and `resolved_at` set

#### Scenario: Unauthorized resolve blocked
- GIVEN freelancer without support permission
- WHEN calling resolve
- THEN API returns 403

### Requirement: On-Chain Support Constraints (where applicable)
When ticket is mirrored on-chain via `OpenSupportTicket`, the system SHALL enforce `job.status ∈ {InProgress, Submitted}` and `dispute.is_none() else CaseAlreadyOpen`; advisor cancellation refunds per on-chain logic.

#### Scenario: Ticket blocked by active dispute
- GIVEN job 7 has Active dispute
- WHEN client tries on-chain support for same job
- THEN instruction fails `CaseAlreadyOpen`

#### Scenario: Ticket blocked by wrong stage
- GIVEN job status Draft/Funded
- WHEN opening on-chain ticket
- THEN fails `CannotDisputeAtStage`-equivalent guard
