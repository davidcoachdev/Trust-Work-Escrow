# Trust Work Escrow v2 🛡️

> **Protocolo de escrow descentralizado en Solana para freelancers y clientes. Construido para el WayLearn Solana Hackathon.**

[![Solana](https://img.shields.io/badge/Solana-2.x-9945FF?logo=solana&logoColor=white)](https://solana.com)
[![Anchor](https://img.shields.io/badge/Anchor-0.32-blue)](https://www.anchor-lang.com)
[![Rust](https://img.shields.io/badge/Rust-1.89-orange?logo=rust)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](./LICENSE)
[![WayLearn](https://img.shields.io/badge/WayLearn-Hackathon-FF6B6B?logo=rocket)](https://dorahacks.io/hackathon/solana-waylearn-2026/detail)

---

## 🎯 Qué Es

**Trust Work Escrow v2** es un protocolo on-chain que permite pagos seguros entre clientes y freelancers (individuales o equipos) sin intermediarios.

```
Cliente deposita fondos → Freelancer entrega trabajo → Cliente aprueba → Pago automático
                                          ↓
                              Si hay desacuerdo → Árbitro resuelve
```

| | Escrow Tradicional | Trust Work Escrow v2 |
|--|-------------------|---------------------|
| **Centralización** | Banco/tercero | Código = confianza |
| **Comisiones** | 3-10% | Configurable |
| **Velocidad** | Días/semanas | Instantáneo en Solana |
| **Identidad** | Requiere KYC | Solo wallet |
| **Disputas** | Decisión unilateral | Arbitraje con % configurable |

---

## ⚡ Características Principales

### Smart Contract (Anchor/Rust)
- **Multi-wallet**: hasta 5 wallets por usuario
- **Equipos**: freelancers en grupo (Owner, PM, Contributors)
- **Arbitraje**: pool de árbitros, resolución con % configurable
- **Milestones**: pagos por hitos
- **Pausable**: admin puede pausar el programa en emergencias

---

## 🏗️ Arquitectura

```
┌─────────────────────────────────────────────────────────┐
│                         FRONTEND                        │
│              Next.js + Wallet Connect                   │
│               (planes post-hackathon)                   │
└─────────────────────┬───────────────────────────────────┘
                      │
┌─────────────────────┴───────────────────────────────────┐
│                    SMART CONTRACT                       │
│              Anchor 0.32 + Rust + Solana               │
│                                                         │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌────────┐  │
│  │  Config  │  │   User   │  │   Team   │  │  Job   │  │
│  └──────────┘  └──────────┘  └──────────┘  └────────┘  │
│                                                         │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌────────┐  │
│  │   Work   │  │ Dispute  │  │Milestone │  │Treasury│  │
│  └──────────┘  └──────────┘  └──────────┘  └────────┘  │
└─────────────────────────────────────────────────────────┘
```

---

## 📦 Estructura del Repo

```
Trust-Work-Escrow/
├── trust-escrow/              # v1 (bootcamp) — CLI, TUI, escrow-core
│   └── README.md             # Documentación legacy
│
├── trust-escrow-v2/          # v2 (hackathon) — Smart contract
│   ├── programs/
│   │   └── trust-escrow-v2/ # Smart contract Anchor
│   │       └── src/
│   │           └── lib.rs            # TODO el contrato (monolítico)
│   ├── Anchor.toml
│   ├── Cargo.toml
│   └── AGENDA.md             # Roadmap de desarrollo
│
└── README.md                 # Este archivo
```

**⚠️ Nota:** El smart contract está en un solo archivo `lib.rs` debido a un bug conocido en Anchor 0.32.x con módulos anidados (`#[program]` macro issue #3690).

---

## 🚀 Cómo Construir

### Prerrequisitos

- Rust 1.89+
- Solana CLI 2.x
- Anchor 0.32+
- Node.js 20 LTS

### Smart Contract (v2)

```bash
cd trust-escrow-v2/programs/trust-escrow-v2

# Instalar dependencias
cargo fetch

# Compilar
cargo build

# Ver IDL (después de anchor build)
cat target/idl/trust_escrow_v2.json | jq '.instructions | length'
# → 31 instrucciones
```

---

## 📊 Progreso del Hackathon

**Deadline:** 23 de marzo de 2026, 23:30 UTC

| Fase | Descripción | Estado |
|------|-------------|--------|
| 01-setup | Config, Users, Wallets | ✅ Completada |
| 02-jobs-teams | Jobs, Teams, Apply/Accept | ✅ Completada |
| 03-disputes | Disputes, Milestones, Treasury | ✅ Completada |
| 04-tests-idl | Tests, IDL, Documentación | 🔲 Pendiente |

### Instrucciones del Contrato (31 total)

| Módulo | # | Instrucciones |
|--------|---|---------------|
| Config | 5 | `initialize_config`, `pause`, `unpause`, `withdraw_treasury`, `update_treasury` |
| User | 4 | `create_user`, `add_wallet`, `set_active_wallet`, `update_user` |
| Team | 2 | `create_team`, `add_team_member` |
| Job | 8 | `create_job`, `deposit_funds`, `apply_to_job`, `accept_application`, `submit_work`, `approve_work`, `reject_work`, `cancel_job` |
| Arbiter | 3 | `create_arbiter_pool`, `add_arbiter`, `remove_arbiter` |
| Dispute | 5 | `raise_dispute`, `submit_evidence`, `assign_arbiter`, `resolve_dispute`, `finalize_dispute_payouts` |
| Milestone | 4 | `create_milestone`, `submit_milestone`, `approve_milestone`, `reject_milestone` |

---

## 🔄 Flujo de un Job

```
CREATED → APPLICATIONS_OPEN → IN_PROGRESS → SUBMITTED → APPROVED
                       ↓                              ↓
                  CANCELLED                      DISPUTED → RESOLVED
                   (refund)
```

**Seguridad:**
- Cliente ≠ Freelancer siempre
- ÁRBITRO seleccionado del pool
- Programa pausable por admin
- Fondos solo se mueven después de approve

---

## 📖 Documentación

| Documento | Descripción |
|-----------|-------------|
| [AGENDA.md](./trust-escrow-v2/AGENDA.md) | Roadmap completo con fases y tareas |
| [v1 README](./trust-escrow/README.md) | Documentación del proyecto bootcamp |

---

## 🤝 Contribuir

Ver [CONTRIBUTING.md](./CONTRIBUTING.md)

## 📄 Licencia

MIT License — ver [LICENSE](./LICENSE)

---

**Construido para el [WayLearn Solana Hackathon 2026](https://dorahacks.io/hackathon/solana-waylearn-2026/detail)**
