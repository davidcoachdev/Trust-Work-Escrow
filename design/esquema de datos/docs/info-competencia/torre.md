# Torre (Torre.ai) - Analisis de publicacion de trabajos

**Fuente:** https://www.torre.co/ y https://torre.ai/ (revisado en vivo)

## Que es
"The global job network". Red global de empleo que **automatiza el reclutamiento con IA en tiempo real**. Modelo freemium, siempre gratis para publicar. Fuerte presencia en **Latinoamerica** y mercado hispano/ingles.

## Como publican los trabajos
- El canal principal es **"Post a job" (gratis)** desde el panel de empresas.
- En vez de subasta abierta, Torre hace **matching automatico con IA** (desde 1B+ candidatos). El profesional "senala" (signal) interes.
- Tambien ofrece "Headhunt with AI", "Hire through staffing", "Build your own job board" (Subtorres) y API/MCP, ATS, Cultural fit, Enterprise.
- Las ofertas se llaman **opportunities** y soportan full-time, freelance e internships.

## Campos / elementos clave de una publicacion (opportunity)
- **Titulo** y **descripcion** del puesto.
- **Compensacion** declarada abiertamente (ej. "Starting at USD 20/hour").
- **Ubicacion / remoto** y tipo de empleo.
- **Skills** y **languages** del perfil requerido.
- **Psychometrics / cultural fit**: Torre construye el perfil del talento con rasgos conductuales (modelo HEXACO) y lo matchea por fit, no solo por CV.
- **Reputation** basada en senales (signalers/signalees), recomendaciones y contactos, no en estrellas tradicionales.

## Lo que podemos aplicar a nuestro esquema
- El matching por **IA + senales** (signal/recommend) es un patron potente: sugerir entidad `signals` o campo `reputation` derivado de interacciones (ya tenemos `reputation_score` en `users`).
- **Compensacion abierta** como campo de la publicacion (nuestro `budget_amount` + tipo).
- **Cultural fit / psychometrics** como metadatos opcionales en el perfil (extensible en `users`/Postgres).
- Modelo sin subasta: la "publicacion" es una opportunity que el sistema distribuye por matching, coherente con nuestro flujo on-chain de aceptacion de aplicacion.
