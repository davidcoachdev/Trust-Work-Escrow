# job-history Specification

## Purpose
Historial aggregates ALL jobs where user is participant (publicados como client + realizados como freelancer) with unified filters and read-only visual detail.

## Requirements

### Requirement: Historial Aggregation
The system SHALL aggregate Historial as all jobs where `job_participants.email == current_user` across any `JobStatus`, partitioned by `role_per_job`. Query `GET /jobs/history?email=...` SHALL return both published and realized without separate endpoints.

#### Scenario: Combined historial
- GIVEN alice is client on 2 jobs and freelancer on 3 jobs (various status Terminado/Cancelado)
- WHEN alice opens Historial
- THEN list shows 5 entries ordered by `updated_at desc`, each labeled `Publicado` or `Realizado` via `role_per_job`

#### Scenario: No historial empty state
- GIVEN new user with zero participations
- WHEN opening Historial
- THEN UI shows "Sin historial aún" with CTA to Crear/Explorar

### Requirement: Filters (Estado, Rol, Fecha, Título, Monto, Con/Sin Disputa)
The system SHALL provide filters: `estado` (JobStatus), `rol-per-job` (client|freelancer), `fecha` (range), `título` (substring), `monto` (range), `con/sin disputa` (join disputes where dispute exists). All filters SHALL compose via AND.

#### Scenario: Filter by estado + disputa
- GIVEN 10 historial jobs, 2 with dispute Active
- WHEN user filters `estado=Terminado` AND `con disputa=true`
- THEN result is only Terminado jobs that have a dispute record

#### Scenario: Titulo substring case-insensitive
- GIVEN jobs titled "Landing para Cafetería" and "Landing API"
- WHEN user filters `titulo=landing`
- THEN both match, case-insensitive

### Requirement: Read-Only Detail for Historial
The system SHALL render JobDetail for historial jobs as read-only: description, chat transcript, evidencias and status are visible but actions (aceptar, disputar, liberar) are disabled. Chat/evidencias SHALL be scoped to participants only.

#### Scenario: Read-only enforcement
- GIVEN historial job with status Terminado
- WHEN freelancer opens JobDetail
- THEN chat is visible read-only, "Enviar mensaje" disabled, evidencia upload hidden

#### Scenario: Disputa badge in historial
- GIVEN historial job had dispute Resuelta
- WHEN viewing list
- THEN row shows badge "Disputa resuelta" and detail reveals dispute timeline read-only
