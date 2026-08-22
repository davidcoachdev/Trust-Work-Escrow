# Dispute

**Store:** contrato  ·  **Archivo de esquema:** `dispute.json`

## Propósito
Resolución de disputa (crítico on-chain).

## Campos
| Campo | Tipo | Descripción |
|---|---|---|
| `job` | `Pubkey` | Job |
| `raised_by` | `Pubkey` | Quien inicia |
| `arbiter` | `Option<Pubkey>` | Árbitro asignado |
| `status` | `DisputeStatus (enum)` | Open/Active/EvidenceSubmitted/ArbiterAssigned/Resolved/Expired |
| `evidence_count` | `u8` | Conteo evidencias |
| `evidence_cleanup_cursor` | `u8` | Cursor de limpieza |
| `deadline` | `i64` | Deadline para aceptar |
| `client_payout_percent` | `u8` | % cliente |
| `freelancer_payout_percent` | `u8` | % freelancer |
| `bump` | `u8` | PDA bump |

## PDA seed
```
[b"dispute", job]
```

## Movido off-chain (no va al contrato)
- `reason -> postgres:disputes_metadata`
- `created_at -> postgres:disputes_metadata`
- `resolved_at -> postgres:disputes_metadata`
- `resolution -> postgres:disputes_metadata`
