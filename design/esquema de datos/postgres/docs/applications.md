# applications

**Store:** postgres  ·  **Archivo de esquema:** `applications.sql`

## Propósito
Postulaciones a un job. La lógica (applicant/status) es on-chain.

## Campos
| Campo | Tipo | Descripción |
|---|---|---|
| `id` | `UUID PK` | Identificador |
| `application_pda` | `TEXT UNIQUE NOT NULL` | PDA Application on-chain |
| `job_pda` | `TEXT FK -> jobs_metadata` | Job |
| `applicant_id` | `UUID FK -> users` | Postulante |
| `proposal` | `TEXT` | Propuesta (texto libre) |
| `status` | `TEXT` | pending/accepted/rejected/withdrawn |
| `applied_at` | `TIMESTAMPTZ` | Creado |
| `updated_at` | `TIMESTAMPTZ` | Actualizado |

## Relaciones / Referencias
- job_pda -> jobs_metadata(pda_address)
- applicant_id -> users

## Índices
- `job_pda`
- `applicant_id`

## Notas
- proposal se movió OFF-CHAIN desde el PDA Application.
