# chat_messages

**Store:** mongo  ·  **Archivo de esquema:** `chat_messages.json`

## Propósito
Mensajería E2EE entre cliente y freelancer de un job.

## Campos
| Campo | Tipo | Descripción |
|---|---|---|
| `job_pda` | `string` | Ref a jobs_metadata.pda_address |
| `from` | `string` | Wallet emisora |
| `to` | `string` | Wallet receptora |
| `ciphertext` | `string` | Mensaje cifrado |
| `ts` | `date` | Timestamp |

## Relaciones / Referencias
- job_pda -> Postgres jobs_metadata

## Índices
- `(job_pda, ts)`

## Notas
- NoSQL: el contenido es cifrado, no legible en claro.
