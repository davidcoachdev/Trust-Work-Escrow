# Separación On-chain / Off-chain — Trust-Work-Escrow v3 (Análisis para Backend)

> **Propósito:** Decidir qué información del contrato v3 debe vivir en el backend (off-chain)
> y qué debe seguir subiéndose al contrato (on-chain), antes de construir el backend en Rust.
> **Fecha:** 2026-08-12
> **Estado:** Análisis previo — no se escribe código de backend todavía.

---

## 1. Contexto

- Contrato objetivo: `trust-escrow-v3` (programa `J1c4QsjbV9bFEPrFQZZGe8GrGWFxNhtAhhrxJFK2xc1h`, Anchor 0.32.1).
- Contrato **ya auditado y APROBADO** (P0/P1=0, security gates PASS, deploy real + suites verdes).
  → **No se modifica el contrato v3 en esta fase.** El backend se acopla a v3 tal cual.
- El backend se construirá en Rust, reutilizando el SDK de `trust-escrow-v2` adaptado a las 39
  instrucciones de v3.

## 2. Mapa de cuentas on-chain (v3)

Se extrajeron los campos reales de los 9 PDAs principales:

| Cuenta | Campos relevantes |
|--------|------------------|
| `Config` | authority, advisor, treasury, arbitration_treasury, fee_bps, paused, bump |
| `Job` | client, freelancer, amount, fee_amount, status, paused, paused_at, title, description, deadline, created_at, updated_at, submitted_at, milestones_*, applicants (Vec), bump |
| `Application` | job, index, applicant, proposal, applied_at, status, bump |
| `ArbiterPool` | authority, arbiters (Vec), bump |
| `Dispute` | job, raised_by, arbiter, status, evidence_count, reason, created_at, deadline, resolution, payout %, bump |
| `Evidence` | dispute, index, author, content (Vec\<u8\>), submitted_at, bump |
| `Milestone` | job, title, description, amount, deadline, status, index, timestamps, bump |
| `SupportTicket` | job, opened_by, reason, status, resolution, timestamps, bump |
| `ArbitrationEscrow` | job, client_bond, freelancer_bond, bump |

## 3. Clasificación

### 3.1 ON-CHAIN (estado de verdad — lo valida el contrato)

Debe seguir en el contrato. Es lo que el programa necesita para hacer cumplir reglas
(authz, montos, deadlines, payouts, lifecycle):

- Claves y relaciones: `client`, `freelancer`, `job`, `raised_by`, `arbiter`, PDAs.
- Dinero: `amount`, `fee_amount`, `client_bond`, `freelancer_bond`, payout percents.
- Lifecycle: `status` (Job/Application/Dispute/Milestone/SupportTicket), `paused`, `paused_at`,
  deadlines, `submitted_at`, `created_at`, `updated_at`, `resolved_at`.
- Config global: `authority`, `advisor`, `treasury`, `arbitration_treasury`, `fee_bps`.
- Contadores que disparan lógica: `evidence_count`, `evidence_cleanup_cursor`,
  `milestones_total/approved/amount_total`.

### 3.2 OFF-CHAIN (metadata + índice — vive en el backend)

Texto largo y binarios que **no participan en la validación del contrato**. El backend los
mantiene y los vincula al on-chain vía las claves/PDAs:

| Campo | Tipo | Razón off-chain |
|-------|------|-----------------|
| `Job.description` | String (MAX_DESC) | Texto largo, no usado en validación |
| `Job.title` | String | Metadata de UI |
| `Job.applicants` | Vec\<Pubkey\> | **Redundante** — ya existen Accounts `Application` |
| `Application.proposal` | String (MAX_PROPOSAL) | Texto largo de propuesta |
| `Dispute.reason` | String | Texto de la queja |
| `Dispute.resolution` | String | Texto de la decisión |
| `Evidence.content` | Vec\<u8\> | **Binario pesado** — hash on-chain + contenido en backend |
| `Milestone.title` / `description` | String | Metadata |
| `SupportTicket.reason` / `resolution` | String | Texto |

## 4. Opinión / Recomendación

**Arquitectura recomendada: backend como capa de metadata + índice.**

1. **On-chain = integridad + lógica.** Solo lo que el contrato necesita para validar reglas.
2. **Backend = rich data + UX.** Títulos, descripciones, proposals, evidencias y un índice
   consultable (por cliente, status, fecha, freelancer).
3. **Patrón para evidencias/binarios:** subir `sha256` + tamaño on-chain (o dejar el `Vec<u8>`
   actual y duplicar en backend para consulta), contenido completo en backend (o IPFS). El hash
   on-chain garantiza integridad sin inflar rent.
4. **`Job.applicants` es redundante.** El backend ya las indexa desde los Accounts `Application`.
   Podría quitarse del contrato para ahorrar rent, **pero v3 ya está aprobado** → no rework ahora.
   El backend lo deriva/ignora según convenga.

**Ventajas:** menos rent on-chain, updates más baratos, búsquedas/filtros en backend,
evidencias grandes sin inflar cuentas.

**Riesgos y mitigaciones:**
- Riesgo: si el backend cae, se pierde metadata off-chain.
  → Mitigar con persistencia propia (DB) + hash on-chain como comprobante de integridad.
- Riesgo: desincronización backend vs on-chain.
  → El backend debe ser *read-through* del on-chain (fuente de verdad = contrato); la metadata
    off-chain es un espejo enriquecido, no fuente de verdad.

## 5. Decisión propuesta para el backend

- El backend **no cambia v3**. Se acopla leyendo los 9 PDAs y manteniendo tablas off-chain
  espejo para la metadata rica.
- Cada entidad off-chain se vincula al on-chain por la clave PDA correspondiente (job, dispute,
  application, milestone, support_ticket).
- Para `Evidence.content` y textos largos: el backend guarda el contenido completo; el hash
  on-chain (si existe) o el content tal cual sirve de integridad.
- `Job.applicants`: el backend lo reconstruye consultando Accounts `Application` por job, en vez
  de depender del Vec on-chain.

## 6. Siguiente paso

Confirmar forma del backend (SDK solo / SDK+API REST / SDK+API+indexer) y proceder al sketch
de kits Cavekit. Ver `docs/planning/` y la conversación de planificación del backend.
