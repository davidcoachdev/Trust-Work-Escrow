# 🎖️ Insignias / Badges

Esta carpeta reúne las **insignias (badges)** que el equipo de Trust Work Escrow va obteniendo
a lo largo del programa de incubación WayLearn & Solana LATAM Labs y otros eventos (hackathons,
residencias, completaciones de milestones, etc.).

Cada insignia tiene su propia subcarpeta con:
- `insignia.md` → datos (nombre, otorgante, fecha, criterio, link de verificación) + nota personal de qué se hizo para ganarla.
- `imagen/` → (opcional) el archivo de la insignia en PNG/SVG si lo tenés.

---

## 📌 Índice de insignias

| # | Insignia | Otorgante | Fecha | Estado | Carpeta | Imagen |
|---|----------|-----------|-------|--------|---------|--------|
| 0 | Selección (Incubación confirmada) | WayLearn & Solana LATAM Labs | 15 Jun 2026 | ✅ obtenida | `00-seleccion/` | `seleccion.md` + `Trust-Work-Escrow-insignias-seleccion.png` |
| 1 | Roadmap | WayLearn & Solana LATAM Labs | 22 Jun 2026 | ✅ obtenida | `01-roadmap/` | `roadmap.md` + `Trust-Work-Escrow-insignias-roadmap.png` |
| 2 | Foundation | WayLearn & Solana LATAM Labs | 29 Jun 2026 | ✅ obtenida | `02-foundation/` | `foundation.md` + `Trust-Work-Escrow-insignias-foundation.png` |
| 3 | Arquitectura | WayLearn & Solana LATAM Labs | 6 Jul 2026 | ✅ obtenida | `03-arquitectura/` | `arquitectura.md` + `Trust-Work-Escrow-insignias-arquitectura.png` |
| 4 | Validación | WayLearn & Solana LATAM Labs | _(31 Jul 2026)_ | ⏳ pendiente | `04-validacion/` | `validacion.md` + _(imagen por agregar)_ |
| 5 | MVP | WayLearn & Solana LATAM Labs | _(21 Ago 2026)_ | ⏳ pendiente | `05-mvp/` | `mvp.md` + _(imagen por agregar)_ |
| 6 | Pitch | WayLearn & Solana LATAM Labs | _(28 Ago 2026)_ | ⏳ pendiente | `06-pitch/` | `pitch.md` + _(imagen por agregar)_ |
| 7 | Demo Day | WayLearn & Solana LATAM Labs | _(31 Ago 2026)_ | ⏳ pendiente | `07-demo-day/` | `demo-day.md` + _(imagen por agregar)_ |

> Las 3 primeras insignias ya están en su carpeta con su `<nombre>.md` + imagen. Las fases 4-7 tienen el esqueleto listo (`04` a `07`); al obtenerlas, pegar la imagen y completar el `<nombre>.md`.
> Texto para compartir en redes: `publicacion-redes.md` (abajo de esta carpeta).

---

## 🧩 Plantilla de insignia

Para agregar una, creá una carpeta `NN-nombre-insignia/` con un `insignia.md` así:

```markdown
# 🎖️ [Nombre de la insignia]

- **Otorgante:** WayLearn & Solana LATAM Labs (u otro)
- **Fecha de obtención:** YYYY-MM-DD
- **Criterio / cómo se ganó:** _qué había que hacer_
- **Link de verificación:** _URL o captura_
- **Milestone / etapa asociada:** _ej. Milestone 4 — Validación con Usuarios_

## Nota del equipo
_Qué hicimos para ganarla, qué aprendimos, cómo suma al proyecto._
```

Y opcionalmente una subcarpeta `imagen/` con el PNG/SVG.

---

## 🔗 Relación con el proyecto

- Las insignias son evidencia de progreso ante WayLearn (útiles para reportes de milestone).
- **No se duplican dentro de `entregables/`**: cada milestone las referencia con un link a esta carpeta (`incubacion/insignias/NN-nombre/`). Esto evita desorden y lecturas incompletas.
- El índice general en `incubacion/README.md` linkea cada insignia obtenida desde la tabla de progreso de milestones.
