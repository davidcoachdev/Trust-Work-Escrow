# 📋 Encuesta Online de Validación — Trust Work Escrow

**Milestone:** 4 — Validación Inicial con Usuarios
**Formato:** Typeform / Google Forms (3-4 min)
**Propósito:** validación cuantitativa. Esta encuesta es la **versión cerrada (formulario)** del guion `01-preguntas-entrevista.md`. Las preguntas son **las mismas, en el mismo orden** que la entrevista 1:1, para que los datos coincidan sea presencial (videollamada) o por formulario.

> 📌 Coherencia: cada pregunta de acá tiene su par en `01`. No se agrega ni se saca nada. Las respuestas alimentan la ponderación del archivo `03-recomendaciones-trabajo.md`.

---

## Sección 0 — Filtro de segmento (para saber a quién leés)

**Q1. ¿Cuál es tu rol principal?** (selección única)
- [ ] Freelancer (vendo servicios)
- [ ] Cliente (contrato freelancers)
- [ ] Ambos
- [ ] Ninguno / solo curioso

> El guion `01` tiene Bloque A (Freelancers), B (Clientes) y C (Árbitros). En la encuesta, si responde "Freelancer" o "Ambos" se le muestra A; si responde "Cliente" se le muestra B; si marca interés en arbitraje, C. Para simplificar, acá van A y B completos; C queda como pregunta corta final opcional.

---

## 🧑‍💻 Bloque A — Freelancers (igual que `01`, Bloque A)

### A0. Intro / romper hielo
**Q2. ¿Qué hacés y cómo conseguís tus clientes hoy?** (texto abierto corto)
**Q3. ¿Hace cuánto trabajás como freelancer?** (texto / selección: menos de 1 año, 1-3, 3-5, 5+)

### A1. Dolores actuales
**Q4. ¿Qué es lo que más te frustra de las plataformas que usás (Upwork, Fiverr, LaborX)?** (texto abierto)
**Q5. ¿Cuánto tiempo pasa entre que entregás el trabajo y se hace el depósito del pago? ¿Y cuánto te descuenta la plataforma?** (texto abierto)
**Q6. ¿Alguna vez tuviste un problema o disputa con un pago?** (sí / no)
**Q7. ¿Cómo se resolvió?** (texto abierto, obligatorio si Q6 = sí; si no, se salta)

### A2. Jerarquía de prioridades (clave → alimenta ponderación)
**Q8. Si tuvieras que rankear estos 4 factores de mayor (1) a menor (4) importancia al elegir plataforma, ¿cómo los ordenás?**
(arrastrar o botones 1-4)
1. Comisiones bajas
2. Velocidad de pago
3. Confianza / seguridad
4. Resolución justa de disputas

> 🔢 Esta pregunta es el par de `01` A2.7. Se vuelca a la tabla de puntaje inverso de `03` (puesto 1 = 4 pts … puesto 4 = 1 pt).

**Q9. ¿Por qué pusiste primero a ese?** (texto abierto)

### A3. Disposición a migrar (la pregunta del millón)
**Q10. ¿Usarías un sistema con escrow on-chain en Solana, comisión <5% y pago inmediato al aprobar entrega?** (sí / no / quizás)
**Q11. Si ya pagás 20% en Upwork, ¿el ahorro solo te haría cambiarte, o necesitás algo más?** (selección única → mapea a A/B/C)
- [ ] A) Solo el ahorro me alcanza para cambiar
- [ ] B) Necesito confianza/garantía, el ahorro no alcanza
- [ ] C) El ahorro me llama, pero no migro sin confianza
**Q12. ¿Qué ayudaría para que realices el cambio desde Upwork/Fiverr a Trust Work Escrow mañana?** (texto abierto)

### A4. Confianza (factor que el feedback pidió documentar)
**Q13. ¿Qué te generaría confianza para meter tu primera plata en una plataforma nueva?** (multiselección)
- [ ] Reputación verificable
- [ ] Contrato auditado / open source
- [ ] Conocidos que ya la usen
- [ ] Garantía de disputa / árbitros
- [ ] Otra: ______
**Q14. ¿Te da más confianza ver los fondos bloqueados en un explorador on-chain que el sistema de reputación de Upwork?** (sí / no / no sé)

### A5. Cierre
**Q15. ¿Hay algo más que quieras agregar?** (texto abierto, opcional)
**Q16. ¿Te puedo contactar cuando tengamos versión para probar?** (email / Discord opcional)

---

## 🧑‍💼 Bloque B — Clientes (igual que `01`, Bloque B)

> Solo si en Q1 marcó "Cliente" o "Ambos".

### B1. Intro
**Q17. ¿Cómo encontrás y contratás freelancers hoy?** (texto abierto)
**Q18. ¿Cuántos contratás por mes y qué presupuesto manejas?** (texto abierto)

### B2. Dolores
**Q19. ¿Qué es lo que menos te gusta del proceso de pago actual?** (texto abierto)
**Q20. ¿Alguna vez perdiste plata con un freelancer? ¿Cómo lo manejaste?** (texto abierto)
**Q21. ¿Cómo generás confianza con alguien que contratás por primera vez?** (texto abierto)

### B3. Interés en escrow
**Q22. ¿Qué opinás de bloquear el pago al inicio y liberarlo solo al aprobar el trabajo?** (texto / escala 1-5)
**Q23. ¿Te daría más confianza ver los fondos en el explorador de Solana?** (sí / no)
**Q24. ¿Cambiarías a una plataforma sin comisiones aunque tengas que usar wallet crypto?** (sí / no / quizás)

---

## 🧑‍⚖️ Bloque C — Árbitros (opcional, igual que `01`, Bloque C)

**Q25. ¿Tenés experiencia resolviendo disputas freelance?** (sí / no)
**Q26. ¿Participarías en un sistema de justicia descentralizado? ¿Qué incentivo necesitarías?** (texto abierto)

---

## 📊 Cómo volcar los resultados a la ponderación

1. **Exportá** respuestas a CSV desde Typeform/Forms.
2. **Q8 (ranking, par de `01` A2.7):** por cada respuesta, asigná puntaje inverso (1º=4, 2º=3, 3º=2, 4º=1) y sumá por factor.
3. **Normalizá:** `puntaje_factor / (respuestas × 4)` → % de importancia relativa.
4. **Q11 (par de `01` A3.10):** contá frecuencias de A / B / C → si B+C dominan, confianza pesa tanto o más que ahorro.
5. **Q10 y Q14:** frecuencias → disposición a usar y peso de confianza on-chain.
6. Volcá todo en la tabla **Resultados de Encuesta** del plan de validación (`trust-escrow-milestone-4-plan-validacion.md`, Sección 5) y en el reporte final.

### Tabla de volcado (ejemplo a llenar)

| Pregunta (par `01`) | Resultado (n=__) | Interpretación |
|---------------------|------------------|----------------|
| Q8 / A2.7 ranking Confianza % | | |
| Q8 / A2.7 ranking Comisiones % | | |
| Q8 / A2.7 ranking Velocidad % | | |
| Q8 / A2.7 ranking Disputas % | | |
| Q10 / A3.9 usaría sistema | | |
| Q11 / A3.10 cat A / B / C | | |
| Q14 / A4.13 confianza on-chain | | |

---

## 🔗 Coherencia con el resto
- `01-preguntas-entrevista.md` → MISMA estructura y preguntas (versión 1:1 presencial/videollamada).
- `02-mercado-objetivo.md` → define el segmento al que difundir (freelancer crypto-native).
- `03-recomendaciones-trabajo.md` → explica la ponderación que esta encuesta alimenta.
