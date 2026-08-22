# 🎙️ Transcripción — UX/UI Lessons from Building a Real Solana App

**Expositor:** Pauline Mila-Alonso
**Link:** https://youtu.be/cHkVX7PcVXs
**Origen:** Sesiones del programa WayLearn & Solana LATAM Labs
**Idioma original:** Inglés | **Traducción:** Español (abajo de cada bloque)

---

## Transcripción (inglés + español)

0:02 [music]
0:10 — lessons from building a real app.
0:21 — Foundation Fore
1:03 Thank you for having me here today.
1:06 Um I will share my screen so it will be easier for you. Again, don't hesitate to ask question when I'm speaking in the chat or maybe I think you can speak. Um if it's easier in Spanish, don't hesitate. Put them in Spanish in the chat. Thanks.
1:23 Okay.
1:34 So just to give a bit of context about who I am before starting this presentation. My name is Paulen. I founded three different application on Solana. One was Dubzi — it was a sort of web flow for web three, building no code applications from scratch. The biggest one was WeSplit. WeSplit is the latest application I did. I've been building this for three to six months now. I went to Montendell, a residency in the United States where I presented WeSplit to different members of the foundation. I had lots of feedback from them and later worked directly with the foundation for different UX/UI projects. I've been on Solana for six years now building mainly decentralized applications, not just mobile but also web app. All the advices I'm going to share are good for mobile but also for web.

**ES:** Gracias por tenerme hoy. Voy a compartir mi pantalla para que sea más fácil. No duden en hacer preguntas en el chat; si es más fácil en español, háganlo en español. Soy Pauline. Fundé tres aplicaciones en Solana. Una fue Dubzi (un web flow no-code para web3). La más grande es WeSplit, que llevo construyendo 3-6 meses. Fui a Montendell (una residencia en EE.UU.) donde presenté WeSplit a la fundación y recibí feedback; luego trabajé con ellos en proyectos de UX/UI. Llevo 6 años en Solana construyendo dapps (mobile y web). Todos los consejos sirven para mobile y web.

3:06 So to give you context: it will be a one hour session, ask whenever you want. I'll share background about WeSplit, then the seven lessons I use every day building on Solana, a toolbox of tools, and Q&A.
3:38 So what is WeSplit? You know apps like Splitwise, Venmo, Cash App — web2 apps to split bills and send money easily, no bank info, just a phone number. The idea behind WeSplit: at blockchain events we had no way to split expenses in crypto. We used Phantom but it was a nightmare when someone didn't know how to send stablecoins. These are the three main screens. You can send/request money in stablecoins on Solana, create a group and split expenses in one tab. We worked with communities (monkey, nomu, NFD) so they got perks.

**ES:** Será una sesión de 1 hora. Contexto: WeSplit es como Splitwise/Venmo pero en crypto — para dividir gastos en stablecoins en Solana. En eventos web3 no podíamos dividir gastos en crypto; usábamos Phantom y era un dolor si alguien no sabía enviar stablecoins. Pantallas: enviar/recibir en stablecoins, grupo para dividir gastos. Trabajamos con comunidades para dar beneficios.

5:16 The visuals are not the first version. We had many versions. Six or eight months ago we were three people, had the idea, built the first version (left): a copycat of Splitwise but on web3/stablecoins. We pushed to community, feedback was: "why use this when I can get reimbursed in fiat with Splitwise/Venmo?" Our error: we built it and assumed users would come. You need to check if people actually want it. Talk with people → find new features.
6:29 The second version (center) added "Dutch" feature (one pays the whole table). People said "could be fun" but no one used it. The ideas people give ≠ what users want. Final version (right): groups with shared money, yield on top, pay for a trip together. Between versions: 2-3 months each, always talking to people, sharing on Twitter, going to events.

**ES:** Las pantallas no son la primera versión. Hicimos muchas. Al principio hicimos un clon de Splitwise en web3/stablecoins. Feedback: "¿por qué usar esto si ya me devuelven en fiat con Splitwise/Venmo?". Error: construir y esperar usuarios. Hay que validar que la gente lo quiere. La v2 añadió "uno paga la mesa" — la gente dijo "divertido" pero nadie lo usó. Ideas ≠ lo que usan. V3: grupos con dinero compartido + rendimiento, para pagar un viaje. Entre versiones: 2-3 meses, siempre hablando con usuarios.

9:10 **Lesson 1 — Onboarding.** Best product in the world: if in first 30 seconds user doesn't understand, they delete it. Those 30 seconds aren't just "connect wallet then you're in" — you need a flow of screens explaining what the app does. Many apps make you sign up/connect wallet/create account (5-10 min) before you even know what the app is. Don't put wallet connection first. Go straight into the app showing value; connect wallet only when needed. Remember: web3 has few people; your goal is web2 users too. They don't know wallets — provide email/Google connection and build the wallet for them in backend (Privy, Phantom Connect). They never know they have a wallet.

**ES:** Lección 1 — Onboarding. Si en 30 segundos no entienden, borran la app. No es solo "conectá wallet". Necesitás pantallas que expliquen qué hace. No pongas wallet primero: mostrá valor, conectá cuando haga falta. Web3 tiene poca gente; tu objetivo es también usuario web2. Ellos no saben de wallets: dales email/Google y creá la wallet por ellos (Privy, Phantom Connect). No saben que tienen wallet.

11:54 WeSplit onboarding: being a banking app we had to connect wallet first, but we're changing to show app then create wallet. You see: connect wallet / continue with email / Phantom Connect. Connect wallet forces using existing (Phantom, Solflare). For Seeker, provide native Seeker wallet or they won't use it. Email wallet straightforward. Phantom Connect (Google/Apple) creates wallet, reusable across apps with same email — best entry point to web3.

**ES:** Onboarding de WeSplit: siendo bancaria debimos conectar wallet primero, pero cambiamos a mostrar app y luego crear wallet. Botones: conectar wallet / email / Phantom Connect. Para Seeker, dar wallet nativa. Phantom Connect (Google/Apple) crea wallet reutilizable en otras apps — mejor entrada a web3.

13:25 After they understand value and create account, give first value: in WeSplit, create a group and split. Push them to first action so they come back. Phantom's 4-screen onboarding: only 2 with real action. Keep onboarding short (WeSplit went from 7 screens to 2).

**ES:** Tras entender valor y crear cuenta, dales el primer valor: en WeSplit, crear grupo y dividir. Empujalos a la primera acción. Onboarding de Phantom: 4 pantallas, solo 2 con acción real. WeSplit bajó de 7 a 2 pantallas.

15:34 **Lesson 2 — Hide the blockchain.** Users don't need to understand everything. Keep vocabulary everyone understands. Instead of "send to [address] + gas fees + signature", use web2 ways: send to a name (.sol of Phantom). Pay gas fees for users (WeSplit does) — in web2 there are transaction fees too, so "transaction fees" is easier. On Solana gas is cheap, you can sponsor it. Speak to humans, not robots: button "Send $50 to María" beats "approve transaction 0x...".

**ES:** Lección 2 — Esconder la blockchain. Usá vocabulario web2. En vez de dirección + gas + firma, enviá a un nombre (.sol). Pagá vos los gas (WeSplit lo hace; en web2 también hay fees, así que "transaction fees" se entiende). En Solana es barato sponsorear. Hablá como humano: "Enviar $50 a María" mejor que "aprobar tx 0x...".

18:28 Look at these two screens. Left (Solflare): "approve transaction minus 0.044 SOL, network fee..." — only a web3 user understands. Right: "I'm sending $50 to Pollin, network fees free, total $50" — human understands. Call to action "Send 50" not "Approve".

**ES:** Pantallas: izquierda (Solflare) solo la entiende un crypto-user. Derecha: "envío $50 a Pollin, fees gratis, total $50" — lo entiende cualquiera. CTA "Enviar 50", no "Aprobar".

20:28 **Lesson 3 — Loading & error states.** Not everything works all the time. Transactions take 5 sec or minutes or fail. User must know. Show pending screen; let them leave and notify when done. On failure, NEVER just "transaction failed" — say "payment error, but don't worry your money is safe in your wallet". Error text must be human ("app did not respond" means nothing). Use waiting moments to gain trust with transparent copy. Use red for errors, green for success. Always say e.g. "your $12 is safe in your wallet".

**ES:** Lección 3 — Loading y errores. No siempre funciona. Mostrá pantalla de pending; dejalos irse y avisales al terminar. En fallo, NUNCA "transaction failed" — decid "error de pago, tu dinero está seguro en tu wallet". Texto humano. Usá el momento de espera para ganar confianza. Rojo=error, verde=éxito. "Tus $12 están seguros en tu wallet".

26:15 **Lesson 4 — Trust is a design job.** Gain trust by designing for the user, not just beauty. Confirmation step with recap (amount, recipient, fees) prevents misclicks. Information hierarchy: amount is biggest (like all banking apps). Only ask needed info; extras optional. Microcopy: text inside app must explain each screen/action — test with users because "you know, they don't". Code can be perfect but bad UX = no adoption. Example: Solflare privacy feature with great microcopy and slide-to-confirm.

**ES:** Lección 4 — La confianza se diseña. Ganás confianza diseñando para el usuario. Paso de confirmación con recap (monto, destinatario, fees) evita errores. Jerarquía: el monto es lo más grande. Pedí solo lo necesario. Microcopy: el texto interno debe explicar; testear con usuarios. Código perfecto + mala UX = no adoptan. Ej.: privacidad de Solflare con buen microcopy y slide-to-confirm.

32:00 **Lesson 5 — Don't reinvent, adapt.** When I started 5 years ago I was lost. People before me had users and feedback. Study what works (Phantom, Jupiter, Sanctum, Umbra) and adapt, not copy. UI = beauty/branding; UX = flow/how it's used. An app can have bad UI but great UX, or vice versa.

**ES:** Lección 5 — No reinventes, adaptá. Estudiá lo que funciona (Phantom, Jupiter, Sanctum, Umbra) y adaptá. UI=estética; UX=flujo. Una app puede tener mala UI pero buena UX.

34:03 **Lesson 6 — Test with real users.** Posting on Twitter for feedback is nice for engagement but Twitter people aren't your users. Ask your mom/dad (non-crypto) to test — watch them hesitate. At events/residencies: give them the app, give a GOAL ("send a transaction", "add a friend"), don't show where things are, take notes on pain points. Never say "test my app" vaguely. Use Twitter for A/B of specific screens, not as real user testing. **Don't find idea → build → see later. Make 1-2 screens, test with 5 real users every week.** Don't waste 6 months building something people don't want.

**ES:** Lección 6 — Testeá con usuarios reales. Twitter sirve para engagement pero no son tus usuarios. Probá con tu mamá (no-crypto) y mirá dudar. En eventos: dales la app, dales un OBJETIVO ("enviá una tx", "agregá un amigo"), no les digas dónde está, anotá pain points. Usá Twitter para A/B de pantallas. NO hagas idea→construir→ver. Hacé 1-2 pantallas, testeá con 5 usuarios reales por semana. No pierdas 6 meses.

38:05 **Lesson 7 — Build a design system (avoid AI slop).** With AI you build fast but get "slop" (generic). Create a small system: colors, fonts, find inspiration (Pinterest, Mobbin). At least give AI a reference so output is personalized. A design system has brand colors + semantic colors (success/error/warning/info) + button states (disabled/enabled/focus) + spacing + tokens. Start small (5-10 min per component).

**ES:** Lección 7 — Hacé un design system (evitá el "AI slop"). Con IA vas rápido pero obtenés slop genérico. Creá sistema chico: colores, fuentes, inspiración (Pinterest, Mobbin). Dadle referencia a la IA. Un design system tiene colores de marca + semánticos (éxito/error/aviso/info) + estados de botón + spacing + tokens. Empezá chico.

41:32 **Toolbox:** Figma (connect Figma + Codex via MCP to iterate design↔code), Claude (best with Figma), Figma dev mode (copy CSS), Iconify (web3 icons, works mobile), Mobbin (real screen references), Pinterest (moodboard), Figma Community (copy templates). Learn UX by dissecting best web3 products (DeFi, wallets) — you can learn UX even without designer eye. Join local Superteam, residencies (Moondance, Castle, Island), Colosseum hackathons (best eyes on your project), build in public.

**ES:** Toolbox: Figma (+ MCP con Codex para iterar diseño↔código), Claude, Figma dev mode, Iconify (iconos web3 mobile), Mobbin (pantallas reales), Pinterest (moodboard), Figma Community. Aprendé UX diseccionando los mejores productos web3. Sumate a Superteam local, residencias, hackathons de Colosseum, build in public.

47:38 Pauline's flow: start in Figma with system design → ask Claude to build first version → ask Claude to match Figma → iterate between Figma and code. Best way to avoid slop and iterate fast.

**ES:** Flujo de Pauline: system design en Figma → Claude arma v1 → Claude iguala Figma → iterar Figma↔código. Evitás slop y iterás rápido.

49:35 Q&A starts.
- **Dev users app (Juan):** for devs as main users, look at Alchemy's e-learning platform for inspiration.
- **Accessibility (hearing/speech disabled):** Apple Human Interface Guidelines; check color contrast (some see colors differently); use clear hierarchy so screen readers work. Build in public for feedback.
- **PM: negotiate UX with engineering when it breaks backend:** don't ship everything at once — find "quick wins" (UX changes that don't break backend). Explain *why* (user understanding/retention), not just "make it prettier". Prototype all states (fail/success/error) and comment everything before sending to engineers.

**ES:** Preguntas: para app de devs, mirá plataforma de Alchemy. Accesibilidad: Human Interface Guidelines de Apple; contraste de color; jerarquía para lectores de pantalla. Negociar UX con ingeniería: quick wins, explicar el porqué, prototipar todos los estados.

59:04 Thanks Pauline. Session ends.

**ES:** Gracias Pauline. Fin de la sesión.

---

## Notas de captura

- Audio obtenido desde YouTube (link estable arriba).
- Transcripción original en inglés; traducción al español sincronizada por bloques (timestamps del video).
- Revisar `resumen.md` para la síntesis con implicaciones de producto.
