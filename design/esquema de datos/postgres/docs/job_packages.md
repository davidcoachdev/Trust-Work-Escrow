# job_packages

**Store:** postgres  ·  **Archivo de esquema:** `job_packages.sql`

## Proposito
Paquetes escalonados del aviso (estilo Fiverr/PeoplePerHour Hourlies). El monto total escrowado vive on-chain; los paquetes son off-chain.

## Campos
| Campo | Tipo | Descripcion |
|---|---|---|
| `id` | `UUID PK` | Identificador |
| `job_pda` | `TEXT FK -> jobs_metadata NOT NULL` | Aviso |
| `tier` | `TEXT (basic|standard|premium)` | Nivel |
| `price` | `BIGINT NOT NULL` | Precio del paquete |
| `delivery_time_days` | `INTEGER` | Tiempo de entrega |
| `revisions` | `INTEGER DEFAULT 0` | Revisiones incluidas |
| `description` | `TEXT` | Descripcion |

## Relaciones
- job_pda -> jobs_metadata

## Indices
- job_pda
