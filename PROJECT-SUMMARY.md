# Trust Work Escrow v3 - Resumen Ejecutivo del Proyecto 🚀

> **Protocolo de escrow descentralizado en Solana para freelancers y clientes**  
> **Fuente de verdad: `trust-escrow-v3` (40 ix, split, Vec50, timelock, RemainingAccounts typed) — `trust-escrow-v2` es legacy**  
> **Desarrollado para el WayLearn Solana Hackathon 2026**

[![Solana](https://img.shields.io/badge/Solana-2.x-9945FF?logo=solana&logoColor=white)](https://solana.com)
[![Anchor](https://img.shields.io/badge/Anchor-0.32-blue)](https://www.anchor-lang.com)
[![Rust](https://img.shields.io/badge/Rust-1.89-orange?logo=rust)](https://www.rust-lang.org)
[![WayLearn](https://img.shields.io/badge/WayLearn-Hackathon-FF6B6B?logo=rocket)](https://dorahacks.io/hackathon/solana-waylearn-2026/detail)

---

## 📋 Resumen Ejecutivo

**Trust Work Escrow v3** es el protocolo on-chain que elimina intermediarios en transacciones cliente-freelancer. Evoluciona de v2 (31 ix, `lib.rs` monolítico 1485 LOC por bug Anchor #3690) a **v3: 40 instrucciones, arquitectura split, `RemainingAccounts` tipado, timelock y límites Vec50**.

### Objetivos Cumplidos (v3)
- ✅ **40 instrucciones** (v2: 31) — config/authority timelock, job Vec50, dispute/support, milestone
- ✅ **Arquitectura split** — `instructions/{config,job,dispute,milestone}` + `state/*`, no monolítico
- ✅ **RemainingAccounts typed** — `Vec<AccountMetaBorsh>` borsh, paginación 10/tx, evita `Vec<Pubkey>` inline
- ✅ **Timelock** — `propose_authority` → `update_authority` (ventana 7d `AUTO_APPROVAL_DELAY`)
- ✅ **Vec50** — `MAX_APPLICATIONS=50` por job, `MAX_MILESTONES=20`, `MAX_EVIDENCE=10`
- ✅ **App Dioxus 0.7** (`app/`) fullstack + `backend/api` (31 endpoints, `/health`) + devnet `7a2YhCd7...`

---

## 🎯 Valor Propuesto

### Problema
- Centralización, comisiones 3-10%, lentitud, disputas unilaterales

### Solución v3
```
Cliente deposita → Freelancer entrega → Cliente aprueba → Pago instantáneo
                                    ↓
                     Si hay conflicto → Árbitro on-chain + platform case
```

| Aspecto | Tradicional | Trust Work Escrow v3 |
|---------|-------------|---------------------|
| **Confianza** | Tercero centralizado | Smart contract split + timelock |
| **Comisiones** | 3-10% fijas | 2.5% por parte solo en disputa → `arbitration_treasury` |
| **Escalabilidad** | Manual | Vec50 + RemainingAccounts paginado |
| **Disputas** | Unilateral | Pool árbitros + `resolve_platform_case` |

---

## 🏗️ Arquitectura Técnica

### Decisiones de Diseño Clave (v3)

#### 1. Split vs Monolítico
**v2**: `lib.rs` 1485 LOC monolítico (bug Anchor #3690).  
**v3**: `lib.rs` delega a `instructions/*` y `state/*` — compilado igual, mantenible, testeado por módulo.

#### 2. RemainingAccounts tipado (V3-ARCH-004)
`RemainingAccounts { metas: Vec<AccountMetaBorsh> }` serializado borsh, mirror de `AccountMeta`. Paginación `MAX_CLEANUP_BATCH=10` / `MAX_EVIDENCE_CLEANUP_BATCH=10`. Fuzz harness valida deserialización arbitraria.

#### 3. Timelock de autoridad
`propose_authority(new) → cancel_authority_proposal` o `update_authority` tras ventana. Separa `authority` (config) de `advisor` (platform cases).

#### 4. Vec50 + límites explícitos
`MAX_APPLICATIONS=50`, `MAX_MILESTONES=20`, `MAX_EVIDENCE=10`, `AUTO_APPROVAL_DELAY=604800`. Evita DoS por spikes.

### Stack
```
┌─────────────────────────────────────────────────────────┐
│  APP: Dioxus 0.7 fullstack (app/) + landing (dcdev)     │
│  BACKEND: Rust Axum + Postgres + Mongo (docker-compose) │
├─────────────────────────────────────────────────────────┤
│  SMART CONTRACT v3: Anchor 0.32 + Rust 1.89             │
│  40 ix, split, 69KB IDL (sha256 de4a6...), 502KB .so   │
└─────────────────────────────────────────────────────────┘
```

---

## 📊 Métricas

| Métrica | v2 (legacy) | v3 (actual) |
|---------|-------------|-------------|
| **Líneas Rust** | 1485 monolítico | ~2519 split (`instructions` 2317 + `state` 139 + `lib` 1270) |
| **Instrucciones** | 31 | 40 (43 pub fn totales, 3 son helpers) |
| **IDL** | `trust_escrow_v2.json` | `trust_escrow_v3.json` 69KB |
| **PDAs** | 8 | 8 + `Application` paginada + `Evidence`/`SupportTicket` |
| **Límites** | MAX_MILESTONES 20 | + MAX_APPLICATIONS 50, timelock 7d |

---

## 🚀 Desarrollo por Fases (v3)

### Fase 1: Fundación ✅
Config split, timelock, `arbitration_treasury` separado, `rust-toolchain.toml` 1.89

### Fase 2: Core ✅
Job Vec50, `RemainingAccounts` + paginación 10/tx, milestone, `pause_job`/`expire_paused_job`

### Fase 3: Disputas ✅
`raise/accept/submit_evidence/assign/resolve` + `resolve_platform_case`, `open_support_ticket`, fee 5% total solo en disputa

### Fase 4: Tests y Deploy ✅
40 ix en IDL, `cargo test` + `anchor test` (skipped sin keys), devnet `7a2YhCd7...` slot 486034851

---

## 🧪 Testing y Validación

| Módulo | Casos | Estado |
|--------|-------|--------|
| Config/Authority | 9 | ✅ timelock + treasury split |
| Job | 14 | ✅ Vec50 + RemainingAccounts fuzz |
| Dispute/Support | 9 | ✅ evidence tipado |
| Milestone | 4 | ✅ 20 milestones |

Validaciones: `Client ≠ Freelancer`, `is_writable` en `RemainingAccounts`, `AUTO_APPROVAL_DELAY` exacto, `check_not_paused`.

---

## 💡 Desafíos Superados

1. **Anchor #3690 monolítico** → v3 split con `__client_accounts_*` re-export
2. **Vec<Pubkey> inline** → `RemainingAccounts` borsh + batch 10
3. **Autoridad sin timelock** → `propose/update/cancel_authority_proposal`
4. **App fullstack** → `app/` Dioxus 0.7 + `backend/sdk` devnet 7a2Y

---

## 🔮 Roadmap Post-Hackathon

- Frontend `app/` → `/dashboard/*` completo (client/freelancer/admin)
- SPL tokens además de SOL, reputation on-chain, DAO governance
- `cargo audit --deny warnings` estricto + `anchor verify` en CI (ver `.github/workflows/ci.yml`)

---

## 🏆 Entregables

- **Contrato v3**: `trust-escrow-v3/programs/trust-escrow-v3/src/lib.rs` + `instructions/*`
- **App**: `app/` Dioxus 0.7 (ver `app/README.md`)
- **Deploy**: devnet `7a2YhCd7iivXfyySkp1pf5jjijGqpjNqwQCUS912q5Vh` (ver `docs/DEPLOYMENT_V3.md`)
- **Video/Discord**: pendiente — ver `docs/ENTREGA_MVP.md` (`Drive: pendiente`, checklist ☐)

---

## 🚀 Deploy

```bash
cd trust-escrow-v3
anchor build -- --max-len 700000
solana program show 7a2YhCd7iivXfyySkp1pf5jjijGqpjNqwQCUS912q5Vh --url devnet
```

---

**🚀 Construido con ❤️ para el WayLearn Solana Hackathon 2026**  
**🛡️ Confianza descentralizada, pagos seguros, futuro transparente — v3 es la fuente de verdad**
