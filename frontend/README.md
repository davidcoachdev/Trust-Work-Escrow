# Trust Work Escrow — Frontend v3 (Next.js 16 dApp)

Frontend dApp para Trust Work Escrow v3. Landing original es **Dioxus** (ver `landing/`); este frontend es la **nueva dApp** en **Next.js 16**.

- Programa on-chain: `7a2YhCd7iivXfyySkp1pf5jjijGqpjNqwQCUS912q5Vh` (localnet, ver `trust-escrow-v3/Anchor.toml`)
- SDK Rust: `backend/sdk` (`trust-escrow-sdk`) — este frontend espeja su API en `src/lib/sdk.ts` (`list_jobs`, `create_job`, `apply` / `apply_to_job`)
- Wallet: `@solana/wallet-adapter-react` + `@solana/wallet-adapter-react-ui` (Phantom, Solflare)

## Páginas

- `/` — home + links
- `/jobs` — lista paginada (`sdk.list_jobs`)
- `/jobs/:id` — detalle + aplicar (`sdk.apply`)
- `/create` — crear job (`sdk.create_job`)

## Dev

```bash
cd frontend
npm install
npm run dev      # http://localhost:3000
npm run build    # verifica build
npm test         # vitest
```

Env: copiar `.env.example` → `.env.local` (RPC localnet por defecto).

## Indexado

- CodeGraph: `frontend/` bajo root `.codegraph`
- Repowise: `frontend/` incluido en índice
- Serena: `frontend/` registrado en `.serena/project.yml`
