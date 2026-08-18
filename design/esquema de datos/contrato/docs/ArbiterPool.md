# ArbiterPool

**Store:** contrato  ·  **Archivo de esquema:** `arbiter_pool.json`

## Propósito
Registro de árbitros neutrales.

## Campos
| Campo | Tipo | Descripción |
|---|---|---|
| `authority` | `Pubkey` | Plataforma |
| `arbiters` | `Vec<Pubkey> (max 50)` | Pool de árbitros |
| `bump` | `u8` | PDA bump |

## PDA seed
```
[b"arbiter_pool"]
```
