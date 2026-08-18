# teams

**Store:** postgres  ·  **Archivo de esquema:** `teams.sql`

## Propósito
Equipos/agencias. Referencia opcional a un PDA on-chain.

## Campos
| Campo | Tipo | Descripción |
|---|---|---|
| `id` | `UUID PK` | Identificador |
| `pda_address` | `TEXT UNIQUE` | PDA on-chain (opcional) |
| `owner_id` | `UUID FK -> users` | Dueño |
| `name` | `TEXT NOT NULL` | Nombre |
| `slug` | `TEXT UNIQUE` | Slug para URLs publicas |
| `description` | `TEXT` | Descripción |
| `avatar_url` | `TEXT` | Avatar |
| `total_earnings` | `BIGINT` | Ganancias totales |
| `jobs_completed` | `INTEGER` | Jobs completados |
| `created_at` | `TIMESTAMPTZ` | Creado |
| `updated_at` | `TIMESTAMPTZ` | Actualizado |

## Relaciones / Referencias
- owner_id -> users(id) ON DELETE CASCADE

## Índices
- `owner_id`
