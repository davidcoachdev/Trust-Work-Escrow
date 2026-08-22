# categories

**Store:** postgres  ·  **Archivo de esquema:** `categories.sql`

## Proposito
Taxonomia jerarquica para clasificar publicaciones (estilo LinkedIn/Freelancer).

**Regla de negocio:** las categorias las crea el **administrador desde su panel de administracion**. El backend hace seed de las mas usadas al arrancar (ver `esquemas/categories_seed.sql`).

## Campos
| Campo | Tipo | Descripcion |
|---|---|---|
| `id` | `UUID PK` | Identificador |
| `parent_id` | `UUID FK -> categories NULL` | Subcategoria (auto-referencia) |
| `name` | `TEXT NOT NULL` | Nombre |
| `slug` | `TEXT UNIQUE NOT NULL` | Slug para URLs/rutas |
| `descripcion` | `TEXT` | Descripcion de la categoria |

## Relaciones
- parent_id -> categories (auto)
- rooms.category_id -> categories
- jobs_metadata.category_id -> categories

## Indices
- parent_id
