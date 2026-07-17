# 🛠️ Recomendaciones de Trabajo para las Entrevistas

**Milestone:** 4 — Validación Inicial con Usuarios
**Contenido:** por dónde hacer las entrevistas + cómo sacar la **ponderación** de las respuestas para decidir el motor de adopción.

---

## Parte 1 — ¿Por qué medio hacer las entrevistas?

### Canales según perfil

| Perfil | Medio recomendado | Por qué |
|--------|-------------------|---------|
| Freelancer crypto-native | **Discord Solana / Telegram / r/solana** + videollamada (Meet/Zoom) | Están ahí, confían en lo escrito; la videollamada da profundidad |
| Freelancer tech general | **r/freelance, grupos WhatsApp, LinkedIn DM** | Más amplio, menos crypto |
| Cliente tech-savvy | **LinkedIn DM / Twitter X** outreach directo a founders | Decisión B2B, se contacta 1:1 |
| Árbitro | **LinkedIn legal-tech, DAOs** | Outreach directo, nicho |

### Formato
- **1:1 videollamada (20-30 min):** para las 5-8 entrevistas profundas. Es donde sale la evidencia cualitativa.
- **Encuesta online (3-4 min):** para las 20-50 respuestas cuantitativas (Typeform/Google Forms). Complementa, no reemplaza.
- **Presencial / evento:** si hay meetup Web3 local, aprovechalo para 1-2 charlas rápidas.

### Tips de ejecución
- Grabá (con permiso) o tomá notas en vivo con el template del archivo `01-preguntas-entrevista.md`.
- No vendas el producto: investigás, no convencés.
- Empezá por las preguntas abiertas (dolor) antes de mostrar la solución.

---

## Parte 2 — Cómo sacar la ponderación de las respuestas

El objetivo es cuantificar **qué factor pesa más** para migrar (comisiones / velocidad / confianza / disputas) y **si el ahorro alcanza o pesa más la confianza**.

### Paso 1 — Pregunta de ranking (en la entrevista)
Usá la **pregunta A2.6**: pediles que ordenen de 1 a 4:
`Comisiones` · `Velocidad de pago` · `Confianza` · `Resolución de disputas`

### Paso 2 — Asignar puntaje inverso
El puesto 1 = 4 puntos, puesto 2 = 3, puesto 3 = 2, puesto 4 = 1.

Ejemplo para 1 entrevista:
| Factor | Puesto | Puntaje |
|--------|--------|---------|
| Confianza | 1 | 4 |
| Comisiones | 2 | 3 |
| Velocidad | 3 | 2 |
| Disputas | 4 | 1 |

### Paso 3 — Sumar y normalizar (todas las entrevistas)
Sumás los puntajes de cada factor entre todas las entrevistas y los dividís por el máximo posible (entrevistas × 4) para obtener un **% de importancia relativa**.

Ejemplo con 8 entrevistas:
| Factor | Suma de puntajes | % relativo |
|--------|-----------------|------------|
| Confianza | 28 | 87.5% |
| Comisiones | 24 | 75% |
| Velocidad | 16 | 50% |
| Disputas | 12 | 37.5% |

> 🔎 Interpretación: si Confianza y Comisiones dominan, el motor de adopción es **confianza + ahorro combinados**, no solo precio.

### Paso 4 — Cruzar con la pregunta de migración (A3.9 / A3.10)
Etiquetadas las respuestas en 3 categorías:
- **(A) Solo ahorro:** "me cambio por la comisión más baja".
- **(B) Solo confianza:** "necesito garantía de disputa / ver el fondo bloqueado".
- **(C) Ambos:** "el ahorro me llama, pero no migro sin confianza".

Contás frecuencias: si la mayoría cae en **(C)** o **(B)**, confirmás que **la confianza pesa tanto o más que el ahorro** → eso responde la pregunta del programa.

### Paso 5 — Escala de urgencia (1-5)
En la encuesta online, la pregunta *"¿qué tan urgente es para vos una alternativa?"* da una media. Si media ≥ 4, el dolor es real y urgente.

### Fórmula rápida de "Motor de Adopción" (sugerida)
```
Motor = (Promedio puntaje Confianza + Promedio puntaje Comisiones) / (Promedio Velocidad + Promedio Disputas)
```
- Si Motor > 1.5 → el producto debe comunicar **confianza + ahorro** en el mensaje principal.
- Si Motor ≤ 1 → el precio es el driver y alcanza con comunicar comisión baja.

---

## Checklist de cierre por entrevista
- [ ] Ranking de prioridades registrado (paso 2)
- [ ] Categoría de migración asignada (A/B/C)
- [ ] Al menos 1 quote textual anotado
- [ ] Insight accionable identificado

> Al terminar las 8 entrevistas, volcá los totales en el `Registro de Entrevistas` del plan principal y en el reporte de resultados del milestone 4.
