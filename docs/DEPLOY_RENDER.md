# Deploy Render — Trust Work Escrow (devnet)

Todo en Render free, sin depender de tu PC.

## Servicios
- `app` (Dioxus fullstack Rust) — Web Service, `PORT=0.0.0.0:10000`, `Dockerfile` Rust
- `twe-postgres` — Render PostgreSQL free
- `twe-mongo` — Mongo Atlas M0 free (o Render Private Service `mongo:7`)

## Env
```
SOLANA_RPC_URL=https://api.devnet.solana.com
DATABASE_URL=postgres://...
MONGO_URL=mongodb+srv://...
SMTP_HOST=smtp.gmail.com
```

## Webhooks/WS
- `POST /api/webhooks/whatsapp` — placeholder, feature flag `whatsapp` apagado hasta Fase F2
- `ws://app.onrender.com/ws` — solo dentro de sala arbitraje, no 24/7

## Verificación
- `https://app.onrender.com/health` → `{"rpc":"ok"}`
- Explorer: JCR9... 0.115 SOL
