# app — Trust Work Escrow (Dioxus Fullstack)

> Nueva app Dioxus 0.7 fullstack (web + server Axum) — extiende `landing/` (dcdev #120808/#FF3C3C, i18n ES/EN) y integra `backend/sdk` + `trust-escrow-v3` en mismo workspace.
> `frontend/` queda como referencia rápida.

## Stack
- Dioxus 0.7 fullstack (web + router + server functions)
- Tailwind 4 (assets/tailwind.css)
- Theme dcdev/cyan/solana + Mode dark/light + i18n ES/EN (copiado de landing/src/theme,i18n)
- Server: Axum + sqlx (twe-postgres) + lettre (email OTP free) + jsonwebtoken + moka cache
- SDK: trust-escrow-sdk (devnet 7a2Y...)

## Estructura
```
app/src/
  route.rs — /, /login, /dashboard/client, /dashboard/freelancer, /dashboard/admin/*
  theme/ — dcdev tokens
  i18n/ — es.json/en.json
  server/{auth,db/{postgres,mongo,cache},webhooks,ws.rs}
  features/{auth,dashboard,escrow,arbitration,storage}
```

## Dev
```bash
dx serve --port 3001 --addr 0.0.0.0
# o
cargo run --features fullstack
```

## Issue
Epic #45 → Module #46 → Task #47 (email-otp) — rama `task/trust-work-escrow-auth-email-otp` → `feat/trust-work-escrow-auth`
