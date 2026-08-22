# reviews

**Store:** postgres  ·  **Archivo de esquema:** `reviews.sql`

## Propósito
Calificaciones entre usuarios al cerrar un job.

## Campos
| Campo | Tipo | Descripción |
|---|---|---|
| `id` | `UUID PK` | Identificador |
| `job_pda` | `TEXT FK -> jobs_metadata` | Job |
| `from_user` | `UUID FK -> users` | Autor |
| `to_user` | `UUID FK -> users` | Destinatario |
| `rating` | `INTEGER 1..5` | Puntaje |
| `comment` | `TEXT` | Comentario |
| `created_at` | `TIMESTAMPTZ` | Creado |

## Relaciones / Referencias
- job_pda -> jobs_metadata(pda_address)
- from_user -> users
- to_user -> users

## Índices
- `to_user`
- `from_user`

## Notas
- rating acotado 1..5 por CHECK.
