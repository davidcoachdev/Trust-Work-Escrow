# 📋 AGENDA - Trust Work Escrow v2

## 🎯 Objetivo General
Desarrollar el protocolo de escrow descentralizado en Solana para el hackathon de WayLearn.

**Deadline:** 23 de marzo de 2026, 23:30 UTC

---

## 📅 Estructura de Fases

### Fase 01: Setup ✅ COMPLETADA
- [x] Config, Users, Wallets
- **Commits:** 3
- **PR:** #20

### Fase 02: Jobs, Teams, Apply/Accept ✅ COMPLETADA
- [x] Teams: create_team, add_member
- [x] Jobs: create_job, deposit_funds, apply_to_job, accept_application
- [x] Trabajo: submit_work, approve_work, reject_work, cancel_job
- **Rama:** `feat/contract/jobs-teams`
- **Commits:** `2637f3b`, nuevo commit consolidado

### Fase 03: Disputes, Milestones, Treasury ✅ COMPLETADA
- [x] Arbiter Pool: create_arbiter_pool, add_arbiter, remove_arbiter
- [x] Disputes: raise_dispute, submit_evidence, assign_arbiter, resolve_dispute, finalize_dispute_payouts
- [x] Milestones: create_milestone, submit_milestone, approve_milestone, reject_milestone
- [x] Treasury: withdraw_treasury, update_treasury
- **Rama:** `feat/contract/jobs-teams` (consolidado)

### Fase 04: Tests, IDL, Documentación 🔲 PENDIENTE
- [ ] Tests de integración
- [ ] IDL generado
- [ ] Documentación completa
- **Rama:** `feat/contract/tests-idl`

---

## 📊 Resumen de Epic: Smart Contract

| Fase | Descripción | Tasks | Estado |
|------|-------------|-------|--------|
| 01 | Setup | 9 | ✅ |
| 02 | Jobs, Teams, Apply | 17 | ✅ |
| 03 | Disputes, Milestones, Treasury | 15 | ✅ |
| 04 | Tests, IDL | 12 | 🔲 |

**Total tasks:** 49 (12 pendientes en Phase 04)

---

## 🔧 Instrucciones del Contrato (37 total)

### Config (4) ✅
| # | Instrucción | Descripción |
|---|-------------|-------------|
| 1 | initialize_config | Inicializa config global |
| 2 | pause | Pausa programa |
| 3 | unpause | Reactiva programa |
| 4 | withdraw_treasury | Retira de treasury |
| 5 | update_treasury | Actualiza treasury address |

### User (4) ✅
| # | Instrucción | Descripción |
|---|-------------|-------------|
| 6 | create_user | Crea perfil de usuario |
| 7 | add_wallet | Agrega wallet (max 5) |
| 8 | set_active_wallet | Cambia wallet activa |
| 9 | update_user | Actualiza bio |

### Team (2) ✅
| # | Instrucción | Descripción |
|---|-------------|-------------|
| 10 | create_team | Crea equipo |
| 11 | add_team_member | Agrega miembro |

### Job (8) ✅
| # | Instrucción | Descripción |
|---|-------------|-------------|
| 12 | create_job | Crea job |
| 13 | deposit_funds | Cliente fondea job |
| 14 | apply_to_job | Freelancer aplica |
| 15 | accept_application | Cliente acepta aplicación |
| 16 | submit_work | Freelancer entrega |
| 17 | approve_work | Cliente aprueba |
| 18 | reject_work | Cliente rechaza |
| 19 | cancel_job | Cancela job |

### Arbiter Pool (3) ✅
| # | Instrucción | Descripción |
|---|-------------|-------------|
| 20 | create_arbiter_pool | Crea pool de árbitros |
| 21 | add_arbiter | Agrega árbitro |
| 22 | remove_arbiter | Elimina árbitro |

### Dispute (5) ✅
| # | Instrucción | Descripción |
|---|-------------|-------------|
| 23 | raise_dispute | Abre disputa |
| 24 | submit_evidence | Submit evidencia |
| 25 | assign_arbiter | Asigna árbitro |
| 26 | resolve_dispute | Resuelve disputa |
| 27 | finalize_dispute_payouts | Ejecuta pagos de disputa |

### Milestone (4) ✅
| # | Instrucción | Descripción |
|---|-------------|-------------|
| 28 | create_milestone | Crea hito |
| 29 | submit_milestone | Freelancer entrega hito |
| 30 | approve_milestone | Cliente aprueba hito |
| 31 | reject_milestone | Cliente rechaza hito |

---

## 📁 Estructura del Contrato

### Single File Architecture ⚠️
**NOTA:** Todo el contrato está en un solo archivo `lib.rs` debido a un bug conocido en Anchor 0.32 con módulos anidados.

```
programs/trust-escrow-v2/src/
└── lib.rs              # TODO el contrato (~1400 líneas)
    ├── Constants
    ├── ErrorCode enum
    ├── State enums (JobStatus, ApplicationStatus, MemberRole, DisputeStatus, MilestoneStatus)
    ├── State structs (Config, User, Job, Team, ArbiterPool, Dispute, Milestone)
    ├── Program module (todas las instrucciones)
    └── Accounts contexts (todos los #[derive(Accounts)])
```

### Cuentas (PDAs)
| Cuenta | Seed | Descripción |
|--------|------|-------------|
| Config | `b"config"` | Configuración global |
| User | `b"user", authority` | Perfil de usuario |
| Team | `b"team", owner` | Equipo de freelancers |
| Job | `b"job", client, job_id` | Job posting |
| ArbiterPool | `b"arbiter_pool"` | Pool de árbitros |
| Dispute | `b"dispute", job` | Disputa abierta |
| Milestone | `b"milestone", job, index` | Hito de job |

---

## 💰 Modelo de Negocio

### Fees
| Tipo | Porcentaje | Cuándo |
|------|-------------|---------|
| Entrada | Configurable (0-100%) | Cliente publica job |
| Salida | Igual al de entrada | Freelancer cobra |

### Disputas
- Stake: 0% (sin implementar aún)
- Resolución: % configurable entre cliente y freelancer
- Árbiro: seleccionado del pool

---

## 🔄 Flujo de Jobs

```
CREATED → APPLICATIONS_OPEN → IN_PROGRESS → SUBMITTED → APPROVED
                       ↓                              ↓
                  CANCELLED                      DISPUTED → RESOLVED
                   (refund)
```

---

## 📅 Timeline Hackathon

| Día | Fecha | Objetivo |
|-----|-------|----------|
| Día 1 | 22-23 Mar | Setup + Fase 01 ✅ |
| Día 2 | 23 Mar | Fase 02 ✅ |
| Día 3 | 23 Mar | Fase 03 ✅ |
| Día 4 | 23 Mar | Fase 04 + Deploy 🔲 |

**Deadline:** 23 de marzo, 23:30 UTC

---

## 🚀 Comandos Útiles

```bash
# Compilar (desde programs/trust-escrow-v2/)
cargo build

# Ver IDL
cat target/idl/trust_escrow_v2.json | jq '.instructions | length'
```

---

_Last updated: 2026-03-23_
