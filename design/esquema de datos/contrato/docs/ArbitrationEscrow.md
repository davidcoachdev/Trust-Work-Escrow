# ArbitrationEscrow

**Store:** contrato  ·  **Archivo de esquema:** `arbitration_escrow.json`

## Propósito
Bonds de arbitraje (fondos).

## Campos
| Campo | Tipo | Descripción |
|---|---|---|
| `client_bond` | `u64` | Bond del cliente (2.5%) |
| `freelancer_bond` | `u64` | Bond del freelancer (2.5%) |
| `bump` | `u8` | PDA bump |

## PDA seed
```
[b"arb_fee", job]
```
