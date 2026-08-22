# events

**Store:** mongo  ·  **Archivo de esquema:** `events.json`

## Propósito
Eventos/streams del indexer (sync on-chain -> off-chain).

## Campos
| Campo | Tipo | Descripción |
|---|---|---|
| `type` | `string` | Tipo de evento |
| `payload` | `object` | Datos del evento |
| `ts` | `date` | Timestamp |

## Relaciones / Referencias
- payload referencia entidades de los 3 stores

## Índices
- `(type, ts)`
