# Toptal — Análisis de publicación de trabajos

**Fuente:** https://www.toptal.com/ (revisado en vivo)

## Qué es
Red exclusiva del **"Top 3%"** de talento (developers, designers, marketers, consultants, PMs). Altamente vetado (98% trial-to-hire).

## Cómo publican los trabajos
- No hay posting público abierto ni subasta: el cliente **describe su necesidad** y Toptal hace **matching** con talento pre-seleccionado (promedio < 24h).
- Flujo: hablar con un experto → talento a medida → prueba (pay only if satisfied).

## Campos / elementos clave de una publicación
- **Disciplina** (en vez de categorías amplias): Developers, Designers, Marketing Experts, Management Consultants, Project Managers, Product Managers, Sales Experts.
- Dentro de cada disciplina, **skills granulares** (p. ej. Developers → React, iOS, AI Engineers, PostgreSQL…).
- Perfiles de talento **verificados** con "previously at" (empresas top).
- Servicios de consultoría (Technology, Marketing Agency, Management Consulting).

## Lo que podemos aplicar a nuestro esquema
- El "posting" real es una **solicitud de necesidad** que luego se matchea: sugerir `matching/assignment` como estado aparte en nuestro flujo (ya tenemos `Application`/`accept_application`).
- `verified` / reputación como campo fuerte en `users` (nuestro `reputation_score`).
- Taxonomía por **disciplina + skill granular** en vez de solo categoría ancha.
