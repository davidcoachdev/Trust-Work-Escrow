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

### Fase 02: Jobs, Teams, Apply/Accept 🔄 EN PROCESO
- [ ] Teams: create_team, update_team, add_member, remove_member, update_member
- [ ] Jobs: create_job, publish_job
- [ ] Aplicación: apply_to_job, accept_application, reject_application, withdraw_application
- [ ] Trabajo: submit_work, approve_work, reject_work, cancel_job, auto_approve_job
- **Rama:** `feat/contract/jobs-teams`
- **Issues:** #8

### Fase 03: Disputes, Milestones, Treasury
- [ ] Disputes: raise_dispute, submit_evidence, assign_arbitrer, resolve_dispute, extend_dispute_time, penalty_arbitrer
- [ ] Milestones: create_milestone, approve_milestone, reject_milestone
- [ ] Treasury: withdraw_treasury, set_treasurer, update_treasury
- **Rama:** `feat/contract/disputes`

### Fase 04: Tests, IDL, Documentación
- [ ] Tests de integración
- [ ] IDL generado
- [ ] Documentación completa
- **Rama:** `feat/contract/tests-idl`

---

## 📊 Resumen de Epic: Smart Contract

| Fase | Descripción | Tasks | Estado |
|------|-------------|-------|--------|
| 01 | Setup | 9 | ✅ |
| 02 | Jobs, Teams, Apply | 17 | ⏳ |
| 03 | Disputes, Treasury | 12 | 🔲 |
| 04 | Tests, IDL | 12 | 🔲 |

**Total tasks:** 49

---

## 🔧 Instrucciones del Contrato (49 total)

### Config (4) ✅
| # | Instrucción | Descripción |
|---|-------------|-------------|
| 1 | initialize_config | Inicializa config global |
| 2 | update_config | Actualiza parámetros |
| 3 | pause | Pausa programa |
| 4 | unpause | Reactiva programa |

### User (2) ✅
| # | Instrucción | Descripción |
|---|-------------|-------------|
| 5 | create_user | Crea perfil |
| 6 | update_user | Actualiza perfil |

### Wallet (3) ✅
| # | Instrucción | Descripción |
|---|-------------|-------------|
| 7 | add_wallet | Agrega wallet |
| 8 | set_active_wallet | Cambia wallet activa |
| 9 | remove_wallet | Elimina wallet |

### Team (5) ⏳
| # | Instrucción | Descripción |
|---|-------------|-------------|
| 10 | create_team | Crea equipo |
| 11 | update_team | Actualiza equipo |
| 12 | add_member | Agrega miembro |
| 13 | remove_member | Elimina miembro |
| 14 | update_member | Actualiza rol/porcentaje |

### Job (2) ⏳
| # | Instrucción | Descripción |
|---|-------------|-------------|
| 15 | create_job | Crea job |
| 16 | publish_job | Publica y fondea job |

### Application (4) ⏳
| # | Instrucción | Descripción |
|---|-------------|-------------|
| 17 | apply_to_job | Freelancer aplica |
| 18 | accept_application | Cliente acepta |
| 19 | reject_application | Cliente rechaza |
| 20 | withdraw_application | Freelancer retira |

### Work (5) ⏳
| # | Instrucción | Descripción |
|---|-------------|-------------|
| 21 | submit_work | Freelancer entrega |
| 22 | approve_work | Cliente aprueba |
| 23 | reject_work | Cliente rechaza |
| 24 | cancel_job | Cancela job |
| 25 | auto_approve_job | Auto-aprueba 7 días |

### Dispute (6) 🔲
| # | Instrucción | Descripción |
|---|-------------|-------------|
| 26 | raise_dispute | Abre disputa |
| 27 | submit_evidence | Submit evidencia |
| 28 | assign_arbitrer | Asigna árbitro |
| 29 | resolve_dispute | Resuelve disputa |
| 30 | extend_dispute_time | Extiende tiempo |
| 31 | penalty_arbitrer | Penalty árbitro |

### Milestone (3) 🔲
| # | Instrucción | Descripción |
|---|-------------|-------------|
| 32 | create_milestone | Crea hito |
| 33 | approve_milestone | Aprueba hito |
| 34 | reject_milestone | Rechaza hito |

### Treasury (3) 🔲
| # | Instrucción | Descripción |
|---|-------------|-------------|
| 35 | withdraw_treasury | Retira de treasury |
| 36 | set_treasurer | Asigna treasurer |
| 37 | update_treasury | Actualiza treasury |

---

## 📁 Estructura de Cuentas

### Config Account
```rust
pub struct Config {
    pub admin: Pubkey,
    pub treasury_wallet: Pubkey,
    pub treasurer: Pubkey,
    pub entry_fee_bps: u16,      // 5%
    pub exit_fee_bps: u16,       // 5%
    pub dispute_stake_bps: u16,   // 2.5%
    pub max_job_duration_days: u32,
    pub auto_approve_days: u8,
    pub paused: bool,
    pub bump: u8,
}
```

### User Account
```rust
pub struct User {
    pub owner: Pubkey,
    pub username: String,
    pub bio: String,
    pub skills: String,
    pub reputation: u8,
    pub jobs_completed: u32,
    pub disputes_won: u32,
    pub disputes_lost: u32,
    pub is_arbiter: bool,
    pub wallet_count: u8,
    pub wallets: Vec<u8>,
    pub active_wallet_index: u8,
    pub bump: u8,
    pub created_at: i64,
    pub updated_at: i64,
}
```

### Team Account (por crear)
```rust
pub struct Team {
    pub owner: Pubkey,
    pub name: String,
    pub description: String,
    pub members: Vec<Member>,
    pub total_percentage: u8,
    pub bump: u8,
    pub created_at: i64,
}

pub struct Member {
    pub user: Pubkey,
    pub role: MemberRole,  // Owner, PM, Contributor
    pub percentage: u8,
    pub joined_at: i64,
}
```

### Job Account (por crear)
```rust
pub struct Job {
    pub client: Pubkey,
    pub title: String,
    pub description: String,
    pub amount: u64,
    pub deadline: i64,
    pub status: JobStatus,
    pub freelancer: Option<Pubkey>,
    pub applications: Vec<Application>,
    pub bump: u8,
    pub created_at: i64,
}

pub enum JobStatus {
    Created,
    ApplicationsOpen,
    InProgress,
    Submitted,
    Approved,
    Disputed,
    Cancelled,
}
```

---

## 💰 Modelo de Negocio

### Fees
| Tipo | Porcentaje | Cuándo |
|------|-------------|---------|
| Entrada | 5% | Cliente publica job |
| Salida | 5% | Freelancer cobra |

### Stake de Disputa
| Parte | Porcentaje | Destino |
|-------|-------------|---------|
| Cliente | 2.5% | Paga al árbitro |
| Freelancer | 2.5% | Paga al árbitro |
| **Total** | 5% | Para el árbitro |

### Reglas
- Auto-aprobar: 7 días sin respuesta del cliente
- Árbitro: 7 días para resolver (extensible)
- Penalty árbitro: 5% si no resuelve a tiempo

---

## 🔄 Flujo de Jobs

```
CREATED → APPLICATIONS_OPEN → IN_PROGRESS → SUBMITTED → APPROVED
                      ↓                              ↓
               REJECTED                       AUTO_APPROVED (7 días)
                      ↓
                  DISPUTED → RESOLVED
```

---

## 📝 Checklist por Fase

### Fase 02 Checklist
- [ ] create_team
- [ ] update_team
- [ ] add_member
- [ ] remove_member
- [ ] update_member
- [ ] create_job
- [ ] publish_job
- [ ] apply_to_job
- [ ] accept_application
- [ ] reject_application
- [ ] withdraw_application
- [ ] submit_work
- [ ] approve_work
- [ ] reject_work
- [ ] cancel_job
- [ ] auto_approve_job
- [ ] Documentación
- [ ] Tests
- [ ] PR

### Fase 03 Checklist
- [ ] raise_dispute
- [ ] submit_evidence
- [ ] assign_arbitrer
- [ ] resolve_dispute
- [ ] extend_dispute_time
- [ ] penalty_arbitrer
- [ ] create_milestone
- [ ] approve_milestone
- [ ] reject_milestone
- [ ] withdraw_treasury
- [ ] set_treasurer
- [ ] update_treasury
- [ ] Documentación
- [ ] Tests
- [ ] PR

---

## 🚀 Comandos Útiles

```bash
# Compilar
anchor build

# Testear
anchor test

# Ver IDL
cat target/idl/trust_escrow_v2.json | jq '.instructions | length'

# Ver estado de cuenta
solana account <PDA>

# Desplegar en devnet
anchor deploy --provider.cluster devnet
```

---

## 📅 Timeline Hackathon

| Día | Fecha | Objetivo |
|-----|-------|----------|
| Día 1 | 22-23 Mar | Setup + Fase 01 ✅ |
| Día 2 | 23 Mar | Fase 02 ⏳ |
| Día 3 | 23 Mar | Fase 03 |
| Día 4 | 23 Mar | Fase 04 + Deploy |

**Deadline:** 23 de marzo, 23:30 UTC

---

_Last updated: 2026-03-23_
