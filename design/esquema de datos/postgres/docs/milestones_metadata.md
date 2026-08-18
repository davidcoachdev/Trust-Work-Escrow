# milestones_metadata

**Store:** postgres  ·  **Archivo de esquema:** `milestones_metadata.sql`

## Propósito
Metadatos de cada hito. amount/status/deadlines son on-chain.

## Campos
| Campo | Tipo | Descripción |
|---|---|---|
| `id` | `UUID PK` | Identificador |
| `job_pda` | `TEXT FK -> jobs_metadata.pda_address` | Job padre |
| `index` | `INTEGER` | Índice del hito en el PDA |
| `title` | `TEXT NOT NULL` | Título |
| `description` | `TEXT` | Descripción |
| `created_at` | `TIMESTAMPTZ` | Creado |

## Relaciones / Referencias
- job_pda -> jobs_metadata(pda_address) ON DELETE CASCADE

## Índices
- `job_pda`

## Notas
- title/description se movieron OFF-CHAIN desde el PDA Milestone.
- UNIQUE(job_pda, index).
