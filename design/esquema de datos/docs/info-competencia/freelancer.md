# Freelancer.com — Análisis de publicación de trabajos

**Fuente:** https://www.freelancer.com/ (revisado en vivo)

## Qué es
Marketplace clásico de proyectos: el comprador publica un proyecto y los freelancers envían propuestas (bids).

## Cómo publican los trabajos
- Botón **"Post a Project"**.
- El comprador define el alcance y recibe **ofertas rápidas** (afirman 80% de jobs con bids en 60s).
- Sistema de **milestones** para liberar el pago por hitos.
- +4.000 categorías; también "contests" y "local jobs".

## Campos / elementos clave de una publicación
- **Título** y **descripción** del proyecto.
- **Presupuesto**: tipo fijo (fixed) u **por hora** (hourly), con monto.
- **Categoría / skills** (p. ej. Web Development, Mobile, SEO, Writing…).
- **Hitos (milestones)**: el pago se divide y se libera por entrega.
- **Ubicación / timezone** opcional (hire by location).
- Idioma del proyecto (jobs by language).

## Lo que podemos aplicar a nuestro esquema
- `budget_type` (fixed | hourly) y `budget_amount` ya encajan con `jobs_metadata`.
- **Milestones** ya los modelamos on-chain (`Milestone` PDA) y off-chain (`milestones_metadata`) — Freelancer confirma que el hito es la unidad de publicación/pago central.
- Agregar `skills/tags[]` y `language` a `jobs_metadata` para búsqueda.
