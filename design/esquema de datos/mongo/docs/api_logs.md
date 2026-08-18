# api_logs

**Store:** mongo  ·  **Archivo de esquema:** `api_logs.json`

## Propósito
Logs de la API (auditoría de acceso).

## Campos
| Campo | Tipo | Descripción |
|---|---|---|
| `method` | `string` | HTTP method |
| `path` | `string` | Ruta |
| `auth_wallet` | `string` | Wallet autenticada (ref users) |
| `status` | `int` | Código HTTP |
| `ts` | `date` | Timestamp |

## Relaciones / Referencias
- auth_wallet -> Postgres users.wallet_principal

## Índices
- `ts`
- `auth_wallet`
