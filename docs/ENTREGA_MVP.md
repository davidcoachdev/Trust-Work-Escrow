# Entrega MVP — Trust Work Escrow v3

**Fecha:** 2026-08-21  
**Programa:** Solana LATAM Hackathon — WayLearn 2026  
**Equipo:** Trust Work Escrow

---

## Links de entrega

| Entregable | Link |
|---|---|
| **Repositorio público GitHub** | https://github.com/davidcoachdev/Trust-Work-Escrow |
| **Branch/tag entrega** | `main` @ `d1eaafe` + tag `v3.0.0-audit` (7a2YhCd7iivXfyySkp1pf5jjijGqpjNqwQCUS912q5Vh) |
| **MVP en Devnet** | **Program:** `7a2YhCd7iivXfyySkp1pf5jjijGqpjNqwQCUS912q5Vh` <br> **Explorer:** https://explorer.solana.com/address/7a2YhCd7iivXfyySkp1pf5jjijGqpjNqwQCUS912q5Vh?cluster=devnet <br> **RPC:** `https://api.devnet.solana.com` <br> **Slot deploy:** `486034851` <br> **Firma deploy:** `6VNKeq2f2rn86etaVkncsWTPSpbeVSUn4YPR7EBiHvKP2ERYANFjFNCp8upp7XhqdKQSxwEoVjMNx715MxuoBbb` <br> **Frontend demo (Vercel):** **https://frontend-two-smoky-60.vercel.app** + Production `https://frontend-htkuid1sk-dcdevreps-projects.vercel.app` (Next 16, `NEXT_PUBLIC_RPC_URL=https://api.devnet.solana.com`, `NEXT_PUBLIC_PROGRAM_ID=7a2YhCd7...`) |
| **Video Demo 2min (1080p MP4 Drive)** | **Drive: pendiente** — _pendiente grabación_ → subir a `drive.google.com` y pegar link aquí (NO fake link) <br> **Contenido:** flujo `create_job` (publisher) → `apply` (freelancer) → `accept` → `submit_work` → `approve` + integración Solana (Phantom, `getHealth`, `program show` 7a2Y, Explorer) <br> **Estado:** ⏳ PENDIENTE |
| **Docs** | `docs/DEPLOYMENT_V3.md` (hashes c964..., c89a..., comandos reproducibles) + `docs/stack/README.md` (levantar todo) + `docs/BACKEND_COVERAGE.md` T26 + `TECH_DEBT_AUDIT.md` |
| **Otros** | `frontend` Next 16 + Zustand + `api/` por método (24/24) + dashboard freelancer/publisher 34/34, `backend` 31 endpoints 72/72, `landing` Dioxus 0.7.9 |

---

## Flujo principal para el video (2min)

**Publisher (Cliente):**
1. Conectar Phantom (devnet) → `POST /jobs` (title/description/amount/deadline) → `create_job` (3 args on-chain + off-chain metadata) → `GET /jobs` lista
2. Ver `applicants` (Vec 50) → `accept_application` (PDA `[b"application", job, index, applicant]`)

**Freelancer:**
1. Conectar wallet distinta → `POST /jobs/:id/apply` (proposal_hash 32) → `list_applications` (cursor opaco)
2. `submit_work` → `Submitted` + `countdown` + `autoApprove 7d`

**Cierre:**
- Publisher `approve_work` → `Released` + `Payout` (fee 2.5% a `arbitration_treasury`) → `History` métricas + export CSV
- Disputa opcional: `raise → submitEvidence (hash) → assignArbiter → resolve → finalize` (tab separado)

**Integración Solana a mostrar:**
- `solana program show 7a2YhCd7... --url https://api.devnet.solana.com` → `ProgramData 7Btj...` + `Explorer` link
- `curl getHealth` devnet → `{"result":"ok"}`
- `NEXT_PUBLIC_PROGRAM_ID=7a2YhCd7...` en footer/header

---

## Aviso Discord

> Copiar y pegar en canal privado WayLearn:
>
> Hola equipo WayLearn! Entrega MVP Trust Work Escrow v3 — Repo: https://github.com/davidcoachdev/Trust-Work-Escrow (tag v3.0.0-audit) — MVP Devnet: https://explorer.solana.com/address/7a2YhCd7iivXfyySkp1pf5jjijGqpjNqwQCUS912q5Vh?cluster=devnet — Frontend: https://frontend-two-smoky-60.vercel.app — Video 2min: [Drive link] — Docs: docs/ENTREGA_MVP.md — Gracias!

---

## Checklist entrega mañana

- [x] Repo público + tag
- [x] MVP devnet 7a2Y slot 486034851 sig 6VNKeq...
- [x] Frontend Vercel deploy `https://frontend-two-smoky-60.vercel.app` (Production `https://frontend-htkuid1sk-dcdevreps-projects.vercel.app`) con `NEXT_PUBLIC_RPC_URL=https://api.devnet.solana.com`
- [ ] Video 2min 1080p MP4 Drive (flujo + integración) — ⏳ PENDIENTE — Drive: pendiente (no fake completion)
- [x] Subir `docs/ENTREGA_MVP.md` con links finales
- [ ] Avisar en Discord privado — ⏳ PENDIENTE (no enviar hasta tener video link real)
