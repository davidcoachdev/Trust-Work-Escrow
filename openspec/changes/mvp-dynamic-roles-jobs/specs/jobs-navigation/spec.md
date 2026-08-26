# jobs-navigation Specification

## Purpose
Single Jobs hierarchy with unified menu, creation flow, per-role canvas and JobDetail. Replaces split /dashboard/client|freelancer. Canvas is read-only visual (no drag), click opens JobDetailPage {desc, chat, evidencias, estado}.

## Requirements

### Requirement: Jobs Hierarchy Navigation
The system SHALL expose a single `Jobs` top-level menu with subitems `Crear`, `Borradores`, `Publicados` (and route `/jobs/create`, `/jobs/drafts`, `/jobs/published`) under guard `has("jobs:view")`. Unauthorized users SHALL receive 403 and no menu entry.

#### Scenario: Client navigates to create job
- GIVEN authenticated user with `jobs:create` permission
- WHEN user clicks Jobs → Crear
- THEN router navigates to `/jobs/create` and renders creation form

#### Scenario: Guest sees no Jobs menu
- GIVEN unauthenticated guest
- WHEN Sidebar evaluates `MenuConfig.has("jobs:view")`
- THEN Jobs entry is hidden

#### Scenario: Direct URL without permission is blocked
- GIVEN user without `jobs:view`
- WHEN user navigates directly to `/jobs/published`
- THEN route guard returns 403

### Requirement: Canvas Columns per Job Status
The system SHALL render kanban canvas without drag, with columns derived from `Job.status` filtered by participant role: Client columns SHALL be `Borrador, Publicado, En curso, Disputado, Cancelado, Terminado`; Freelancer columns SHALL be `Solicitado, Aceptado, Rechazado, En curso, En disputas, Terminado`.

#### Scenario: Client views canvas
- GIVEN user is `client` participant on 3 jobs (Publicado, En curso, Terminado)
- WHEN user opens Publicados canvas
- THEN each job appears in its status column (1 per column, 3 total visible)

#### Scenario: Empty column shows placeholder
- GIVEN no jobs match column status
- WHEN canvas renders
- THEN column shows "Sin trabajos" placeholder, not error

### Requirement: JobDetail on Click
The system SHALL open `JobDetailPage` on canvas card click, displaying description, chat, evidencias and estado as read-only for Historial; active jobs allow actions gated by `dynamic-roles`.

#### Scenario: Click opens detail
- GIVEN canvas shows job card `pda=7a2...`
- WHEN user clicks card
- THEN router navigates to `/jobs/:pda` and loads JobDetailPage with 4 sections

#### Scenario: Deep link preserves state
- GIVEN JobDetail open on evidencia tab
- WHEN user reloads page
- THEN detail rehydrates with same pda and tab via route params
