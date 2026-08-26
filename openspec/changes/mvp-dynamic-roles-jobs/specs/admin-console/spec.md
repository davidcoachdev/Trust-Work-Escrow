# admin-console Specification

## Purpose
Dashboard admin `/admin` with 7 subroutes guarded by `admin:*|support:view|accountant`: métricas, usuarios (habilitar roles), jobs global, tickets, disputas, wallets app, contabilidad. Single source for permissions.

## Requirements

### Requirement: Admin Console Guard and Subroutes
The system SHALL expose `/admin` with subroutes `usuarios, permisos, asignaciones, wallets, metricas, tickets, disputas, contabilidad`. Guard SHALL require at least one of `admin:*`, `support:view`, `accountant`; otherwise 403. Sidebar Administración SHALL be visible only if `has("admin:*")`.

#### Scenario: Admin sees all
- GIVEN user with `roles=[admin]` and permissions `admin:users, admin:wallets, admin:support`
- WHEN opening /admin
- THEN all 7 subroutes render and API GET /admin/* returns 200

#### Scenario: Freelancer blocked
- GIVEN user with only `jobs:apply`
- WHEN navigating to /admin/users
- THEN route guard returns 403 and backend GET /admin/users returns 403

### Requirement: User Management
The system SHALL allow admin to `PATCH /admin/users/:email {roles, permissions}` to enable roles (e.g., add `arbiter`). Changes SHALL be audited with `updated_by` and SHALL reflect in next `GET /arbiter-pool` if arbiter added/removed. `is_active` toggle via soft delete.

#### Scenario: Enable arbiter role
- GIVEN admin patches bob@example.com with `roles add arbiter`
- WHEN bob next loads menu
- THEN Arbitraje menu appears via pool or role check, and GET /arbiter-pool includes bob's wallet if linked

#### Scenario: Disable user
- GIVEN admin disables carol@example.com (`is_active=false`)
- WHEN carol tries login
- THEN login returns 403 `UserDisabled`

### Requirement: Global Jobs and Tickets Views
The system SHALL provide `/admin/jobs` global (all pdas, filters estado/fecha/monto) and `/admin/support` bandeja (tickets Open→Resolved). Admin SHALL resolve tickets via `POST /support/resolve`.

#### Scenario: Admin filters global jobs
- GIVEN 50 jobs across users
- WHEN admin filters estado=Disputado in /admin/jobs
- THEN only disputed jobs appear

### Requirement: Wallets App Config and Accounting
The system SHALL expose `/admin/wallets` for fee config `fee_bps` default 250 (editable by `admin:wallets`) and treasury view; `/admin/accounting` for `admin:accountant` shows sum escrow, fees collected, pending releases. Treasury editable SHALL be flagged as Open Question pending decision.

#### Scenario: Update fee_bps
- GIVEN admin with `admin:wallets`
- WHEN PATCH /admin/config {fee_bps: 300}
- THEN new jobs use 300 bps, existing jobs retain original fee_amount

### Requirement: Disputes Admin View
The system SHALL allow admin to view all disputes, reassign after arbiter rejection (PendingReassign), and audit resolution.

#### Scenario: Reassign after rejection
- GIVEN dispute in PendingReassign after arbiter rejected
- WHEN admin POST /disputes/:id/assign {new_arbiter}
- THEN dispute returns to ArbiterAssigned with new arbiter, checked against `ArbiterCannotBeParty` (dispute.rs:435)
