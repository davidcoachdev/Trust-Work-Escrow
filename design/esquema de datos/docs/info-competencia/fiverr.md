# Fiverr — Análisis de publicación de trabajos

**Fuente:** https://www.fiverr.com/ (revisado en vivo)

## Qué es
Marketplace de "gigs" (servicios predefinidos) publicados por los vendedores. El comprador no abre un concurso: elige un gig ya publicado y compra un paquete.

## Cómo publican los trabajos
- El **vendedor** crea el gig, no el comprador.
- Catálogo por **categorías**: Graphics & Design, Programming & Tech, Digital Marketing, Video & Animation, Writing & Translation, Music & Audio, Business, Finance, AI Services, Personal Growth, Consulting, Data, Photography.
- Cada gig se estructura en **3 paquetes escalonados**: Basic / Standard / Premium.

## Campos / elementos clave de una publicación (gig)
- **Título** del servicio.
- **Categoría / subcategoría**.
- **Paquetes** (3 niveles), cada uno con:
  - Descripción del entregable
  - **Precio** por paquete
  - **Tiempo de entrega** (delivery time)
  - **N° de revisiones** incluidas
- Sin subasta abierta: precio fijo por paquete.

## Lo que podemos aplicar a nuestro esquema
- Modelar **paquetes/tiers** (Basic/Standard/Premium) como entidad aparte, no solo un monto.
- `delivery_time` y `revisions` como atributos de cada paquete.
- `category` como taxonomía de primer nivel (sugerir tabla `categories`).
