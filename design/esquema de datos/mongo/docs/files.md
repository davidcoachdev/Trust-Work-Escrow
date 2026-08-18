# files

**Store:** mongo  ·  **Archivo de esquema:** `files.json`

## Propósito
Referencias a adjuntos (el binario vive en object storage).

## Campos
| Campo | Tipo | Descripción |
|---|---|---|
| `ref` | `string` | Referencia |
| `owner_pda` | `string` | Dueño (on-chain/user) |
| `url` | `string` | URL del binario |
| `mime` | `string` | Tipo MIME |
| `ts` | `date` | Timestamp |

## Relaciones / Referencias
- owner_pda -> On-chain / Postgres users

## Índices
- `owner_pda`
