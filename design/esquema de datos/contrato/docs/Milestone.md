# Milestone

**Store:** contrato  ·  **Archivo de esquema:** `milestone.json`

## Propósito
Hitos de fondos (crítico on-chain).

## Campos
| Campo | Tipo | Descripción |
|---|---|---|
| `job` | `Pubkey` | Job |
| `amount` | `u64` | Monto |
| `status` | `MilestoneStatus (enum)` | Pending/Submitted/Approved/Rejected |
| `index` | `u8` | Índice |
| `bump` | `u8` | PDA bump |

## PDA seed
```
[b"milestone", job, index]
```

## Movido off-chain (no va al contrato)
- `title -> postgres:milestones_metadata`
- `description -> postgres:milestones_metadata`
- `deadline -> postgres:milestones_metadata`
- `submitted_at -> postgres:milestones_metadata`
- `approved_at -> postgres:milestones_metadata`
- `created_at -> postgres:milestones_metadata`
