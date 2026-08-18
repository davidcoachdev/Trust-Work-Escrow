# disputes_metadata

**Store:** postgres  ·  **Archivo de esquema:** `disputes_metadata.sql`

## Propósito
Metadatos de la disputa. La lógica/resolución es on-chain.

## Campos
| Campo | Tipo | Descripción |
|---|---|---|
| `id` | `UUID PK` | Identificador |
| `dispute_pda` | `TEXT UNIQUE NOT NULL` | PDA Dispute on-chain |
| `job_pda` | `TEXT FK -> jobs_metadata` | Job asociado |
| `reason` | `TEXT` | Motivo (texto libre) |
| `resolved_at` | `TIMESTAMPTZ` | Resuelto (espejo on-chain) |
| `resolution` | `TEXT` | Resolucion (espejo on-chain) |
| `created_at` | `TIMESTAMPTZ` | Creado |

## Relaciones / Referencias
- job_pda -> jobs_metadata(pda_address) ON DELETE CASCADE

## Índices
- `job_pda`

## Notas
- reason se movió OFF-CHAIN desde el PDA Dispute.
