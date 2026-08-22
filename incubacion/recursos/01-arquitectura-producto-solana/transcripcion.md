# 🎙️ Transcripción — Arquitectura de producto sobre Solana para founders

**Expositor:** Alex Ramírez (Senior Solana Engineer en Dermarket; co-founder/CTO de Mutrack)
**Duración:** ~1:18:57 (78 min)
**Link:** https://youtu.be/9EALzk-WmUs
**Origen:** Sesiones del programa WayLearn & Solana LATAM Labs

---

## Transcripción completa

> Nota: transcripción obtenida desde el entorno (audio → texto). Puede contener pequeñas imprecisiones de reconocimiento de voz.

[CONTENIDO DE LA TRANSCRIPCIÓN]

Alex Ramírez abre la charla presentándose: es Senior Solana Engineer en Dermarket y fue co-founder y CTO de Mutrack, una startup de tracking logístico construida sobre Solana. Dice que la charla no es puramente técnica sino "desde la visión de founder": cómo decidir la arquitectura de un producto sobre Solana sin morir en el intento.

### Las 3 decisiones caras de revertir

Alex enfatiza que hay tres decisiones que, una vez tomadas, son muy costosas de cambiar:

1. **Esquema de cuentas y PDAs (Program Derived Addresses).** El modelo de cuentas de Solana es distinto a Ethereum (no hay `mapping` global). Si diseñás mal las PDAs y la relación entre cuentas, reestructurar después implica migrar estado on-chain, lo cual es doloroso.
2. **Qué va on-chain y qué no.** No todo debe ir on-chain. Solana cobra "renta" por almacenamiento (cuentas). Meter datos pesados on-chain es caro y no escala.
3. **Modelo de confianza de los datos.** Cómo se sabe que un dato off-chain es verdadero (oráculos, firmas, autoridades). Este es el dolor real, según él.

### Custodia de fondos siempre on-chain

Dice textualmente: *"Todo lo que sea custodia de fondos, escrow, fideicomiso, siempre ponerlo on chain"*. La razón: si los fondos están en una cuenta controlada por un server off-chain, se pierde la garantía de descentralización y podés perder el control o ser hackeado. El escrow on-chain es la ventaja competitiva de construir sobre Solana.

### Costo de almacenamiento y datos pesados

Recomienda usar **NFTs comprimidos** (compressed NFTs / Metaplex) para metadatos y guardar lo pesado off-chain (S3, IPFS) referenciando el hash on-chain. No meter todo en cuentas (renta cara).

### El error de Mutrack: fondos bloqueados por falta de árbitro

Cuenta una anécdota clave: en Mutrack manejaban pagos entre partes mediante un escrow on-chain, pero *"llegó un punto en el que el contrato no sabía a quién darle los fondos y los fondos se quedaron bloqueados para siempre"*. No tenían un árbitro (humano o sistema) que decidiera la resolución de la disputa. Esta es la lección: **el escrow necesita un mecanismo de resolución de disputas claro**, si no, el dinero queda atrapado.

### Disputas como MVP (Q&A con founder de "Steaky")

Un founder del público (menciona que trabaja en "Steaky", que tiene el problema de disputas) pregunta cómo resolver disputas en un escrow. Alex recomienda:

- **Empezar simple:** tipo Binance P2P, donde las partes intentan resolverse entre sí y, si no, alguien del equipo revisa el caso. Al inicio hay pocas disputas, así que un proceso manual alcanza.
- **Evolucionar por fases:** luego se puede pasar a un jurado descentralizado (gobernanza, token holders votando), pero **no en el stage 1**. Construir jurado descentralizado desde el día 1 es sobre-ingeniería.

### Abstraer la wallet del usuario

Alex dice que para adopción masiva hay que **abstraer la wallet**: el usuario no debería saber que tiene una wallet crypto. Menciona servicios como **Privy** o **Tiplink**. Analogía: *"tu mamá usa WhatsApp sin saber qué es HTTP"*. Esto impacta directamente en la fricción de onboarding.

### Conseguir usuarios = vender

Frase repetida: *"Vender, hermano, vender, vender, vender"*. Conseguir usuarios en web3 es igual que en web2: hay que salir a buscarlos, no esperar a que lleguen. La validación con usuarios reales es lo que importa.

### MVP y no sobre-ingeniería

Insta a resolver el problema de hoy en 2-3 semanas con un MVP, y no arquitecturar a 3-5 años. Iterar con feedback real de usuarios.

### Seguridad

- Hacer **validaciones reales on-chain**, no solo en el cliente (el cliente puede ser manipulado).
- Considerar **auditoría** de los programas o al menos revisión con IA.
- Usar **multisig** (ej. Squads) para el deployment y gestión de tesorería.

### Cierre

Alex cierra reforzando: construir sobre Solana da velocidad y costo bajísimo de transacción, pero la responsabilidad de diseñar bien el modelo de confianza y la custodia es del founder. "Vender y validar" antes de escribir código innecesario.

---

## Notas de captura

- Audio obtenido desde YouTube (link estable arriba).
- Transcripción generada por herramienta de reconocimiento de voz en entorno local.
- Revisar `resumen.md` para la síntesis con implicaciones de producto.
