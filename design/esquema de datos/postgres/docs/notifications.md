# notifications

**Store:** postgres  ·  **Archivo de esquema:** `notifications.sql`

## Propósito
Avisos por usuario.

## Campos
| Campo | Tipo | Descripción |
|---|---|---|
| `id` | `UUID PK` | Identificador |
| `user_id` | `UUID FK -> users` | Destinatario |
| `type` | `TEXT` | Tipo de aviso |
| `payload` | `JSONB` | Datos del aviso |
| `read` | `BOOLEAN DEFAULT false` | Leído |
| `created_at` | `TIMESTAMPTZ` | Creado |

## Relaciones / Referencias
- user_id -> users(id) ON DELETE CASCADE

## Índices
- `(user_id, read)`
