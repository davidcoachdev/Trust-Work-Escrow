# dynamic-roles Specification

## Purpose
Per-Job authority: same email is client on own jobs and freelancer on others. On-chain `Job.client` immutable (job.rs:385 `CannotWorkOnOwnJob`); off-chain `JobParticipants` is UX authority until on-chain `freelancer` assignment.

## Requirements

### Requirement: JobParticipants Per-Job Authority
The system SHALL persist `JobParticipants {job_pda, user_email, role_per_job: client|freelancer, wallet_pubkey, joined_at}` with creator auto-inserted as `client`. Authorization for job actions SHALL check `participant.role_per_job`, not global `User.role`. Global `User.role` SHALL remain only for `admin|arbiter|guest` fallback.

#### Scenario: Same email dual role
- GIVEN alice@example.com created Job A (client) and applied to Job B
- WHEN system queries `GET /jobs?email=alice@example.com`
- THEN Job A returns with `role_per_job=client`, Job B with `freelancer`, no toggle required

#### Scenario: Role check gates acceptance
- GIVEN participant role `client` on Job 7
- WHEN user calls `accept_application` for Job 7
- THEN API allows it; if role is `freelancer`, API returns 403 NotAuthorized

### Requirement: Self-Apply Validation
The system SHALL enforce `client != applicant`, `wallet_client != wallet_freelancer`, and `arbiter != client && arbiter != freelancer` (dispute.rs:435 `ArbiterCannotBeParty`). On-chain `apply_to_job` SHALL fail with `CannotWorkOnOwnJob` (job.rs:385) when `applicant.key() == job.client`; off-chain API SHALL also return 400 before relay.

#### Scenario: Self-apply blocked
- GIVEN job.client pubkey `ABC` and applicant pubkey `ABC` (same wallet)
- WHEN applicant calls `apply_to_job` with `job_id=7`
- THEN API returns 400 `CannotWorkOnOwnJob` and on-chain would require `applicant != job.client` (job.rs:385)

#### Scenario: Same user different wallet still blocked by email
- GIVEN alice owns publish wallet `ABC` (client on Job 7) and apply wallet `XYZ`
- WHEN alice applies to Job 7 with wallet `XYZ` (different pubkey but same email participant client)
- THEN API returns 400 `CannotWorkOnOwnJob` because `participant.email` already holds `client` on that job

#### Scenario: Arbiter cannot be party
- GIVEN arbiter pubkey equals job.client or job.freelancer
- WHEN pool tries `assign_arbiter` (dispute.rs:435)
- THEN instruction requires `arbiter != job_client && arbiter != freelancer` else `ArbiterCannotBeParty`

### Requirement: Duplicate Application Guard
The system SHALL reject duplicate applies via `AlreadyApplied` and `application_index == applicants.len()` off-chain, mirroring on-chain `MAX_APPLICATIONS` and `ApplicationIndexMismatch`.

#### Scenario: Duplicate apply
- GIVEN applicant `XYZ` already in `job.applicants`
- WHEN same applicant reapplies to same job
- THEN API returns 400 `AlreadyApplied` without creating new PDA
