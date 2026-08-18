# support_tickets_metadata

**Store:** postgres  ·  **Archivo de esquema:** `support_tickets_metadata.sql`

## Propósito
Metadatos del ticket de soporte. El estado (Open/Resolved) vive on-chain.

## Campos
| Campo | Tipo | Descripción |
|---|---|---|
| `id` | `UUID PK` | Identificador |
| `ticket_pda` | `TEXT UNIQUE NOT NULL` | PDA SupportTicket on-chain |
| `job_pda` | `TEXT FK -> jobs_metadata` | Job asociado |
| `reason` | `TEXT` | Motivo (texto libre) |
| `created_at` | `TIMESTAMPTZ` | Creado |
| `resolved_at` | `TIMESTAMPTZ` | Resuelto |
| `resolution` | `TEXT` | Resolución |

## Relaciones / Referencias
- job_pda -> jobs_metadata(pda_address) ON DELETE CASCADE

## Índices
- `job_pda`

## Notas
- reason, created_at, resolved_at y resolution se movieron OFF-CHAIN desde el PDA SupportTicket.
