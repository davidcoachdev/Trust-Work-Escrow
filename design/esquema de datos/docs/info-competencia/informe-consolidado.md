# Informe Consolidado - Analisis de Competencia y Plan de Implementacion

**Alcance:** Disenar mejor la distribucion de las publicaciones de trabajos en Trust-Work-Escrow v3, basandose en como las principales plataformas freelance publican y estructuran sus avisos.

**Fuentes:** los 7 reportes individuales en `docs/info-competencia/` (fiverr, freelancer, workana, toptal, peopleperhour, torre, linkedin) mas `reparto-datos.md` y los esquemas ya definidos.

---

## 1. Resumen ejecutivo

Se revisaron 7 plataformas. El patron comun es: **categoria + titulo/descripcion + presupuesto (fixed/hourly/package) + skills + hitos + reputacion/verificacion + matching por IA/senales**.

Nuestro sistema ya cubre lo critico **on-chain** (amount, status, milestones, deadlines, payout %, hashes). El gap real esta en la **capa descriptiva/clasificatoria off-chain**: taxonomia de categorias, skills, paquetes/tiers, tipo de publicacion (subasta vs oferta fija), senales de interes y disponibilidad. Ahi es donde podemos enriquecer la distribucion de publicaciones sin tocar el contrato.

---

## 2. Hallazgos por plataforma

| Plataforma | Modelo de publicacion | Lo distintivo para nosotros |
|---|---|---|
| [Fiverr](./fiverr.md) | Gigs con 3 paquetes (Basic/Standard/Premium) | Precio fijo por paquete, `delivery_time`, `revisions` |
| [Freelancer](./freelancer.md) | Proyecto + subasta + milestones | `budget` fixed/hourly, skills, hitos de pago |
| [Workana](./workana.md) | Dos modelos (devs full-time / freelancers por proyecto u horas) | Foco LatAm, `engagement_type` |
| [Toptal](./toptal.md) | Red vetada Top 3%, matching por disciplina + skill | Sin posting abierto; verificacion fuerte |
| [PeoplePerHour](./peopleperhour.md) | Hibrido: Post Project (subasta) + Offers fijas (Hourlies) + IA | Paquetes fijos + `hourly_rate` |
| [Torre](./torre.md) | Opportunities + matching por IA en tiempo real | `senales` (signal), cultural fit, reputation por senales |
| [LinkedIn](./linkedin.md) | Post-a-job + taxonomia amplia + Open To Work + ATS | `employment_type`, `seniority`, `open_to_work` |

> Notas de acceso: Upwork bloqueo el fetch (403) y LinkedIn solo expuso la landing (posting tras login). Ver detalles en cada reporte.

---

## 3. Patrones transversales de publicacion

1. **Categoria / taxonomia** de primer nivel (siempre presente, a menudo jerarquica).
2. **Titulo + descripcion** del aviso.
3. **Presupuesto** con `budget_type` (fixed | hourly | package) + monto.
4. **Skills / tags[]** granulares para busqueda y matching.
5. **Milestones / hitos** como unidad de pago (Freelancer lo confirma; ya lo tenemos on-chain).
6. **Paquetes / tiers** (3 niveles: precio, entrega, revisiones) en modelos tipo gig.
7. **Modelo hibrido**: subasta abierta vs oferta fija lista.
8. **Reputacion / verificacion**: ratings, reviews, talento vetado.
9. **Matching por senales / IA**: la publicacion dispara distribucion automatica, no solo un tablon.

---

## 4. Que aplicamos a Trust-Work-Escrow v3

### 4.1 Recordatorio del split (principio rector)
- **On-chain (contrato):** lo que el programa necesita para hacer cumplir fondos y trust (pubkeys, amounts, status, deadlines, contadores, %, hashes).
- **Postgres:** descriptivo / clasificatorio (metadatos, relaciones, filtros).
- **Mongo:** voluminoso / no estructurado (chat E2EE, evidence, logs).

### 4.2 Postgres - nuevas tablas y columnas sugeridas

**Nueva tabla `categories`** (taxonomia jerarquica):
```sql
CREATE TABLE categories (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  parent_id UUID REFERENCES categories(id) ON DELETE CASCADE,
  name TEXT NOT NULL,
  slug TEXT UNIQUE NOT NULL
);
```

**Nueva tabla `job_packages`** (modelo Fiverr / PPH Hourlies):
```sql
CREATE TABLE job_packages (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  job_pda TEXT NOT NULL REFERENCES jobs_metadata(pda_address) ON DELETE CASCADE,
  tier TEXT NOT NULL CHECK (tier IN ('basic','standard','premium')),
  price BIGINT NOT NULL,
  delivery_time_days INTEGER,
  revisions INTEGER DEFAULT 0,
  description TEXT,
  UNIQUE (job_pda, tier)
);
```

**Nueva tabla `job_interests`** (las "senales" livianas de Torre / LinkedIn Open To Work):
```sql
CREATE TABLE job_interests (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  job_pda TEXT NOT NULL REFERENCES jobs_metadata(pda_address) ON DELETE CASCADE,
  user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  created_at TIMESTAMPTZ DEFAULT NOW(),
  UNIQUE (job_pda, user_id)
);
```
> Nota: `job_interests` es mas liviano que el PDA `Application` on-chain (que sigue siendo la postulacion real para dedup/count). La senal es solo expresion de interes para matching.

**ALTER `jobs_metadata`** (enriquecer la publicacion):
```sql
ALTER TABLE jobs_metadata
  ADD COLUMN category_id UUID REFERENCES categories(id),
  ADD COLUMN skills TEXT[],
  ADD COLUMN budget_type TEXT CHECK (budget_type IN ('fixed','hourly','package')),
  ADD COLUMN employment_type TEXT CHECK (employment_type IN ('full_time','part_time','contract','internship')),
  ADD COLUMN engagement_type TEXT CHECK (engagement_type IN ('project','full_time','hourly')),
  ADD COLUMN publication_type TEXT CHECK (publication_type IN ('bidding','fixed_offer')),
  ADD COLUMN language TEXT,
  ADD COLUMN location TEXT,
  ADD COLUMN is_remote BOOLEAN DEFAULT FALSE;
```

**ALTER `users`** (perfil de talento):
```sql
ALTER TABLE users
  ADD COLUMN verified BOOLEAN DEFAULT FALSE,
  ADD COLUMN hourly_rate BIGINT,
  ADD COLUMN open_to_work BOOLEAN DEFAULT FALSE;
```

### 4.3 On-chain (contrato) - minimo
Mantener `Job`, `Milestone`, `Application`, `Dispute` como estan. **No** mover titulo/descripcion ni budget_type al chain (ya viven off-chain). El `amount` on-chain ya cubre el presupuesto total escrowado; los paquetes y la senal son off-chain. La postulacion real sigue siendo el PDA `Application` (dedup + `applications_count`).

### 4.4 Mongo - sin cambios grandes
`chat_messages` y `dispute_evidence` ya estan. El feed de eventos de publicacion/interes ya tiene destino en `events`. Mantener.

### 4.5 Backend - endpoints sugeridos
- `POST /jobs/:pda/packages` - crear paquetes off-chain (modelo Fiverr/PPH).
- `GET /jobs?category=&skills=&budget_type=&employment_type=&publication_type=` - filtros de distribucion.
- `POST /jobs/:pda/interest` - senal liviana (Torre/LinkedIn).
- `GET /categories`, `POST /categories` - taxonomia.
- `PATCH /users/:wallet` - `verified`, `hourly_rate`, `open_to_work`.

---

## 5. Roadmap sugerido (prioridad)

- **P0 (base de distribucion):** tabla `categories` + ALTER `jobs_metadata` (category_id, skills[], budget_type, employment_type, engagement_type, publication_type, language, is_remote) + endpoint de filtros. Esto ya habilita busqueda/clasificacion como Freelancer/Workana/LinkedIn.
- **P1 (modelos ricos):** `job_packages` (Fiverr/PPH) + `job_interests` / senales (Torre/LinkedIn).
- **P2 (confianza y matching):** `verified`, `open_to_work`, `hourly_rate` en `users` + matching por senales.
- **P3 (opcional/avanzado):** cultural fit / psychometrics (Torre) como metadato extendido en Postgres.

---

## 6. Referencias
- Reportes individuales: [fiverr](./fiverr.md), [freelancer](./freelancer.md), [workana](./workana.md), [toptal](./toptal.md), [peopleperhour](./peopleperhour.md), [torre](./torre.md), [linkedin](./linkedin.md)
- [reparto-datos.md](../overview/reparto-datos.md) (split on-chain/Postgres/Mongo)
- Esquemas actuales en `postgres/esquemas/`, `mongo/esquemas/`, `contrato/esquemas/`
- Diagramas ER en `postgres/diagramas/`, `mongo/diagramas/`, `contrato/diagramas/`
