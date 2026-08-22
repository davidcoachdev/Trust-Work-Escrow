# Epic #1 — Trust Work Escrow para incubación Solana

> **Rama madre:** `feat/trust-work-escrow` → destino `dev`  
> **Stack:** `app/` Dioxus 0.7 fullstack (web + server Axum) + `backend/sdk` + `trust-escrow-v3` program `7a2YhCd7iivXfyySkp1pf5jjijGqpjNqwQCUS912q5Vh` (devnet)  
> **Referencia:** `frontend/` queda como pantalla rápida, `landing/` base dcdev/i18n  
> **Metodología:** `skill:trust-work-flow` + `.backup/planeacion de trabajo/` (Epic→Module→Task)  
> **Fecha:** 2026-08-21  
> **Objetivo incubación:** Producto completo, escrow 0.1 SOL ya probado (`JCR9...` 0.115 SOL), dashboards por rol, P2P arbitraje, todo en Render free, devnet hasta autorización mainnet.

## Visión

SaaS escrow descentralizado: `guest → email OTP (lettre/JWT free) → wallet link SIWS` → 7 roles (`guest, client, freelancer, arbiter, manager, treasurer, custodian, admin`). Fondos en PDA `Job` (program owner), solo programa mueve. P2P WebRTC para arbitraje, Cloudinary/R2 para archivos, webhooks push sin WS persistente.

## Estructura entregables

```
app/src/
  route.rs — /, /login, /dashboard/client, /dashboard/freelancer, /dashboard/arbiter, /dashboard/admin/*
  theme/ — dcdev/cyan/solana tokens (#120808, #FF3C3C)
  i18n/ — es.json/en.json (design/i18n)
  server/db/{postgres,mongo,cache} — moka cache
  server/webhooks/ — whatsapp placeholder
  server/ws.rs — broadcast
  features/{auth,dashboard,escrow,arbitration,storage}
openspec/changes/<module>/spec.md
.github/ISSUE_TEMPLATE/epic.md
```

## Módulos y Tasks

### Module A — Auth completo (free, sin pagar) — `feat/trust-work-escrow/auth`
**Goal:** `guest → email OTP → wallet link` con dcdev + i18n desde día 1.
- [ ] **Task A1 — `task/trust-work-escrow/auth/email-otp`** — `lettre` + `jsonwebtoken` OTP 6 dígitos hasheado en `users, verificationTokens`, SMTP Gmail free, `POST /auth/email`. **AC:** OTP expira 10m, rate-limit, no log secretos.
- [ ] **Task A2 — `task/trust-work-escrow/auth/siws-link`** — `ed25519` verify, `users.wallet_pubkey`, `POST /auth/link-wallet`, `guest→client/freelancer` elección. **AC:** firma válida cambia role, inválida 401.
- [ ] **Task A3 — `task/trust-work-escrow/auth/middleware-i18n-theme`** — `middleware.ts` Dioxus, `Theme`/`Lang` de `landing/src/theme,i18n`, `next-themes` portado a `apply_theme`/`apply_lang`, header switchers. **AC:** persiste `twe-theme`/`twe-lang`, fallback ES.

### Module B — Dashboards por rol — `feat/trust-work-escrow/dashboard`
- [ ] **Task B1 — `task/trust-work-escrow/dashboard/layouts`** — Route Groups `app/(client)`, `(freelancer)`, `(arbiter)`, `(admin)` con `Sidebar` filtrado. **AC:** `client` no ve `/admin`.
- [ ] **Task B2 — `task/trust-work-escrow/dashboard/client`** — crear job 0.1 SOL, ver `JCR9...`, accept freelancer. **AC:** `POST /jobs` con `amount` real (fix bug `1_000_000` hardcode).
- [ ] **Task B3 — `task/trust-work-escrow/dashboard/freelancer`** — listar jobs, `apply_to_job` on-chain `B5Ks...`. **AC:** `QWgp...` aparece en PDA.
- [ ] **Task B4 — `task/trust-work-escrow/dashboard/arbiter-admin`** — manager/treasurer/custodian vistas (métricas, treasury 6KSy/CfkG).

### Module C — Escrow core on-chain — `feat/trust-work-escrow/escrow`
- [ ] **Task C1 — `task/trust-work-escrow/escrow/milestones`** — `create_milestone`/`submit`/`approve` UI + PDAs `milestone`.
- [ ] **Task C2 — `task/trust-work-escrow/escrow/disputes`** — `raise_dispute`/`submit_evidence` con hash on-chain + archivo Cloudinary.
- [ ] **Task C3 — `task/trust-work-escrow/escrow/release`** — `approve_work` libera a freelancer, `cancel_job` devuelve.

### Module D — Arbitraje P2P + tiempo real — `feat/trust-work-escrow/arbitration`
- [ ] **Task D1 — `task/trust-work-escrow/arbitration/webrtc-p2p`** — `RtcPeerConnection` + `DataChannel` chat/archivos, signaling vía `ws` 2s.
- [ ] **Task D2 — `task/trust-work-escrow/arbitration/ws-push`** — `axum::ws` `broadcast` + `POST /api/webhooks/*` para push sin polling 15s (ahorra Render free).
- [ ] **Task D3 — `task/trust-work-escrow/arbitration/screens`** — pantalla `who`/`how` + sala arbitraje con chat, llamada, envío archivo.

### Module E — Storage + cache — `feat/trust-work-escrow/storage`
- [ ] **Task E1 — `task/trust-work-escrow/storage/cloudinary-r2`** — `Cloudinary` imágenes 25GB free + `R2` 10GB free para docs, hash en `Evidence`.
- [ ] **Task E2 — `task/trust-work-escrow/storage/cache-mongo`** — `moka` cache `get_job`, `Mongo` evidence/logs, Atlas M0 free.

### Module F — Deploy y tracción incubación — `feat/trust-work-escrow/deploy`
- [ ] **Task F1 — `task/trust-work-escrow/deploy/render`** — `Dockerfile` Rust para `app` en Render, `PORT=0.0.0.0`, Postgres/Mongo en Render.
- [ ] **Task F2 — `task/trust-work-escrow/deploy/whatsapp-reserva`** — placeholder `app/api/webhooks/whatsapp` y tabla `messages`, feature flag `whatsapp` apagado.

## Dependencias

- B1 ← A (auth)
- C ← B (dashboards) + SDK
- D ← C (escrow)
- E ← C, D
- F ← todo

## Seguridad y performance

- Sin secretos en git, `chmod 600` keypairs, `INITIAL_AUTHORITY` solo admin, treasuries `6KSy.../CfkG...` distintas.
- `clippy` + `cargo test` verde por Module, `Red Podvisor` review por PR.
- `moka` evita `GET /jobs` cada 15s; `ws` solo en sala.

## Criterios incubación

- Demo devnet con `JCR9...` 0.115 SOL y `B5Ks...` aplicación verificables en explorer.
- Video 2min + `docs/ARQUITECTURA.md` + `context/kits` trazable.

## Siguiente

- Crear Issue Epic #1 con esta plantilla, luego `02-module.md` por cada Module, worktree `feat/trust-work-escrow` + `task/...` por Task.

