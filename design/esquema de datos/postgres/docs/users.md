# users

**Store:** postgres  ·  **Archivo de esquema:** `users.sql`

## Propósito
Perfil de usuario cacheado. La wallet es la fuente de verdad on-chain; el resto es metadato sincronizado.

## Campos
| Campo | Tipo | Descripción |
|---|---|---|
| `id` | `UUID PK` | Identificador interno |
| `wallet_principal` | `TEXT UNIQUE NOT NULL` | Wallet principal (on-chain) |
| `username` | `TEXT UNIQUE` | Nombre de usuario |
| `bio` | `TEXT` | Biografía |
| `avatar_url` | `TEXT` | URL de avatar |
| `reputation_score` | `DECIMAL(3,2)` | Reputación 0..5 |
| `jobs_completed` | `INTEGER` | Jobs completados |
| `disputes_won` | `INTEGER` | Disputas ganadas |
| `disputes_lost` | `INTEGER` | Disputas perdidas |
| `verified` | `BOOLEAN DEFAULT FALSE` | Verificacion de talento (estilo Toptal/PPH) |
| `hourly_rate` | `BIGINT` | Tarifa por hora (lamports/centavos) |
| `open_to_work` | `BOOLEAN DEFAULT FALSE` | Disponibilidad (estilo LinkedIn Open To Work) |
| `created_at` | `TIMESTAMPTZ` | Creado |
| `updated_at` | `TIMESTAMPTZ` | Actualizado |

## Relaciones / Referencias
- Raíz: user_wallets, teams, jobs_metadata, applications, notifications, reviews referencian a users.

## Índices
- `wallet_principal`
- `username`

## Notas
- reputation_score acotado 0..5 por CHECK.
- verified / hourly_rate / open_to_work habilitan matching y confianza (estilo Toptal/LinkedIn).
