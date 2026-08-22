# Job

**Store:** contrato  ·  **Archivo de esquema:** `job.json`

## Propósito
Estado de fondos y flujo del job (crítico on-chain).

## Campos
| Campo | Tipo | Descripción |
|---|---|---|
| `client` | `Pubkey` | Cliente |
| `freelancer` | `Option<Pubkey>` | Freelancer |
| `amount` | `u64` | Monto |
| `fee_amount` | `u64` | Fee de plataforma reservada |
| `status` | `JobStatus (enum)` | Created/Funded/InProgress/Submitted/Released/Disputed/Resolved/Cancelled |
| `paused` | `bool` | Pausado |
| `paused_at` | `i64` | Inicio de pausa |
| `deadline` | `i64` | Deadline |
| `submitted_at` | `Option<i64>` | Auto-aprobación |
| `milestones_total/approved` | `u8` | Conteo hitos |
| `milestones_amount_total` | `u64` | Monto hitos |
| `applicants` | `Vec<Pubkey>` | Postulantes (para dedup) |
| `bump` | `u8` | PDA bump |

## PDA seed
```
[b"job", client, job_id]
```

## Movido off-chain (no va al contrato)
- `title -> postgres:jobs_metadata`
- `description -> postgres:jobs_metadata`
- `created_at -> postgres:jobs_metadata`
- `updated_at -> postgres:jobs_metadata`
