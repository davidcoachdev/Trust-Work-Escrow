# team_members

**Store:** postgres  ·  **Archivo de esquema:** `team_members.sql`

## Propósito
Miembros de un equipo.

## Campos
| Campo | Tipo | Descripción |
|---|---|---|
| `team_id` | `UUID FK -> teams` | Equipo (PK compuesta) |
| `user_id` | `UUID FK -> users` | Usuario (PK compuesta) |
| `role` | `TEXT` | owner/lead/pm/developer/designer/qa/member |
| `department` | `TEXT` | frontend/backend/design/qa/management |
| `payout_percentage` | `INTEGER 0..100` | % de pago |
| `is_active` | `BOOLEAN` | Activo |
| `joined_at` | `TIMESTAMPTZ` | Ingreso |
| `left_at` | `TIMESTAMPTZ` | Salida (NULL si activo) |

## Relaciones / Referencias
- team_id -> teams(id) ON DELETE CASCADE
- user_id -> users(id) ON DELETE CASCADE

## Índices
- `user_id`

## Notas
- PK compuesta (team_id, user_id).
