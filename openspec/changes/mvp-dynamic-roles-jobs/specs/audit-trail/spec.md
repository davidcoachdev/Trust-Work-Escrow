# audit-trail Specification

## Purpose
All tables include `created_at, updated_at, created_by, updated_by, is_active, deleted_at`; soft delete only; filtered queries; WalletHasActiveJob delete guard.

## Requirements

### Requirement: Audit Fields on All Tables
Every table (`users, user_wallets, job_participants, jobs, applications, disputes, support_tickets, evidences`) SHALL include `created_at, updated_at, created_by, updated_by, is_active: bool default true, deleted_at: Option<DateTime>`. All writes SHALL set `updated_at=now()` and `updated_by=current_user.email`. Creation SHALL set both `created_*` and `updated_*`.

#### Scenario: Create sets audit
- GIVEN alice creates job
- WHEN job row inserted
- THEN `created_at==updated_at`, `created_by==updated_by==alice@example.com`, `is_active=true`, `deleted_at=None`

#### Scenario: Update touches updated_*
- GIVEN admin patches user bob
- WHEN row updated
- THEN `updated_at` advances, `updated_by=admin@example.com`, `created_*` unchanged

### Requirement: Soft Delete Only
The system SHALL NOT hard DELETE. Delete operations SHALL set `is_active=false, deleted_at=now(), updated_by=actor`. All listing queries SHALL filter `WHERE is_active=true` by default. Hard DELETE code SHALL be absent/ unreachable.

#### Scenario: Soft delete
- GIVEN user deletes support ticket
- WHEN DELETE /support/:id
- THEN row remains with `is_active=false`, not removed from DB, query without `include_deleted` hides it

#### Scenario: Filter excludes inactive
- GIVEN 2 wallets, 1 soft-deleted
- WHEN GET /users/:email/wallets
- THEN only active wallet returned; with `?include_deleted=true` (admin only) both returned

### Requirement: Wallet Delete Guard WalletHasActiveJob
`DELETE /users/:email/wallets/:pubkey` SHALL return 400 `WalletHasActiveJob` if wallet is associated with any job where `JobStatus ∈ {InProgress, Submitted}` or `DisputeStatus ∈ {Active, EvidenceSubmitted, ArbiterAssigned}`; otherwise soft delete SHALL succeed.

#### Scenario: Delete blocked by active job
- GIVEN wallet `ABC` is client on Job 7 InProgress
- WHEN DELETE /users/alice/wallets/ABC
- THEN API returns 400 `WalletHasActiveJob` and wallet stays `is_active=true`

#### Scenario: Delete succeeds after completion
- GIVEN same wallet but Job 7 now Terminado/Released and no active dispute
- WHEN DELETE same wallet
- THEN soft delete succeeds, `is_active=false`
