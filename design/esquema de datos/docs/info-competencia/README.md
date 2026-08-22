# info-competencia — Indice y sintesis

Reportes de competidores revisados en vivo para disenar mejor la **distribucion de publicaciones** en Trust-Work-Escrow v3.

**Informe principal:** [informe-consolidado.md](./informe-consolidado.md) — sintesis de las 7 plataformas + plan de implementacion mapeado a nuestro split on-chain/Postgres/Mongo.

## Plataformas analizadas
- [Fiverr](./fiverr.md) — gigs con paquetes escalonados (Basic/Standard/Premium).
- [Freelancer](./freelancer.md) — proyecto + subasta + milestones + presupuesto fijo/por hora.
- [Workana](./workana.md) — foco LatAm; dos modelos (devs full-time / freelancers por proyecto u horas).
- [Toptal](./toptal.md) — red vetada Top 3%; matching por disciplina + skill granular, sin posting abierto.
- [PeoplePerHour](./peopleperhour.md) — hibrido: Post Project (subasta) + Offers fijas (Hourlies) + IA de matching.
- [Torre](./torre.md) — red global con IA en tiempo real; opportunities + matching por senales y cultural fit.
- [LinkedIn](./linkedin.md) — red profesional general; post-a-job con taxonomia amplia, Open To Work y ATS (revision parcial: el formulario requiere login).

> Notas de acceso: Upwork bloqueo el fetch directo (HTTP 403); LinkedIn solo expuso la landing publica (el posting esta tras login). Los reportes se basan en lo observable en vivo mas estructura conocida del modelo.

## Patrones comunes en como publican los trabajos
1. **Categoria / taxonomia** de primer nivel (siempre presente).
2. **Titulo + descripcion** (nosotros: `jobs_metadata.title/description`).
3. **Presupuesto** con `budget_type`: fixed | hourly (Freelancer, Workana, PPH).
4. **Skills / tags[]** granulares para busqueda/matching (todos).
5. **Milestones / hitos** como unidad de pago (Freelancer confirma esto; ya lo tenemos on-chain).
6. **Paquetes/tiers** (Fiverr, PPH Offers): 3 niveles con precio, entrega y revisiones.
7. **Modelo hibrido** en varios: subasta abierta vs oferta fija lista.
8. **Reputacion / verificacion**: ratings, reviews, talento vetado (Toptal, PPH) -> nuestro `reputation_score` + `reviews`.
9. **Matching por senales / IA** (Torre, Toptal, LinkedIn, PPH): la "publicacion" dispara distribucion automatica, no solo un tablon.

## Sugerencias para nuestro esquema de publicaciones (`jobs_metadata` + on-chain `Job`)
Campos a considerar agregar a `jobs_metadata` (Postgres):
- `category` (FK a tabla `categories`, jerarquica) y `skills text[]`.
- `budget_type` (fixed | hourly | package) y `budget_amount`.
- `engagement_type` (project | full_time | hourly) y `employment_type` (full-time | part-time | contract | internship).
- `publication_type` (bidding | fixed_offer) — modelo hibrido.
- `seniority` y `language` para filtros.
- Relacion 1:N `job_packages` (tiers Basic/Standard/Premium: precio, delivery_time, revisions).
- Relacion 1:N `job_milestones` ya cubierta por `milestones_metadata`.
- Entidad `signals` / campo `reputation` derivado de interacciones (modelo Torre/LinkedIn).

Esto mantiene lo critico on-chain (amount, status, milestones, deadlines) y lo descriptivo/clasificatorio en Postgres, respetando el split definido.

*Carpeta: `design/esquema de datos/docs/info-competencia/`*
