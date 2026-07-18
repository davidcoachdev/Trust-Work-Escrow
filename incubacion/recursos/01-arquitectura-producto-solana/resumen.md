# 📋 Resumen — Arquitectura de producto sobre Solana para founders

**Expositor:** Alex Ramírez (Senior Solana Engineer en Dermarket; co-founder/CTO de Mutrack)
**Duración:** ~1:18:57
**Link:** https://youtu.be/9EALzk-WmUs
**Transcripción completa:** `./transcripcion.md`

---

## Puntos clave

1. **3 decisiones caras de revertir:** (1) esquema de cuentas y PDAs, (2) qué va on-chain y qué no, (3) modelo de confianza de los datos.
2. **Custodia de fondos siempre on-chain:** "todo lo que sea custodia de fondos, escrow/fideicomiso, siempre ponerlo on chain". → Valida nuestra arquitectura de escrow.
3. **Costo de almacenamiento:** renta en Solana; usar NFTs comprimidos + off-chain (S3/IPFS) para datos pesados. No meter todo on-chain.
4. **Confianza de datos es el dolor real:** en Mutrack se quedaron fondos bloqueados en escrow porque no había árbitro que decidiera a quién darlos ("el contrato no sabía a quién darle los fondos y los fondos se quedaron bloqueados para siempre"). → **Valida que el sistema de disputas con árbitro es necesario**.
5. **Disputas como MVP:** empezar simple (como Binance P2P: partes se resuelven entre sí o alguien del equipo revisa) y luego evolucionar a jurado descentralizado por fases. No construir jurado descentralizado en stage 1.
6. **Abstraer la wallet del usuario:** usar Privy o Tiplink para que el usuario no sepa que usa crypto. Impacta adopción.
7. **Conseguir usuarios = vender:** "Vender, hermano, vender, vender, vender" — igual que web2.
8. **MVP y no sobre-ingeniería:** resolver el problema de hoy en 2-3 semanas; iterar con usuarios reales.
9. **Seguridad:** validaciones reales on-chain; considerar auditoría o revisión con IA; multisig (Squads) para deployment.

---

## Implicaciones para Trust Work Escrow

- ✅ Confirma que el **escrow on-chain** es el lugar correcto para custodia de fondos (hipótesis H2 del plan de milestone 4).
- ✅ Valida que el **árbitro en disputas** no es opcional: es justo lo que le falló a Mutrack. Nuestra sección A4 de confianza y el factor "resolución justa de disputas" del ranking (`herramientas/01` A2.7) están bien puestos.
- ✅ Respalda nuestro enfoque de **MVP de disputas simple** antes que jurado descentralizado — útil para priorizar en las entrevistas.
- 💡 Para el **mensaje de adopción**: el freelancer crypto-native SÍ quiere ver el escrow (transparencia), pero el cliente menos tech podría necesitar abstracción de wallet. Preguntar esto en las entrevistas (`herramientas/01` B3.7).
- 💡 Usar "vender = salir a buscar usuarios" como empuje para nuestras entrevistas 1:1 y la convocatoria WayLearn.

---

## Usos en el proyecto

- `entregables/milestone-4-validacion-usuarios/respuestas/mensaje-waylearn-milestone4.md` → cita este resumen.
- `entregables/milestone-4-validacion-usuarios/herramientas/01-preguntas-entrevista.md` → sección A4 (confianza) y B3.7 (adopción/wallet).
- Milestone 2 (Arquitectura) → decisiones on/off-chain y modelo de escrow.
