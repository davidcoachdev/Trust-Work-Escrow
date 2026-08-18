# user_wallets

**Store:** postgres  ·  **Archivo de esquema:** `user_wallets.sql`

## Propósito
Wallets asociadas a un usuario (hasta 10).

## Campos
| Campo | Tipo | Descripción |
|---|---|---|
| `id` | `UUID PK` | Identificador |
| `user_id` | `UUID FK -> users` | Dueño |
| `wallet_address` | `TEXT` | Dirección wallet |
| `wallet_label` | `TEXT` | Etiqueta |
| `provider` | `TEXT DEFAULT 'phantom'` | phantom/solflare/backpack/ledger |
| `is_verified` | `BOOLEAN` | Verificada |
| `is_active` | `BOOLEAN` | Activa |
| `created_at` | `TIMESTAMPTZ` | Creado |

## Relaciones / Referencias
- user_id -> users(id) ON DELETE CASCADE

## Índices
- `user_id`
- `wallet_address`

## Notas
- UNIQUE(user_id, wallet_address) evita duplicados.
