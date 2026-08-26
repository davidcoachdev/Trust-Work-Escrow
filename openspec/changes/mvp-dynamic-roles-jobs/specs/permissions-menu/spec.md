# permissions-menu Specification

## Purpose
Single dynamic menu: `Sidebar {roles: Vec<Role>, permissions: Vec<String>}` + `MenuConfig {has(p) -> bool}` with route guards. Freelancer canvas and disputes are visual-only for historial.

## Requirements

### Requirement: Dynamic MenuConfig and Sidebar
The system SHALL derive `MenuConfig` from `UserMetadata {roles: Vec<String>, permissions: Vec<String>}` (replacing single `role`). `Sidebar` SHALL accept `Vec<Role>` not `DashboardRole` single. `has(p)` SHALL match exact or wildcard `admin:*`. Menu items SHALL render iff required permission present; combined roles SHALL merge submenus without toggle.

#### Scenario: Combined roles merge menu
- GIVEN user `roles=[client, freelancer, admin]` with permissions `jobs:view, jobs:create, disputes:view, admin:users`
- WHEN Sidebar renders
- THEN Jobs (Crear/Borradores/Publicados+Canvas), Disputas (Métricas+canvas), Administración (Usuarios/...) all visible together, no role toggle

#### Scenario: Permissions derive menu
- GIVEN permissions `[jobs:view:own, jobs:apply, config:wallet]`
- WHEN MenuConfig.has("jobs:view")
- THEN false (requires view all), but `has("jobs:view:own")` true → Jobs shows own-only canvas

#### Scenario: Role array not single
- GIVEN legacy `role="client"` still stored
- WHEN backend returns `roles=["client"]` alias
- THEN frontend treats as Vec and renders correctly (backward compat flag)

### Requirement: Route Guards by Permission
Every dashboard route SHALL have guard checking `has(required_permission)`. Unauthorized direct navigation SHALL return 403, not redirect loop. Guards SHALL be defined in `app/src/route.rs`.

#### Scenario: Guard blocks admin route
- GIVEN user without `admin:users`
- WHEN navigating to `/admin/users`
- THEN guard returns 403 page, no API call attempted

#### Scenario: Guard allows permitted route
- GIVEN user with `arbitration:assigned`
- WHEN navigating to `/arbitraje/asignadas`
- THEN guard passes and page loads

### Requirement: Freelancer Canvas and Disputes Visual-Only Constraint
Freelancer canvas and disputas views for historial SHALL be visual-only (no drag, no status mutation via canvas). Status changes SHALL only occur via explicit JobDetail actions (aceptar, disputar, resolver) gated by `dynamic-roles`.

#### Scenario: Historial canvas click is read-only
- GIVEN freelancer viewing Historial canvas, job status Terminado
- WHEN clicking card
- THEN JobDetail opens read-only; no drag handle, no inline status dropdown

#### Scenario: Active canvas actions gated
- GIVEN freelancer viewing active Publicado job where they are applicant pending
- WHEN opening detail
- THEN actions like "Aceptar solicitud" are hidden (client-only), per `role_per_job` check

### Requirement: Permission Drift Prevention
Permission strings SHALL be defined in single `backend/api/src/metadata.rs` allowlist; frontend SHALL import same constants via `get_config` or shared DTO. Drift SHALL be detected by contract test asserting `frontend permissions ⊆ backend allowlist`.

#### Scenario: Drift detection
- GIVEN frontend adds `jobs:delete:own` not in backend allowlist
- WHEN contract test runs
- THEN test fails indicating drift

### Requirement: Backward Compatibility Flag
Flag `permissions-menu` off SHALL revert to `DashboardRole` single and `roles->[0]` alias; audit fields nullable for rollback.

#### Scenario: Flag off fallback
- GIVEN flag disabled
- WHEN user logs in
- THEN Sidebar receives `DashboardRole` single derived from `roles[0]`, menu degrades to legacy client/freelancer split but no error
