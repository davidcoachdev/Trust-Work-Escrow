# PeoplePerHour — Análisis de publicación de trabajos

**Fuente:** https://www.peopleperhour.com/ (revisado en vivo)

## Qué es
Marketplace UK-first, alcance global (100+ países). Talento revisado a mano. Tiene asistente de IA ("Phoenix") para matching.

## Cómo publican los trabajos
Modelo **híbrido**:
1. **Post Project** — el comprador publica y los freelancers envían propuestas.
2. **Offers / "Hourlies"** — servicios listos, de precio fijo, comprables directo (similar a los gigs de Fiverr).

## Campos / elementos clave de una publicación
- **Categorías**: AI Services, Technology & Programming, Writing & Translation, Design, Digital Marketing, Video/Photo, Business, Music & Audio, Marketing/Branding/Sales, Social Media.
- Para proyectos: **título, descripción, presupuesto, skills**.
- Para Offers: **precio fijo**, **tiempo de entrega** (p. ej. "$15 · 1 day"), paquete de entregables.
- **Tasa por hora** de cada freelancer ($11/hr … $67/hr) y **n° de proyectos/reviews**.

## Lo que podemos aplicar a nuestro esquema
- El modelo híbrido (proyecto abierto + oferta fija) sugiere soportar **dos tipos de publicación** en `jobs_metadata`: `bidding` y `fixed_offer`.
- `hourly_rate` como atributo del freelancer (tabla `users`/`user_wallets` o perfil) y `fixed_price` + `delivery_time` en la oferta.
- `reviews_count` + `rating` ya los tenemos en `users`/`reviews`.
