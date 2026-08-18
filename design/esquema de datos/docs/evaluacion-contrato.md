# Evaluacion del contrato (trust-escrow-v3) para las nuevas features de distribucion

**Alcance:** Ver si las features propuestas (rooms/salas, categories, job_packages, job_interests/senales, budget_type, skills, etc.) exigen cambios en el programa on-chain (Anchor, `trust-escrow-v3/programs/trust-escrow-v3/src/lib.rs`).

## Conclusion rapida
**No se requieren cambios estructurales en el contrato** para ninguna de las nuevas features. Todas son metadatos descriptivos/clasificatorios y viven off-chain (Postgres / backend), respetando el split definido.

## Detalle por feature
| Feature | On-chain? | Por que |
|---|---|---|
| `rooms` (salas) | No | Creadas automaticamente por el backend; solo es un id off-chain en `jobs_metadata.room_id`. |
| `categories` | No | Taxonomia admin; solo `category_id` off-chain en `jobs_metadata`. |
| `job_packages` (paquetes) | No | El monto total ya esta en `Job.amount`; los tiers son off-chain. |
| `job_interests` (senales) | No | La postulacion real ya es el PDA `Application` (dedup + `applicants[]` on-chain). La senal es liviana/off-chain. |
| `budget_type` / `employment_type` / `engagement_type` / `publication_type` | No | Descriptivos; no afectan la logica de escrow. |
| `skills` / `language` / `location` / `is_remote` | No | Descriptivos para matching/filtrado. |

## Hallazgo importante (desalineacion con el diseno)
El PDA `Job` **ya guarda `title` (max 100) y `description` (max 500) on-chain** (lib.rs lineas 410-413, y `create_job` los recibe como args). Nuestro `reparto-datos.md` y `jobs_metadata.md` dicen que "title/description se movieron OFF-CHAIN". En la realidad hay **duplicacion**: existen en el PDA `Job` Y en `jobs_metadata`.

Recomendacion (no bloquea las nuevas features):
- **Corto plazo:** mantener ambos. El on-chain `title`/`description` actua como preview corto; `jobs_metadata` guarda el contenido completo/rico. Corregir `reparto-datos.md` para reflejar que el PDA todavia los tiene.
- **Mediano plazo (opcional):** si se quiere quitar del contrato, implica cambiar la firma de `create_job`, el tamaño del PDA y un redeploy + migracion de cuentas. Fuera de scope ahora.

## Cambios futuros que SI tocarian el contrato (fuera de scope)
- **Hourly / streaming escrow:** si `budget_type = hourly` debe liberar fondos por tiempo, el contrato necesitaria soporte de release por tiempo (Clock-based). Hoy el on-chain solo maneja `amount` total. Decision: `budget_type` queda descriptivo; el `amount` on-chain sigue siendo el total escrowado.
- Cualquier campo que afecte fondos/trust (nuevos porcentajes, nuevos estados) si se quisiera on-chain.

## Acciones sugeridas
1. Dejar el contrato v3 intacto para este round de distribucion.
2. Backend: en `create_job`, despues de crear el PDA, crear `jobs_metadata` + auto-crear la `room` y linkear `category_id`/`room_id`.
3. Corregir `reparto-datos.md` (nota de title/description on-chain).
4. Seed de `categories` desde `categories_seed.sql` (admin las gestiona; back semilla las mas usadas).
