# 📋 Resumen — Security in the AI Era

**Expositor:** Juan Marchetto (founder de Solana Argentina / Superteam Argentina; 6 años en el ecosistema Solana; ex-founder/CTO de varios proyectos)
**Duración:** ~51 min
**Link:** https://youtu.be/ROyh5JUhVdE
**Transcripción completa:** `./transcripcion.md`

---

## Contexto

Charla del programa WayLearn & Solana LATAM Labs. Juan analiza cómo la IA cambió la seguridad: ahora los agentes de IA **leen código a gran escala y encuentran/exploitan vulnerabilidades**, y también **introducen bugs** al escribir código. Atacantes y defensores tienen el mismo "upgrade".

---

## Puntos clave

1. **La curva es exponencial:** desde fines de 2024 la IA identifica y explota bugs (CVEs). En sept 2025 un solo investigador entregó 170 issues válidos. Antes impensable, ahora realidad.
2. **Fugas de secrets (el fallo #1):** dar a la IA acceso a la carpeta del proyecto donde vive `.env` expone claves (API keys, private keys). Las empresas de IA auditan los prompts con personas y los comparten/venden a terceros. Riesgo de exposición.
3. **Consologueo de secretos:** herramientas de log (error monitoring) a menudo loguean passwords/secret phrases sin que uno lo sepa hasta que es tarde. Caso real: una wallet que logueaba las seed phrases de todos los que se unían.
4. **Rotar secrets SIEMPRE:** sobre todo tras sprints de desarrollo asistido por IA (MVP en una semana). La rotación debe ser rutinaria (semanal/mensual según criticidad), no solo ante incidente. Si hay hot signers o wallet en `.env`, rotar primero.
5. **Gates de revisión humana:** alguien debe leer cada PR. Herramientas de CI escanean secretos y bloquean. Minimizar privilegios de los agentes.
6. **TDD como defensa:** escribir spec + tests primero, que la IA complete los tests. Garantiza que cumpla lo esperado y no "alucine".
7. **Usá la IA como atacante:** correr herramientas de security review sobre tu propio código para encontrar vectores antes que otros. Ej.: Claude Code Security Review, Gemini Code Assist Autofix, OpenCode Security, Anthropic/Gemini/Google code scanners.
8. **Revisiones escopiadas:** revisar por PRs, con distintos agentes, contexto limpio (cerrar sesión y abrir nueva), que un agente revise los findings del otro.
9. **Base de desarrollo diario:** no confiar ciegamente en el output del modelo; privilegio mínimo por defecto; **nunca armar la "trifecta letal"**: datos privados + contenido no confiable + vía para llamar afuera en el mismo agente.
10. **Aprovechar el "red teaming" de la IA:** pedirle que actúe de abogado del diablo ("¿qué está mal en lo que acabamos de hacer?").
11. **Recursos de Solana:** Superteam Brasil sacó un **Auditor Skill** (testea todos los vectores de ataque conocidos en smart contracts, jerarquiza por criticidad, sugiere fixes — "el informe solo vale oro"); **Solana Kit** (skills con criterios de seguridad); newsletter de seguridad de Clean Gever.
12. **Para founders no-técnicos (MVP con IA):** al llegar al MVP, cambiar manualmente TODAS las credenciales que se le dieron a la IA (están en `.env` en el directorio principal). El 95% del problema es separar el conocimiento de la IA del secreto. Si el MVP escala, contratar un CTO/founder engineer.
13. **.gitignore + separar dev/prod:** enumerar en `.gitignore` lo que no se sube; usar claves de prueba en dev y cargar las de producción en la plataforma de deploy (no en `.env` local). Truco: poner `.env` en directorio padre, fuera del proyecto.
14. **No escatimar en seguridad:** destinar presupuesto a auditoría, bug bounties (Cantina, OtterSec, Trail of Bits, Zellic, etc.). Un bounty de US$100k desincentiva robar US$1M. El concepto principal: "ahorren en lo que sea, menos en seguridad".
15. **Prompt injection / prompt poisoning:** el vector MÁS difícil de parar hoy. Contramedidas: no usar skills de autor desconocido, no autorizar búsqueda en webs no confiables, no copiar/pegar sin leer. Hay formas sutiles (recomendar comprar ciertas cryptos a largo plazo).

---

## Implicaciones para Trust Work Escrow

- 💡 **El escrow on-chain protege el dinero**, pero la **seguridad del desarrollo** es responsabilidad nuestra: rotar secrets, no commitear `.env` (ya lo tenemos en `.gitignore` con `.atl/`), revisión de PRs.
- 💡 Para el milestone 4 (validación) y el futuro MVP: el factor **confianza/seguridad** del escrow (sección A4 de `01`) se refuerza con "el dinero vive en un programa auditado on-chain, no en nuestro server".
- 💡 **Bug bounty + auditoría** (Cantina/OtterSec) como parte del plan de seguridad cuando el MVP escale — citarlo en el plan de arquitectura/milestone 2.
- 💡 El **Auditor Skill de Superteam** es accionable ya: correrlo contra los smart contracts del escrow apenas existan.
- 💡 Adopción: el usuario no-técnico (cliente) delega en nosotros la seguridad; nuestra propuesta de valor es justo que no necesita manejar keys. Preguntar en entrevistas (`01` B3) si el usuario confía en que la plataforma maneje la custodia.

---

## Usos en el proyecto

- `entregables/milestone-4-validacion-usuarios/respuestas/mensaje-waylearn-milestone4.md` → menciona esta charla como refuerzo del ángulo de confianza del escrow.
- `herramientas/01-preguntas-entrevista.md` → sección A4 (confianza) y B3 (adopción).
- `herramientas/02-mercado-objetivo.md` / `03-recomendaciones-trabajo.md` → factor confianza.
