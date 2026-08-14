# 03 · Estado: Cuentas, Enums, Constantes y Helpers

Define la memoria on-chain del contrato. Todo lo declarado aquí es compartido
por las instrucciones documentadas en las demás partes.

## Constantes

| Constante | Valor | Uso |
|-----------|-------|-----|
| `BASIS_POINTS` | `10_000` | Base para `fee_bps` (10000 = 100%). Corrige el bug de v2. |
| `MIN_JOB_AMOUNT` | `100_000` lamports | Mínimo para `create_job`/`deposit_funds`. |
| `MAX_TITLE_LENGTH` | `100` | Largo máximo de `title`. |
| `MAX_DESCRIPTION_LENGTH` | `500` | Largo máximo de `description`. |
| `MAX_PROPOSAL_LENGTH` | `512` | Largo máximo de propuesta de aplicación. |
| `MAX_DISPUTE_REASON` | `500` | Largo máximo de razón de disputa. |
| `MAX_DISPUTE_EVIDENCE` | `2048` | Largo máximo de cada evidencia. |
| `MAX_MILESTONE_TITLE` | `64` | Largo máximo del título de milestone. |
| `MAX_MILESTONES` | `20` | Cantidad máxima de milestones por job. |
| `MAX_APPLICATIONS` | `50` | Cantidad máxima de aplicaciones por job. |
| `MAX_ARBITERS` | `50` | Cantidad máxima de árbitros en el pool. |
| `ARBITER_FEE_BPS_PER_PARTY` | `250` (=2.5%) | Fee de arbitraje por parte. Se cobra al cliente y al freelancer (5% total del job) **solo en disputas**. |
| `AUTO_APPROVAL_DELAY` | `604800` segundos | Ventana exacta desde `submitted_at`; el límite es inclusivo (`now >= submitted_at + 604800`). |

## Enums (con `Space` impl manual, estilo v2)

Cada enum implementa manualmente `anchor_lang::Space` (`INIT_SPACE = 1`) porque
se serializa en 1 byte.

- **`JobStatus`**: `Created, Funded, InProgress, Submitted, Released, Disputed, Resolved, Cancelled`. `submit_work` pasa directamente a `Submitted`; no existe `Received`.
- **`ApplicationStatus`**: `Pending, Accepted, Rejected, Withdrawn`.
- **`DisputeStatus`**: `Open, EvidenceSubmitted, ArbiterAssigned, Resolved, Expired`.
- **`MilestoneStatus`**: `Pending, Submitted, Approved, Rejected`.

## Cuentas (PDAs)

### `Config` — seed `[b"config"]`
- `authority: Pubkey` — quien puede pausar/actualizar treasury.
- `advisor: Pubkey` — asesor de plataforma; resuelve `PlatformCase` y disputeos no mutuos. **Separado de `authority`.**
- `treasury: Pubkey` — wallet que recibe las fees de plataforma (debe firmar en `withdraw_treasury`).
- `arbitration_treasury: Pubkey` — destino separado de fee/shortfall de arbitraje.
- `fee_bps: u16` — fee de plataforma en basis points.
- `paused: bool`
- `bump: u8`

### `ArbitrationEscrow` — seed `[b"arb_fee", job]` (se crea al abrir disputa)
Guarda los bonos de arbitraje: `client_bond` (2.5%) y `freelancer_bond` (2.5%).
Se cierra en `finalize` enviando los bonos posteados a
`arbitration_treasury`; el resolutor solo autoriza la operación.

### `Job` — seed `[b"job", client, job_id]`
Campos clave: `client`, `freelancer: Option<Pubkey>`,
`amount`, `fee_amount`, `status`, `title`, `description`, `deadline`,
`created_at`, `updated_at`, `submitted_at: Option<i64>`,
`milestones_total`, `milestones_approved`, `milestones_amount_total`,
`applicants: Vec<Pubkey>` (máximo 50, solo índice compacto para detectar
duplicados), `bump`. Las propuestas no se almacenan inline.

> **Nota de diseño (arbitraje):** el `Job` **no** guarda un árbitro. El árbitro
> es neutral y lo asigna la **plataforma** (`config.authority`) únicamente cuando
> se abre una disputa, para no cargar a los árbitros con "trabajos fantasma"
> (jobs que nunca entran en disputa). El árbitro vive en la cuenta `Dispute`.

### `ArbiterPool` — seed `[b"arbiter_pool"]`
`authority`, `arbiters: Vec<Pubkey>`, `bump`.

### `Dispute` — seed `[b"dispute", job]`
`job`, `raised_by`, `arbiter: Option<Pubkey>` (asignado por la plataforma en
`assign_arbiter`), `status`, `evidence_count`, `evidence_cleanup_cursor`, `reason`,
`created_at`, `deadline`, `resolved_at`, `resolution`,
`client_payout_percent`, `freelancer_payout_percent`, `bump`.

### `Milestone` — seed `[b"milestone", job, index]`
`job`, `title`, `description`, `amount`, `deadline`, `status`, `index`,
`submitted_at`, `approved_at`, `bump`, `created_at`.

### `Application` — seed `[b"application", job, index, applicant]`
- PDA individual creada por `apply_to_job`; contiene `job`, `index`, `applicant`,
  `proposal` (máximo `MAX_PROPOSAL_LENGTH`), `applied_at`, `status` y `bump`.
- El índice debe ser el siguiente (0–49), el job y applicant deben coincidir con
  las seeds, y `Job.applicants` impide duplicados.
- `accept_application` valida la PDA y cierra la cuenta, devolviendo su rent al
  applicant; la rent nunca se considera parte del payout.

### Evidence PDA — seed `[b"evidence", dispute, index]`
- Cada evidencia vive en una cuenta PDA individual `Evidence`; `Dispute` solo
  conserva contadores y cursores de cleanup.
- `Evidence { dispute, index, author, content, submitted_at, bump }`.
- Máximo 10 PDAs por disputa y máximo 2048 bytes por `content`.
- `finalize_dispute_payouts`/`cleanup_dispute_evidence` cierran las PDAs en
  orden y devuelven su rent al cliente; esa rent no es payout.
- `Evidence` no documenta ni almacena un hash de evidencia salvo que el IDL/código
  vigente muestre explícitamente ese campo. Los digests de archivos o reportes
  fuera del contrato son evidencia externa.

## Helper: `compute_fee`

```rust
pub fn compute_fee(amount: u64, fee_bps: u16) -> Result<u64> {
    let fee = (amount as u128)
        .checked_mul(fee_bps as u128)
        .ok_or(ErrorCode::MathOverflow)?
        / BASIS_POINTS as u128;
    Ok(fee as u64)
}
```

**Por qué**: centraliza el cálculo de la fee con aritmética chequeada para
evitar desbordamientos silenciosos y el error de base de v2 (`/10000` con
validación 0–100). Se usa en `create_job` (y se usará en las liberaciones).
