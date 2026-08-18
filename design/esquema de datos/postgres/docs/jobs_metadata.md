# jobs_metadata

**Store:** postgres  ·  **Archivo de esquema:** `jobs_metadata.sql`

## Proposito
Metadatos descriptivos del Job. El estado/fondos viven on-chain; aca el texto y la clasificacion.

## Campos
| Campo | Tipo | Descripcion |
|---|---|---|
| `id` | `UUID PK` | Identificador |
| `pda_address` | `TEXT UNIQUE NOT NULL` | PDA Job on-chain (link) |
| `client_id` | `UUID FK -> users` | Cliente |
| `freelancer_id` | `UUID FK -> users NULL` | Freelancer asignado |
| `team_id` | `UUID FK -> teams NULL` | Equipo (opcional) |
| `room_id` | `UUID FK -> rooms NULL` | Sala de distribucion/filtrado |
| `category_id` | `UUID FK -> categories NULL` | Taxonomia |
| `title` | `TEXT NOT NULL` | Titulo del aviso |
| `description` | `TEXT` | Descripcion del aviso |
| `skills` | `TEXT[]` | Tags granulares para matching |
| `budget_type` | `TEXT (fixed|hourly|package)` | Tipo de presupuesto |
| `employment_type` | `TEXT (full_time|part_time|contract|internship)` | Tipo de empleo |
| `engagement_type` | `TEXT (project|full_time|hourly)` | Modalidad |
| `publication_type` | `TEXT (bidding|fixed_offer)` | Subasta vs oferta fija |
| `language` | `TEXT` | Idioma |
| `location` | `TEXT` | Ubicacion |
| `is_remote` | `BOOLEAN DEFAULT FALSE` | Remoto |
| `created_at` | `TIMESTAMPTZ` | Creado |
| `updated_at` | `TIMESTAMPTZ` | Actualizado |

## Relaciones / Referencias
- client_id -> users
- freelancer_id -> users
- team_id -> teams
- room_id -> rooms
- category_id -> categories
- pda_address es la clave hacia el contrato

## Indices
- `client_id`
- `freelancer_id`
- `team_id`
- `room_id`
- `category_id`

## Notas
- title/description se movieron OFF-CHAIN desde el PDA Job.
- Las aplicaciones[] del PDA viven en Postgres; solo queda applications_count on-chain.
- room_id + category_id + skills[] + budget_type habilitan el filtrado/distribucion.
