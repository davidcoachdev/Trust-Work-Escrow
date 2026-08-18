# dispute_evidence

**Store:** mongo  ·  **Archivo de esquema:** `dispute_evidence.json`

## Propósito
Contenido de evidencias de disputa.

## Campos
| Campo | Tipo | Descripción |
|---|---|---|
| `dispute_pda` | `string` | Ref a disputes_metadata.dispute_pda |
| `index` | `int` | Índice de evidencia |
| `author` | `string` | Wallet autora |
| `content` | `string` | Contenido (texto/doc) |
| `submitted_at` | `date` | Timestamp |

## Relaciones / Referencias
- dispute_pda -> Postgres disputes_metadata

## Índices
- `(dispute_pda, index)`

## Notas
- El CONTENIDO vive acá; en el PDA Evidence on-chain solo queda content_hash para integridad.
