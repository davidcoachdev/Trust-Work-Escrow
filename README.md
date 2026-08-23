# Trust Work Escrow v3 🛡️

> **Protocolo de escrow descentralizado en Solana para freelancers y clientes. Fuente de verdad: `trust-escrow-v3`. Construido para el WayLearn Solana Hackathon.**

[![Solana](https://img.shields.io/badge/Solana-2.x-9945FF?logo=solana&logoColor=white)](https://solana.com)
[![Anchor](https://img.shields.io/badge/Anchor-0.32-blue)](https://www.anchor-lang.com)
[![Rust](https://img.shields.io/badge/Rust-1.89-orange?logo=rust)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](./LICENSE)
[![WayLearn](https://img.shields.io/badge/WayLearn-Hackathon-FF6B6B?logo=rocket)](https://dorahacks.io/hackathon/solana-waylearn-2026/detail)

---

## 🎯 Qué Es

**Trust Work Escrow v3** es un protocolo on-chain que permite pagos seguros entre clientes y freelancers (individuales o equipos) sin intermediarios. **v3 es la fuente de verdad** (40 instrucciones, arquitectura split, `trust-escrow-v2` queda como legacy).

```
Cliente deposita fondos → Freelancer entrega trabajo → Cliente aprueba → Pago automático
                                          ↓
                              Si hay desacuerdo → Árbitro resuelve
```

| | Escrow Tradicional | Trust Work Escrow v3 |
|--|---|---|
| **Centralización** | Banco/tercero | Código = confianza |
| **Comisiones** | 3-10% | 2.5% por parte en disputas, configurable |
| **Velocidad** | Días/semanas | Instantáneo en Solana |
| **Identidad** | Requiere KYC | Solo wallet |
| **Disputas** | Decisión unilateral | Arbitraje con pool + platform case |

---

## ⚡ Características v3

### Smart Contract (Anchor 0.32 / Rust 1.89)
- **40 instrucciones** (v2: 31) — config, job, dispute, milestone, treasury, authority
- **Arquitectura split** — `lib.rs` delega a `instructions/{config,job,dispute,milestone}` + `state/*` (no monolítico `1485 LOC`)
- **Vec50** — `MAX_APPLICATIONS = 50` por job (v2: sin límite tipado), `MAX_MILESTONES = 20`, paginación `RemainingAccounts` (10 por tx)
- **Timelock** — `propose_authority` → `update_authority` con ventana `AUTO_APPROVAL_DELAY = 604800` (7d), `cancel_authority_proposal`
- **RemainingAccounts typed** — `RemainingAccounts { metas: Vec<AccountMetaBorsh> }` borsh-serializado, evita `Vec<Pubkey>` inline
- **Otros**: `auto_approve_work` (7d), `pause_job`/`expire_paused_job`, `open_support_ticket`/`resolve_platform_case`, fee a `arbitration_treasury` separado

### Backend & App
- **Backend** (`backend/api`): 31 endpoints, Postgres + Mongo, health `/health`
- **App** (`app/`): Dioxus 0.7 fullstack (web + Axum server), Tailwind 4, i18n ES/EN, integra `backend/sdk` + `trust-escrow-v3` (devnet `7a2YhCd7...`)

---

## 🏗️ Arquitectura

```
┌─────────────────────────────────────────────────────────┐
│                    APP (Dioxus 0.7)                      │
│         Fullstack + SDK + Wallet (devnet 7a2Y)          │
└─────────────────────┬───────────────────────────────────┘
                      │
┌─────────────────────┴───────────────────────────────────┐
│                  SMART CONTRACT v3                      │
│            Anchor 0.32 + Rust 1.89 + Solana             │
│                                                         │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌────────┐  │
│  │  Config  │  │   Job    │  │ Dispute  │  │Milestone│ │
│  │(timelock)│  │ (Vec50)  │  │(Remaining│  │        │  │
│  └──────────┘  └──────────┘  │ Accounts)│  └────────┘  │
│                              └──────────┘               │
└─────────────────────────────────────────────────────────┘
```

---

## 📦 Estructura del Repo

```
Trust-Work-Escrow/
├── trust-escrow-v3/         # v3 — FUENTE DE VERDAD
│   ├── programs/trust-escrow-v3/src/
│   │   ├── lib.rs                   # 40 ix, delega a instructions/*
│   │   ├── instructions/{config,job,dispute,milestone}.rs
│   │   └── state/{config,job,dispute,milestone,application,evidencia}.rs
│   ├── Anchor.toml / Cargo.toml / rust-toolchain.toml (1.89.0)
│   └── tests/
├── app/                     # Dioxus 0.7 fullstack (ver app/README.md)
├── backend/api/             # API Rust (Dockerfile rust:1.89)
├── trust-escrow-v2/         # v2 legacy (31 ix, monolítico lib.rs 1485 LOC) — no usar para nuevo dev
└── docs/                    # ENTREGA_MVP.md, DEPLOYMENT_V3.md, etc.
```

> **Nota v2 vs v3:** v2 era monolítico por bug Anchor #3690. v3 ya está split en módulos y usa `RemainingAccounts` tipado + timelock.

---

## 🚀 Cómo Construir

### Prerrequisitos
- Rust 1.89+ (`rust-toolchain.toml` pinneado)
- Solana CLI 2.x
- Anchor 0.32+
- Node.js 20 / Yarn

### Smart Contract (v3)

```bash
cd trust-escrow-v3

# Compilar
anchor build

# Ver IDL
cat target/idl/trust_escrow_v3.json | jq '.instructions | length'
# → 40

# Tests
anchor test
```

### App (Dioxus)

```bash
cd app
dx serve --port 3001 --addr 0.0.0.0
# o
cargo run --features fullstack
```

### Backend

```bash
docker compose up --build # api + postgres + mongo (healthchecks + twe-net)
```

---

## 📊 Progreso del Hackathon

**Deadline:** 23 de marzo de 2026, 23:30 UTC

| Fase | Descripción | Estado |
|------|-------------|--------|
| 01-setup | Config, timelock, treasury split | ✅ Completada |
| 02-jobs-teams | Jobs Vec50, RemainingAccounts, milestone | ✅ Completada |
| 03-disputes | Disputes, evidence, platform case | ✅ Completada |
| 04-tests-idl | Tests, IDL 40 ix, docs v3 | ✅ Completada |

### Instrucciones v3 (40)

| Módulo | # | Instrucciones |
|--------|---|---------------|
| Config/Authority | 9 | `initialize_config`, `pause`, `unpause`, `update_treasury`, `update_arbitration_treasury`, `withdraw_treasury`, `withdraw_arbitration`, `propose_authority`, `update_authority`, `cancel_authority_proposal`, `create_arbiter_pool`, `add_arbiter`, `remove_arbiter` |
| Job | 14 | `create_job`, `deposit_funds`, `apply_to_job`, `accept_application`, `reject_application`, `withdraw_application`, `cleanup_applications`, `submit_work`, `auto_approve_work`, `approve_work`, `reject_work`, `cancel_job`, `pause_job`, `unpause_job`, `expire_paused_job` |
| Dispute/Support | 9 | `raise_dispute`, `accept_dispute`, `submit_evidence`, `assign_arbiter`, `resolve_dispute`, `resolve_platform_case`, `request_platform_intervention`, `open_support_ticket`, `resolve_support_ticket` |
| Milestone | 4 | `create_milestone`, `submit_milestone`, `approve_milestone`, `reject_milestone` |

---

## 📖 Documentación

| Documento | Descripción |
|-----------|-------------|
| [DEPLOYMENT_V3.md](./docs/DEPLOYMENT_V3.md) | Deploy reproducibile + hashes |
| [ENTREGA_MVP.md](./docs/ENTREGA_MVP.md) | Entrega hackathon (video pendiente) |
| [trust-escrow-v3/docs/](./trust-escrow-v3/docs/) | Specs contrato v3 |
| [app/README.md](./app/README.md) | App Dioxus |

---

**Construido para el [WayLearn Solana Hackathon 2026](https://dorahacks.io/hackathon/solana-waylearn-2026/detail)**
