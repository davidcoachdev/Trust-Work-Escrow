# payments

**Store:** postgres  ·  **Archivo de esquema:** `payments.sql`

## Propósito
Espejo de transacciones on-chain.

## Campos
| Campo | Tipo | Descripción |
|---|---|---|
| `id` | `UUID PK` | Identificador |
| `signature` | `TEXT UNIQUE NOT NULL` | Signature de la tx (on-chain) |
| `job_pda` | `TEXT FK -> jobs_metadata` | Job |
| `payer` | `TEXT` | Pagador |
| `payee` | `TEXT` | Receptor |
| `amount` | `BIGINT` | Monto (lamports) |
| `type` | `TEXT` | deposit/release/refund/fee/arbitration |
| `created_at` | `TIMESTAMPTZ` | Creado |

## Relaciones / Referencias
- job_pda -> jobs_metadata(pda_address)

## Índices
- `job_pda`

## Notas
- signature es la fuente de verdad.
