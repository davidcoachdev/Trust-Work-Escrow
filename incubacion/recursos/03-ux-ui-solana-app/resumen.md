# 📋 Resumen — UX/UI Lessons from Building a Real Solana App

**Expositor:** Pauline Mila-Alonso (fundadora de 3 apps en Solana; 6 años en el ecosistema; construyó *WeSplit*, una app tipo Splitwise/Venmo para dividir gastos en stablecoins; trabajó con la Solana Foundation en proyectos de UX/UI)
**Duración:** ~59 min
**Link:** https://youtu.be/cHkVX7PcVXs
**Transcripción completa:** `./transcripcion.md` (inglés + traducción al español)

---

## Contexto de la charla

Pauline presenta **7 lecciones** aprendidas construyendo *WeSplit* (app para dividir gastos en stablecoins en Solana). Clave: el producto pasó por 3 versiones en ~6-8 meses, cada una con feedback de usuarios reales. La lección transversal: **validar con usuarios desde el día 1, no después de construir**.

---

## Las 7 lecciones + toolbox

### 1. Onboarding en los primeros 30 segundos
- Si en 30 segundos el usuario no entiende qué hace la app, la borra.
- **No pongas la conexión de wallet primero.** Mostrá el valor de la app, y conectá wallet/email/Google solo cuando lo necesite.
- Para usuarios no-crypto: ofrecé login con email o Google y **creá la wallet por ellos en el backend** (Privy, Phantom Connect). No saben que tienen wallet = adoptan igual.
- Onboarding corto (2-4 pantallas, no 7). Mostrá los pasos (progress indicator) para que no abandonen.

### 2. Esconder la blockchain
- Usuarios humanos, no robots: vocabulario web2. En vez de "send to [dirección]", usá nombres (`.sol` de Phantom). En vez de "approve transaction 0x...", usá "Send $50 to María".
- **Pagá vos los gas fees** (en Solana son baratos) → el usuario no se asusta. Mostrá "transaction fees" transparentes.
- Mostrar el monto como "+$50 enviados", no "-$50" (es dinero real que sale, no un débito críptico).

### 3. Loading y error states (crítico en web3)
- Nunca pantalla en negro mientras firma en la wallet: mostrá "waiting for signature".
- Transacciones pueden tardar segundos o minutos → pantalla de pending con feedback.
- **Error state nunca sea "transaction failed" genérico.** Decir: *"el pago no se envió, tu dinero está seguro en tu wallet"*. Gana confianza.
- Usar copywriting en momentos de espera para ganar trust.

### 4. Trust is a design job (la confianza se diseña)
- Paso de confirmación con recap: monto, destinatario, fees. Jerarquía visual: el monto es lo más grande.
- **Microcopy**: el texto dentro de la app debe explicar qué hace cada pantalla. Testear con usuarios porque "vos sabés qué hace, ellos no".
- El código puede ser perfecto, pero si la UX es mala, no usan la app.

### 5. No reinventar: adaptar lo que ya funciona
- Estudiar productos top (Phantom, Jupiter, Sanctum, Umbra) y adaptar, no copiar.
- Diferenciar **UI** (lo bello/branding) de **UX** (el flujo/cómo se usa). Una app puede tener mala UI pero buena UX y funcionar.

### 6. Testear con usuarios reales (no solo crypto Twitter)
- **Probá con tu mamá/papá** (no-crypto): ver cómo dudan en cada pantalla.
- En eventos/residencias: dales la app, **dales un goal** ("enviá una transacción", "agregá un amigo"), no les digas dónde está cada cosa, y mirá sus pain points.
- Twitter sirve para A/B testing de pantallas específicas y feedback de builders, pero **no reemplaza testeo con usuarios reales**.
- **No construyas 6 meses solo:** hacé 1-2 pantallas, testeá con 5 usuarios reales cada semana.

### 7. Design system (evitar "AI slop")
- Con IA generás rápido pero obtenés "slop" (todo idéntico/generizado).
- Creá un design system pequeño: colores de marca + **colores semánticos** (success/error/warning/info), tipografías, botones (estados: disabled/enabled/focus), spacing.
- Subir un moodboard/referencia a la IA → output personalizado, no genérico.
- Toolbox: Figma (+ MCP con Codex/Claude para iterar diseño↔código), Figma Community (templates), Framer, Iconify (iconos web3, funciona en mobile), Mobbin (referencia de pantallas reales), Pinterest (moodboard).

### Extra: accesibilidad
- Para usuarios con discapacidad visual/auditiva: Human Interface Guidelines de Apple; chequear contraste de color (lectores de pantalla dependen de jerarquía y semántica).

### Extra: negociar UX con ingeniería
- No shipear todo de una: identificar "quick wins" (cambios de UX que no rompen el backend).
- Explicar al engineer *por qué* (para que el usuario entienda y se quede), no solo "hacelo más lindo". Comentar/prototipar todos los estados (fail/success/error) antes de mandar.

---

## Implicaciones para Trust Work Escrow

- 💡 **Onboarding sin wallet primero**: en el MVP, mostrar qué es TWE (escrow de freelancers crypto-native) y conectar wallet/email solo cuando vaya a firmar. Esto impacta directo el factor de **adopción** que validamos en las entrevistas (`01` B3).
- 💡 **Esconder la blockchain / pagar gas**: el cliente menos tech no debería ver "approve transaction". Usar lenguaje claro ("Pagar $X a [freelancer]"). Relevante para la hipótesis de adopción no-crypto.
- 💡 **Error states de escrow**: si una transacción de liberación de fondos falla, decir "tu dinero sigue en el escrow" (no "tx failed"). Crítico para la **confianza** (sección A4 de `01`).
- 💡 **Testear con usuarios reales desde el MVP**: la lección #6 es literal nuestro Milestone 4. Usar "dar un goal" (simular crear un escrow, liberar pago) en las pruebas de uso del plan §6.
- 💡 **Design system para no parecer slop**: cuando construyamos el MVP, armar design system pequeño (colores semánticos: éxito al liberar, error al disputar).
- 💡 **Validación temprana**: no construir 6 meses → prototipo + 5 usuarios/semana. Esto es justo el espíritu del milestone.

---

## Usos en el proyecto

- `entregables/milestone-4-validacion-usuarios/trust-escrow-milestone-4-plan-validacion.md` → §6 (testing de prototipo/MVP): usar "dar un goal" y testear con no-crypto.
- `herramientas/01-preguntas-entrevista.md` → sección B3 (adopción/wallet) y A4 (confianza).
- `herramientas/03-recomendaciones-trabajo.md` → factor adopción.
