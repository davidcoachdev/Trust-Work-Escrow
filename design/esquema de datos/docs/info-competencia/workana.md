# Workana — Análisis de publicación de trabajos

**Fuente:** https://www.workana.com/ (revisado en vivo)

## Qué es
Marketplace enfocado en **Latinoamérica** (talento que habla tu idioma y trabaja en tu zona horaria). Pagos protegidos y garantía de satisfacción.

## Cómo publican los trabajos
Dos modelos de contratación claramente separados:
1. **Hire Developers** — desarrolladores pre-seleccionados y certificados, por un período definido, cobrando en USD.
2. **Hire Freelancers** — por proyecto (goal-based) u horas, recibiendo múltiples propuestas y acordando un precio.

## Campos / elementos clave de una publicación
- **Categorías**: IT & Programming, Design & Multimedia, Writing & Translation, Sales & Marketing, Admin Support, Legal, Finance & Management, Engineering & Manufacturing.
- **Título** y **descripción**.
- **Tipo de engagement**: full-time (devs) vs proyecto puntual (freelancers).
- **Presupuesto**: por objetivo (goal-based) u **por hora**.
- **Skills** derivados de la categoría (Web development, WordPress, E-commerce, Mobile, Logo design, SEO…).
- Propuestas de freelancers con precio acordado.

## Lo que podemos aplicar a nuestro esquema
- Distinguir `engagement_type` (full-time | project | hourly) como campo de la publicación.
- `category` + `skills[]` para filtros regionales (LatAm) — útil si apuntamos a ese mercado.
- El modelo "devs certificados por período" sugiere una entidad `teams`/`contractors` ya contemplada en nuestro Postgres.
