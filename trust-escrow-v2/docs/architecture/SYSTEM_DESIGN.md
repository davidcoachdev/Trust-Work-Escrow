# System Design - Trust Work Escrow v2

---

## 1. Arquitectura de Alto Nivel

```
┌──────────────────────────────────────────────────────────────────────────┐
│                              USERS                                       │
│  ┌──────────┐     ┌───────────┐     ┌──────────┐     ┌──────────┐        │
│  │ Clients  │     │Freelancers│     │ Arbiters │     │  Admin   │        │
│  └────┬─────┘     └─────┬─────┘     └────┬─────┘     └────┬─────┘        │
└───────┼─────────────────┼────────────────┼────────────────┼──────────────┘
        │                 │                │                │
        └─────────────────┴────────┬───────┴────────────────┘
                                   │
                    ┌──────────────┴──────────────┐
                    │         FRONTEND            │
                    │   Next.js 14 + Tailwind     │
                    │   Wallet Connect + Zustand  │
                    └──────────────┬──────────────┘
                                   │
                    ┌──────────────┴──────────────┐
                    │         BACKEND API         │
                    │   Rust Axum + SQLx          │
                    │   REST + WebSocket          │
                    └──────────────┬──────────────┘
                                   │
        ┌──────────────────────────┼────────────────────────┐
        │                          │                        │
┌───────┴───────┐          ┌───────┴───────┐        ┌───────┴───────┐
│    SDK        │          │   DATABASE    │        │   BLOCKCHAIN  │
│  escrow-core  │          │  PostgreSQL   │        │    Solana     │
│   (Rust)      │          │   MongoDB     │        │   + Anchor    │
│               │          │    Redis      │        │               │
└───────────────┘          └───────────────┘        └───────────────┘
```

---

## 2. Flujo de Datos

```
┌─────────────────────────────────────────────────────────────────────────┐
│                           FLOW: CREAR JOB                               │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  Client (Frontend)                                                      │
│       │                                                                 │
│       │ 1. POST /api/jobs { title, amount, ... }                        │
│       ▼                                                                 │
│  Backend API (Axum)                                                     │
│       │                                                                 │
│       │ 2. Validar request                                              │
│       ▼                                                                 │
│  PostgreSQL                                                             │
│       │ 3. Insert job (status: draft)                                   │
│       ▼                                                                 │
│  Backend API                                                            │
│       │ 4. Construir transacción via SDK                                │
│       ▼                                                                 │
│  SDK (escrow-core)                                                      │
│       │ 5. Crear instrucción `create_job`                               │
│       │ 6. Derivar PDA                                                  │
│       ▼                                                                 │
│  Solana RPC                                                             │
│       │ 7. Firmar y enviar transacción                                  │
│       ▼                                                                 │
│  Solana Blockchain                                                      │
│       │ 8. Ejecutar smart contract                                      │
│       │ 9. Emitir evento JobCreated                                     │
│       ▼                                                                 │
│  Helius Webhook ──────────────────────────────────────────────────────► │
│       │ 10. POST /api/webhooks/solana                                   │
│       ▼                                                                 │
│  Backend API                                                            │
│       │ 11. Actualizar job (status: created)                            │
│       ▼                                                                 │
│  PostgreSQL                                                             │
│       │ 12. Update job status                                           │
│       ▼                                                                 │
│  WebSocket Server                                                       │
│       │ 13. Emitir `job:created`                                        │
│       ▼                                                                 │
│  Frontend (Client)                                                      │
│       │ 14. Recibir evento                                              │
│       │ 15. Actualizar UI                                               │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

**Nota:** Este flujo también aplica desde CLI/TUI usando el SDK de Rust.

---

## 3. Flujo de Fondos

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         FLOW: PAGAR JOB                                 │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  Client (Frontend)                                                      │
│       │                                                                 │
│       │ 1. POST /api/jobs/:id/approve                                   │
│       ▼                                                                 │
│  Backend API                                                            │
│       │ 2. Validar: client == signer                                    │
│       │ 3. Validar: job.status == Submitted                             │
│       ▼                                                                 │
│  SDK (escrow-core)                                                      │
│       │ 4. Construir instrucción `approve_work`                         │
│       ▼                                                                 │
│  Solana RPC                                                             │
│       │ 5. Firmar y enviar                                              │
│       ▼                                                                 │
│  Smart Contract                                                         │
│       │ 6. Verificar: signer == client                                  │
│       │ 7. Calcular: freelancer_amount = amount - fee                   │
│       │ 8. Transfer: freelancer_amount → freelancer                     │
│       │ 9. Transfer: fee → treasury                                     │
│       │ 10. Emitir evento WorkApproved                                  │
│       ▼                                                                 │
│  Helius Webhook ──────────────────────────────────────────────────────► │
│       │ 11. Actualizar DB                                               │
│       ▼                                                                 │
│  Database                                                               │
│       │ 12. job.status = 'approved'                                     │
│       │ 13. Insert ledger entry                                         │
│       ▼                                                                 │
│  Notify Freelancer                                                      │
│       │ 14. Email / Push / In-app                                       │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

**Flujo de Disputas:**
1. Cualquiera (cliente o freelancer) puede abrir disputa
2. Ambas partes deben pagar 2.5% del monto del job (total 5%)
3. Sistema asignaógrafo aleatorio del pool
4. El stake (5%) se le paga al ÁRBITRO por su trabajo
5. Árbitro tiene 7 días para resolver
6. Admin puede extender 7 días más si el árbitro lo necesita
7. Si el árbitro no resuelve en 7 días: 5% de multa a tesorería + nuevo árbitro

---

## 4. Arquitectura de Seguridad

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        SEGURIDAD EN CAPAS                               │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  ┌──────────────────────────────────────────────────────────────────┐   │
│  │                        FRONTEND                                  │   │
│  │  • HTTPS only                                                    │   │
│  │  • CSP headers                                                   │   │
│  │  • XSS protection                                                │   │
│  │  • Wallet signature verification                                 │   │
│  └──────────────────────────────────────────────────────────────────┘   │
│                                  │                                      │
│  ┌──────────────────────────────────────────────────────────────────┐   │
│  │                        BACKEND                                   │   │
│  │  • JWT validation                                                │   │
│  │  • RBAC (Role-Based Access Control)                              │   │
│  │  • Rate limiting (per wallet + per IP)                           │   │
│  │  • Input validation (Zod)                                        │   │
│  │  • SQL injection prevention (SQLx)                               │   │
│  │  • CORS configuration                                            │   │
│  └──────────────────────────────────────────────────────────────────┘   │
│                                  │                                      │
│  ┌──────────────────────────────────────────────────────────────────┐   │
│  │                      SMART CONTRACT                              │   │
│  │  • No self-hiring (client != freelancer)                         │   │
│  │  • State machine (valid transitions only)                        │   │
│  │  • Atomic transactions                                           │   │
│  │  • Re-entrancy protection                                        │   │
│  │  • Authority checks                                              │   │
│  │  • Árbitro ≠ cliente Y árbitro ≠ freelancer                      │   │
│  └──────────────────────────────────────────────────────────────────┘   │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
---

## 5. Diagrama de Componentes

```
┌─────────────────────────────────────────────────────────────────────────┐
│                           FRONTEND (Next.js)                            │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐     │
│  │   Pages     │  │ Components  │  │    Hooks    │  │   Lib       │     │
│  │             │  │             │  │             │  │             │     │
│  │ • /jobs     │  │ • JobCard   │  │ • useJob    │  │ • sdk.ts    │     │
│  │ • /dashboard│  │ • Modal     │  │ • useWallet │  │ • api.ts    │     │
│  │ • /admin    │  │ • Button    │  │ • useAuth   │  │ • utils.ts  │     │
│  └─────────────┘  └─────────────┘  └─────────────┘  └─────────────┘     │
│                                                                         │
│  ┌─────────────────────────────────────────────────────────────────┐    │
│  │                        State Management                         │    │
│  │   Zustand Store                                                 │    │
│  │   ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐           │    │
│  │   │ user     │ │ jobs     │ │ notifs   │ │ theme    │           │    │
│  │   └──────────┘ └──────────┘ └──────────┘ └──────────┘           │    │
│  └─────────────────────────────────────────────────────────────────┘    │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                         BACKEND (Rust Axum)                             │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐     │
│  │  Routes     │  │  Services   │  │   Models    │  │ Middleware  │     │
│  │             │  │             │  │             │  │             │     │
│  │ • /jobs     │  │ • JobSvc    │  │ • Job       │  │ • Auth JWT  │     │
│  │ • /users    │  │ • UserSvc   │  │ • User      │  │ • RateLimit │     │
│  │ • /teams    │  │ • TeamSvc   │  │ • Team      │  │ • Logger    │     │
│  │ • /disputes │  │ • DisputeSvc│  │ • Dispute   │  │ • Tracing   │     │
│  └─────────────┘  └─────────────┘  └─────────────┘  └─────────────┘     │
│                                                                         │
│  ┌─────────────────────────────────────────────────────────────────┐    │
│  │                     External Integrations                       │    │
│  │   Helius RPC ──── Solana Blockchain ──── Squads Multisig        │    │
│  │   OpenAI API ──── AI Arbitration Engine                         │    │
│  │   SendGrid ─────── Email Notifications                          │    │
│  └─────────────────────────────────────────────────────────────────┘    │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
                                    │
                    ┌───────────────┴───────────────┐
                    ▼                               ▼
┌───────────────────────────┐       ┌───────────────────────────┐
│      PostgreSQL           │       │        MongoDB            │
│                           │       │                           │
│  • users                  │       │  • chat_messages          │
│  • jobs                   │       │  • audit_logs             │
│  • teams                  │       │  • ai_reports             │
│  • milestones             │       │                           │
│  • disputes               │       │                           │
│  • notifications          │       │                           │
│  • audit_logs             │       │                           │
│  • financial_ledger       │       │                           │
└───────────────────────────┘       └───────────────────────────┘
                    │
                    ▼
          ┌───────────────────┐
          │      Redis        │
          │                   │
          │  • Sessions       │
          │  • Cache          │
          │  • Pub/Sub        │
          └───────────────────┘
```

---

## 6. Flujo de Autenticación

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    FLOW: AUTENTICACIÓN WALLET                           │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  User (Browser)                                                         │
│       │                                                                 │
│       │ 1. Click "Connect Wallet"                                       │
│       ▼                                                                 │
│  Frontend (Wallet Connect)                                              │
│       │ 2. Mostrar modal de wallets                                     │
│       ▼                                                                 │
│  User selects Phantom                                                   │
│       │ 3. Connect request                                              │
│       ▼                                                                 │
│  Phantom Extension                                                      │
│       │ 4. Return wallet address                                        │
│       ▼                                                                 │
│  Frontend                                                               │
│       │ 5. POST /api/auth/verify { wallet }                             │
│       ▼                                                                 │
│  Backend                                                                │
│       │ 6. Generate nonce                                               │
│       │ 7. Save nonce in Redis (5 min TTL)                              │
│       │ 8. Return { message: "Sign: {nonce}" }                          │
│       ▼                                                                 │
│  Frontend                                                               │
│       │ 9. Prompt user to sign message                                  │
│       ▼                                                                 │
│  Phantom                                                                │
│       │ 10. Return signed message                                       │
│       ▼                                                                 │
│  Frontend                                                               │
│       │ 11. POST /api/auth/verify { wallet, signature }                 │
│       ▼                                                                 │
│  Backend                                                                │
│       │ 12. Verify signature                                            │
│       │ 13. Create/update user if needed                                │
│       │ 14. Generate JWT                                                │
│       │ 15. Return JWT + user data                                      │
│       ▼                                                                 │
│  Frontend                                                               │
│       │ 16. Store JWT in localStorage                                   │
│       │ 17. Update auth state                                           │
│       ▼                                                                 │
│  Dashboard loaded                                                       │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

---

_Last updated: 2026-03-22_
