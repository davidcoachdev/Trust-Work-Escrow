# LinkedIn — Analisis de publicacion de trabajos

**Fuente:** https://www.linkedin.com/jobs/ (revisado en vivo; el formulario de posting requiere login)

## Que es
La red profesional mas grande del mundo. No es solo freelance: cubre empleo full-time, part-time, contract/freelance e internships. Fuerte en "Open To Work" y ATS de empresas.

## Como publican los trabajos
- CTA **"Post a job"** -> linkedin.com/talent/post-a-job (requiere cuenta empresa).
- Taxonomia amplia de categorias (observadas en la landing): Engineering, Business Development, Finance, Administrative, Retail, Customer Service, Operations, IT, Marketing, HR, Healthcare, Sales, Program/Project Management, Accounting, Arts and Design, Consulting, Education, Legal, Media, Product Management, etc.
- Funcion **"Open To Work"**: el candidato senala disponibilidad (privado a reclutadores o publico).
- Integracion con LinkedIn Learning (habilidades/cursos).

## Campos / elementos clave de una publicacion (job post estandar)
- Titulo del puesto y empresa.
- Ubicacion y modalidad (on-site / remote / hybrid).
- Tipo de empleo (full-time, part-time, contract, internship) y seniority.
- Descripcion y responsabilidades.
- Habilidades (skills) y requisitos.
- Rango salarial (opcional, cada vez mas requerido).
- Postulacion via ATS / Easy Apply.

## Lo que podemos aplicar a nuestro esquema
- Taxonomia de categorias muy amplia: nuestro `category` debe ser flexible/jerarquico.
- `employment_type` (full-time | part-time | contract | internship) y `seniority` como campos de la publicacion.
- `open_to_work` / disponibilidad como senal de matching (similar a los "signals" de Torre).
- Integracion de habilidades con formacion sugiere entidad `skills` enlazada a perfiles.
