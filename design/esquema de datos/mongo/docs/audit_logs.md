# audit_logs

**Store:** mongo  ·  **Archivo de esquema:** `audit_logs.json`

## Propósito
Auditoría de acciones sobre PDAs on-chain.

## Campos
| Campo | Tipo | Descripción |
|---|---|---|
| `actor` | `string` | Wallet que actuó |
| `action` | `string` | Acción |
| `pda` | `string` | PDA afectado (on-chain) |
| `ts` | `date` | Timestamp |

## Relaciones / Referencias
- pda -> On-chain PDA

## Índices
- `pda`
- `ts`
