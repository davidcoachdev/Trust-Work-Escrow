# Evidence

**Store:** contrato  ·  **Archivo de esquema:** `evidence.json`

## Propósito
Evidencia (integridad on-chain).

## Campos
| Campo | Tipo | Descripción |
|---|---|---|
| `dispute` | `Pubkey` | Dispute |
| `index` | `u8` | Índice |
| `author` | `Pubkey` | Autor |
| `content_hash` | `[u8; 32]` | Hash del contenido (integridad) |
| `bump` | `u8` | PDA bump |

## PDA seed
```
[b"evidence", dispute, index]
```

## Movido off-chain (no va al contrato)
- `content -> mongo:dispute_evidence (solo el hash queda on-chain)`
- `submitted_at -> mongo:dispute_evidence`
