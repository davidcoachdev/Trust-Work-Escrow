# Config

**Store:** contrato  ·  **Archivo de esquema:** `config.json`

## Propósito
Configuración crítica del protocolo (todo on-chain).

## Campos
| Campo | Tipo | Descripción |
|---|---|---|
| `authority` | `Pubkey` | Quien pausa/actualiza treasury |
| `advisor` | `Pubkey` | Asesor de plataforma (resuelve PlatformCase) |
| `treasury` | `Pubkey` | Wallet que recibe fees |
| `arbitration_treasury` | `Pubkey` | Destino fee/shortfall arbitraje |
| `fee_bps` | `u16` | Fee de plataforma en basis points |
| `paused` | `bool` | Estado de pausa |
| `bump` | `u8` | PDA bump |

## PDA seed
```
[b"config"]
```
