# Application

**Store:** contrato  ·  **Archivo de esquema:** `application.json`

## Propósito
Postulación (lógica on-chain).

## Campos
| Campo | Tipo | Descripción |
|---|---|---|
| `job` | `Pubkey` | Job |
| `index` | `u8` | Índice |
| `applicant` | `Pubkey` | Postulante |
| `proposal_hash` | `[u8; 32]` | Hash de la propuesta |
| `status` | `ApplicationStatus (enum)` | Pending/Accepted/Rejected/Withdrawn |
| `bump` | `u8` | PDA bump |

## PDA seed
```
[b"application", job, index, applicant]
```

## Movido off-chain (no va al contrato)
- `proposal -> postgres:applications`
- `applied_at -> postgres:applications`
