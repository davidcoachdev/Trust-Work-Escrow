# Preguntas Pendientes por Resolver
## Trust Work Escrow v2

> Este archivo contiene todas las preguntas que necesitan respuesta antes de proceder con la implementación. Completa cada sección y guarda este archivo.

---

## 1. Modelo de Negocio

### 1.1 Comisiones
- [x] **¿El 5% de entrada (cliente) se cobra al momento de publicar o al fondear?**
  
  Ejemplo: Si el job es de 100 SOL:
  - Opción A: Cliente paga 105 SOL al publicar (5% fee incluido)
  - Opción B: Cliente paga 100 SOL, pero se le cobra 5 SOL adicional al finalizar
  
  **Tu decisión:** opcion A

### 1.2 Moneda
- [x] **¿Los jobs serán en SOL, USDC, o ambos?**
  
  - [x] Solo SOL (más simple, menos costos)
  - [ ] Solo USDC (más estable para freelancers)
  - [ ] Ambos (más complejo, pero más flexible)
  
  **Tu decisión:** la mas simple aumentar un archivo md con las funcionalidades futuras y agragar la integracion de lo usdc

---

## 2. Sistema de Equipos

### 2.1 Estructura de Equipos
- [x] **¿Los equipos son on-chain (en el smart contract) o solo off-chain (en la DB)?**
  
  - [ ] On-chain (más seguro, más costo de gas)
  - [ ] Off-chain (más flexible, menos seguro)
  - [x] Híbrido (configuración off-chain, pagos on-chain)
  
  **Tu decisión:** creo que es la mas rentable pero si me equivoco diemlo

### 2.2 Límites de Equipos
- [x] **¿Cuál es el número máximo de miembros por equipo?** (建议: 10-20)
  
  **Tu decisión:** esta bien de 10 a 20 

- [ ] **¿Se permiten equipos dentro de equipos (sub-equipos)?**
  
  - [ ] Sí
  - [x] No (más simple)
  
  **Tu decisión:** no eso ya es muy dificil para esta fase agrgarlo al md de futuras funcionalidades

---

## 3. Sistema de Disputas

### 3.1 Stake de Disputa
- [x] **¿El stake de disputa (5%) quién lo paga?**
  
  - [ ] Solo quien abre la disputa
  - [x] Ambos (cliente y freelancer pagan 2.5% cada uno - total 5%)
  
  **Tu decisión:** este valor se le acredita al árbitro cuando termine su trabajo de disputa. Si no lo hace en el tiempo dispuesto (7 días), se le da una multa del 5% por incumplimiento y se asigna otro árbitro. El admin puede darle 7 días más al árbitro si lo necesita.

### 3.2 Asignación de Árbitros
- [x] **¿Cómo se asigna el árbitro?**
  
  - [x] Aleatorio del pool de árbitros
  - [ ] El cliente elige al crear el job
  - [ ] Sistema asigna según especialización
  - [ ] Los primeros N árbitros disponibles
  
  **Tu decisión:** el sitema lo asigma utomatica mente por sorteo despues del cobre de la tasa por abrir consulta 

### 3.3 Pool de Árbitros
- [x] **¿Cómo se convierte uno en árbitro?**
  
  - [x] Admin registra manualmente
  - [ ] Postulación + verificación de reputación
  - [x] Require X trabajos completados + Y reputation score
  
  **Tu decisión:** las dos porque lo primeros van a ser personal de la app despues si va a ser por la opcuion tres

### 3.4 Límites de Tiempo
- [x] **¿Cuánto tiempo tiene el árbitro para resolver una disputa?**
  
  - [ ] 24 horas
  - [ ] 48 horas
  - [ ] 72 horas
  - [x] 7 días
  - [ ] Sin límite
  
  **Tu decisión:** 7 dias si no se le pondra una multa del 5% de jod esto implica que el abrito se le conjelara este valor y se le vuelve despues de terminar la disputa que te parce la idea y tambien estaba pensado que la comicion sea para el arbitro por el trabajo para que lo haga con jana porque sino quien quera ser arbito eso creo que es lo gusto y ahorra cambiamos lo de comicion se le va descontar a las dos partes sin devolucion 
  el tiempo tambie los sdmin le puede dar mas tiempo si el caso lo requiere pilas

---

## 4. Funcionalidades

### 4.1 Hitos (Milestones)
- [x] **¿Implementamos hitos en v1 o v2?**
  
  - [ ] v1 (prioridad hackathon)
  - [x] v2 (post-hackathon)
  
  **Tu decisión:** aumentalo al md de futuras funcinalidades

### 4.2 Chat E2EE
- [x] **¿El chat es obligatorio o opcional?**
  
  - [x] Obligatorio (todos los jobs tienen chat)
  - [ ] Opcional (habilitado por el cliente)
  - [ ] No hay chat (solo notificaciones)
  
  **Tu decisión:** debe de tener un medio de comunicacion denttro de la app para poder usar de base para usar como pruebas en las disputas

### 4.3 Categorías
- [x] **¿Qué categorías de trabajo we'll support?**
  
  - [ ] Solo desarrollo (web, mobile, blockchain)
  - [ ] Desarrollo + diseño
  - [ ] Todas las categorías (desarrollo, diseño, marketing, etc.)
  
  **Tu decisión:** todo es abierto en la descripcion se detallar lo que pide y esa parte donde se va a guarda en solana me va a dcostar pero en solana puede ir el nombre o titulo de el jod y en db los detalles asi me parce mejor

---

## 5. Frontend

### 5.1 Tipos de Usuario
- [x] **¿La aplicación será web, desktop, o ambas?**
  
  - [x] Solo Web (Next.js deployado en Vercel)
  - [ ] Web + Desktop (Electron o Tauri)
  - [ ] Solo Desktop (como extensión de navegador)
  
  **Tu decisión:** solo web con pda como lo ahce youtuve

### 5.2 Wallet Connect
- [x] **¿Qué wallets we'll support oficialmente?**
  
  - [x] Phantom (prioridad)
  - [ ] Phantom + Solflare
  - [ ] Phantom + Solflare + Backpack
  - [ ] Todas las wallets compatibles con Wallet Connect
  
  **Tu decisión:** las demas conecciones agregala al md de futuras fucionalidades

### 5.3 Onboarding
- [x] **¿El usuario necesita completar un perfil antes de usar la plataforma?**
  
  - [ ] No, solo wallet (KYC Cripto-Light)
  - [x] Sí, username + bio mínimo
  - [ ] Sí, verificación de email
  - [ ] Sí, verificación de identidad (documento)
  
  **Tu decisión:**  para el hakaton solo usermen y datos minimos agraga al md de funcinalidae futuras el resto 

---

## 6. Infraestructura

### 6.1 Backend
- [x] **¿Qué tecnología para el backend?**
  
  - [x] Rust + Axum (unificado con SDK)
  - [ ] Node.js + Express/Fastify
  - [ ] Python + FastAPI
  - [ ] Go + Fiber/Echo
  
  **Tu decisión:** solo rust y sus librerias despues vemos si ampliamos a go

### 6.2 Base de Datos
- [x] **¿Qué base de datos principal?**
  
  - [ ] PostgreSQL (via Supabase o Railway)
  - [x] PostgreSQL + MongoDB
  - [ ] Solo MongoDB
  
  **Tu decisión:** y redis para cache

### 6.3 Hosting
- [x] **¿Dónde deployamos?**
  
  | Componente | Opción preferida |
  |------------|------------------|
  | Frontend | Vercel / Netlify / Cloudflare Pages |
  | Backend | Railway / Render / Fly.io |
  | DB | Supabase / Neon / Railway |
  
  **Tu decisión:** front en vercel -> back en render -> recuerda  la db son varias momgo en momgo altas, posgersql en render o supabase y redis no se donde

### 6.4 Helius
- [x] **¿Usamos Helius para webhooks y RPC?**
  
  - [ ] Sí, Helius completo (webhooks + RPC premium)
  - [x] Sí, solo RPC (webhooks propios)
  - [ ] No, RPC público de Solana
  
  **Tu decisión:** esto esta por verse porque no se si hay capa gratuiota porque si no toca ver como se lo codea en rust

---

## 7. IA para Arbitraje

### 7.1 Integración de IA
- [x] **¿La IA será real o mock en v1?**
  
  - [x] Mock (para demo del hackathon)
  - [ ] Real (usando API de Gemini/OpenAI)
  
  **Tu decisión:** agrega la real en el md de futuras funcionalidades

### 7.2 Modelo de IA
- [x] **¿Qué modelo de IA usar?**
  
  - [x] Gemini (gratis, límite generoso)
  - [ ] GPT-4o (más caro, mejor reasoning)
  - [ ] Claude (buen balance)
  
  **Tu decisión:** me parce el mejor para la hakaton

---

## 8. Seguridad

### 8.1 Treasury Multisig
- [x] **¿Implementamos multisig para treasury?**
  
  - [x] Sí, Squads Protocol
  - [ ] Sí, custom multisig en el contract
  - [ ] No, wallet simple del admin (para el hackathon)
  
  **Tu decisión:** si pero si da problema solo admin para la hakaton y la pondresmo en el ms de futuras funcionalidaes 

### 8.2 Rate Limiting
- [x] **¿Implementamos rate limiting?**
  
  - [x] Sí, por wallet (en backend)
  - [x] Sí, por IP
  - [ ] No (para el hackathon)
  
  **Tu decisión:** la seguridad es lo primero pero si se hace dificil lo pasamos al md de fuituras implementaciones

---

## 9. Multi-Wallet

### 9.1 Límites
- [x] **¿Cuántas wallets máximo por usuario?**
  
  **Tu decisión:** 5

### 9.2 Vinculación
- [x] **¿Cómo se verifica que el usuario posee la wallet secundaria?**
  
  - [x] Sign-message (recomendado)
  - [ ] Transferencia mínima de verificación
  - [ ] Solo admin puede agregar wallets
  
  **Tu decisión:** la marcada

---

## 10. Otros

### 10.1 Notificaciones
- [x] **¿Sistema de notificaciones?**
  
  - [x] In-app (base)
  - [ ] In-app + Email
  - [ ] In-app + Email + Push
  
  **Tu decisión:** y si se alcanza notificacion psuh la de emial agregala al md de futura funcionalidades porque eso debe verificar el email y eso tambien es una futura funcionalidad 

### 10.2 Internacionalización
- [x] **¿Qué idiomas we'll support?**
  
  - [ ] Solo inglés
  - [x] Inglés + Español
  - [ ] Multi-idioma
  
  **Tu decisión:** el multi idioma agregalo a md de futura actualizaciones

---

## 📝 Notas Adicionales

_Agrega aquí cualquier decisión adicional o contexto importante:_

_______________________________________________________________________________

________________________________________________________________________________

________________________________________________________________________________

---

## ✅ Checklist de Completado

Una vez respondidas todas las preguntas, marca aquí:

- [x] Todas las preguntas respondidas
- [x] Decisiones documentadas
- [x] Prioridades clarificadas
- [x] Listo para iniciar implementación

---

_Last updated: 2026-03-22_
