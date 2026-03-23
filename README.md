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
| **Comisiones** | 3-10% | 5% entrada + 5% salida |
| **Velocidad** | Días/semanas | Instantáneo en Solana |
| **Identidad** | Requiere KYC | Solo wallet |
| **Disputas** | Decisión unilateral | Arbitraje con % configurable |

---

## ⚡ Características Principales

### Smart Contract (Anchor/Rust)
- **Multi-wallet**: hasta 5 wallets por usuario
- **Equipos**: freelancers en grupo con split automático de pagos (Owner, PM, Contributors)
- **Arbitraje**: pool de árbitros, stake del 2.5% por parte (total 5%), resolución en 7 días
- **Auto-aprobación**: si el cliente no responde en 7 días → pago automático al freelancer
- **Pausable**: admin puede pausar el programa en emergencias

### Modelo de Comisiones
| Tipo | Porcentaje | Cuándo se cobra |
|------|-----------|-----------------|
| Fee de entrada | 5% | Cliente publica el job |
| Fee de salida | 5% | Freelancer cobra |
| Stake de disputa | 2.5% × 2 | Cada parte al abrir disputa → se paga al árbitro |

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
│                      BACKEND API                        │
│                 Rust + Axum + SQLx                      │
│          (planes post-hackathon)                        │
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
│   │   └── trust-escrow-v2/  # Smart contract Anchor
│   │       └── src/
│   │           ├── lib.rs            # Instrucciones declaradas
│   │           ├── error.rs          # Códigos de error
│   │           └── state/           # Cuentas (Config, User, Job, Team, etc.)
│   ├── Anchor.toml
│   ├── AGENDA.md             # Roadmap de desarrollo
│   └── docs/                 # Planificación completa
│       ├── planning/         # PRD, TDD, SDD, requirements
│       └── architecture/     # System design, DB schema
│
└── README.md                 # Este archivo
```

---

## 🚀 Cómo Construir

### Prerrequisitos

- Rust 1.89+
- Solana CLI 2.x
- Anchor 0.32+
- Node.js 20 LTS

### Smart Contract (v2)

```bash
cd trust-escrow-v2

# Instalar dependencias
cd programs/trust-escrow-v2 && cargo fetch

# Compilar
anchor build

# Ver IDL
cat target/idl/trust_escrow_v2.json | jq '.instructions | length'
# → 37 instrucciones
```

### v1 (CLI + TUI)

```bash
cd trust-escrow

# CLI
cargo build --manifest-path cli/Cargo.toml

# TUI
cargo build --manifest-path tui/Cargo.toml

# Tests
cd trust-escrow && yarn install && anchor test
```

---

## 📊 Progreso del Hackathon

**Deadline:** 23 de marzo de 2026, 23:30 UTC

| Fase | Descripción | Estado |
|------|-------------|--------|
| 01-setup | Config, Users, Wallets | ✅ Completada |
| 02-jobs-teams | Jobs, Teams, Apply/Accept | 🔄 En progreso |
| 03-disputes | Disputes, Milestones, Treasury | 🔲 Pendiente |
| 04-tests-idl | Tests, IDL, Documentación | 🔲 Pendiente |

### Instrucciones del Contrato (37 total)

| Módulo | # | Instrucciones |
|--------|---|---------------|
| Config | 4 | `initialize_config`, `update_config`, `pause`, `unpause` |
| User | 2 | `create_user`, `update_user` |
| Wallet | 3 | `add_wallet`, `set_active_wallet`, `remove_wallet` |
| Team | 5 | `create_team`, `update_team`, `add_member`, `remove_member`, `update_member` |
| Job | 2 | `create_job`, `publish_job` |
| Application | 4 | `apply_to_job`, `accept_application`, `reject_application`, `withdraw_application` |
| Work | 5 | `submit_work`, `approve_work`, `reject_work`, `cancel_job`, `auto_approve_job` |
| Dispute | 6 | `raise_dispute`, `submit_evidence`, `assign_arbitrer`, `resolve_dispute`, `extend_dispute_time`, `penalty_arbitrer` |
| Milestone | 3 | `create_milestone`, `approve_milestone`, `reject_milestone` |
| Treasury | 3 | `withdraw_treasury`, `set_treasurer`, `update_treasury` |

---

## 🔧 Stack Tecnológico

| Componente | Tecnología | Estado |
|------------|------------|--------|
| Smart Contract | Anchor 0.32 + Rust | 🔄 En desarrollo |
| SDK | Rust | 🔲 Pendiente |
| Backend | Rust + Axum | 🔲 Post-hackathon |
| Frontend | Next.js 14 + Tailwind | 🔲 Post-hackathon |
| CLI | Rust + Clap | ✅ Funcional (v1) |
| TUI | Rust + Ratatui | ✅ Funcional (v1) |

---

## 🔄 Flujo de un Job

```
CREATED → APPLICATIONS_OPEN → IN_PROGRESS → SUBMITTED → APPROVED
                      ↓                              ↓
                 REJECTED                       AUTO_APPROVED (7 días)
                       ↓
                   DISPUTED → RESOLVED
```

**Seguridad:**
- Cliente ≠ Freelancer siempre
- ÁRBITRO ≠ cliente Y ÁRBITRO ≠ freelancer
- Programa pausable por admin
- Fondos solo se mueven una vez

---

## 📖 Documentación

| Documento | Descripción |
|-----------|-------------|
| [AGENDA.md](./trust-escrow-v2/AGENDA.md) | Roadmap completo con fases y tareas |
| [PRD](./trust-escrow-v2/docs/planning/PRD.md) | Product Requirements Document |
| [System Design](./trust-escrow-v2/docs/architecture/SYSTEM_DESIGN.md) | Arquitectura de alto nivel |
| [SPEC_DRIVER](./trust-escrow-v2/docs/implementation/SPEC_DRIVER.md) | Especificaciones para IA |
| [v1 README](./trust-escrow/README.md) | Documentación del proyecto bootcamp |

---

## 🤝 Contribuir

Ver [CONTRIBUTING.md](./CONTRIBUTING.md)

## 📄 Licencia

MIT License — ver [LICENSE](./LICENSE)

---

**Construido para el [WayLearn Solana Hackathon 2026](https://dorahacks.io/hackathon/solana-waylearn-2026/detail)**
