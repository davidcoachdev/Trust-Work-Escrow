# SupportTicket

**Store:** contrato  ·  **Archivo de esquema:** `support_ticket.json`

## Propósito
Ticket de soporte (estado on-chain).

## Campos
| Campo | Tipo | Descripción |
|---|---|---|
| `job` | `Pubkey` | Job |
| `opened_by` | `Pubkey` | Quien abre |
| `status` | `SupportTicketStatus (enum)` | Open/Resolved |
| `bump` | `u8` | PDA bump |

## PDA seed
```
[b"support_ticket", job]
```

## Movido off-chain (no va al contrato)
- `reason -> postgres:support_tickets_metadata`
- `created_at -> postgres:support_tickets_metadata`
- `resolved_at -> postgres:support_tickets_metadata`
- `resolution -> postgres:support_tickets_metadata`
