# job_interests

**Store:** postgres  ·  **Archivo de esquema:** `job_interests.sql`

## Proposito
Senales de interes livianas (estilo Torre signal / LinkedIn Open To Work). Mas liviano que el PDA Application on-chain, que es la postulacion real para dedup/count.

## Campos
| Campo | Tipo | Descripcion |
|---|---|---|
| `id` | `UUID PK` | Identificador |
| `job_pda` | `TEXT FK -> jobs_metadata NOT NULL` | Aviso |
| `user_id` | `UUID FK -> users NOT NULL` | Usuario interesado |
| `created_at` | `TIMESTAMPTZ` | Creado |

## Relaciones
- job_pda -> jobs_metadata
- user_id -> users

## Indices
- job_pda
- user_id
