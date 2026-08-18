# rooms

**Store:** postgres  ·  **Archivo de esquema:** `rooms.sql`

## Proposito
Salas de distribucion/filtrado (tablon/canal) donde se publican los jobs. Agrupan avisos por categoria para un filtrado mas fino.

**Regla de negocio:** las salas son **creadas automaticamente por el backend** al publicar un job (ej. una sala por categoria o tema), no por el usuario. Solo se guarda `room_id` off-chain en `jobs_metadata`.

## Campos
| Campo | Tipo | Descripcion |
|---|---|---|
| `id` | `UUID PK` | Identificador |
| `category_id` | `UUID FK -> categories NOT NULL` | Categoria de la sala |
| `name` | `TEXT NOT NULL` | Nombre de la sala |
| `slug` | `TEXT UNIQUE NOT NULL` | Slug |
| `description` | `TEXT` | Descripcion |
| `created_by` | `UUID FK -> users NULL` | Creador |
| `is_public` | `BOOLEAN DEFAULT TRUE` | Visibilidad |
| `created_at` | `TIMESTAMPTZ` | Creado |

## Relaciones
- category_id -> categories
- created_by -> users
- jobs_metadata.room_id -> rooms

## Indices
- category_id
- created_by
