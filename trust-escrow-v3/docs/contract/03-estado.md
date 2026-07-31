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

## Enums (con `Space` impl manual, estilo v2)

Cada enum implementa manualmente `anchor_lang::Space` (`INIT_SPACE = 1`) porque
se serializa en 1 byte.

- **`JobStatus`**: `Created, Funded, InProgress, Submitted, Released, Disputed, Resolved, Cancelled`.
- **`ApplicationStatus`**: `Pending, Accepted, Rejected, Withdrawn`.
- **`DisputeStatus`**: `Open, EvidenceSubmitted, ArbiterAssigned, Resolved, Expired`.
- **`MilestoneStatus`**: `Pending, Submitted, Approved, Rejected`.

## Cuentas (PDAs)

### `Config` — seed `[b"config"]`
- `authority: Pubkey` — quien puede pausar/actualizar treasury.
- `advisor: Pubkey` — asesor de plataforma; resuelve `PlatformCase` y disputeos no mutuos. **Separado de `authority`.**
- `treasury: Pubkey` — wallet que recibe las fees de plataforma (debe firmar en `withdraw_treasury`).
- `fee_bps: u16` — fee de plataforma en basis points.
- `paused: bool`
- `bump: u8`

### `ArbitrationEscrow` — seed `[b"arb_fee", job]` (se crea al abrir disputa)
Guarda los bonos de arbitraje: `client_bond` (2.5%) y `freelancer_bond` (2.5%).
Se cierra en `finalize` pagando el 5% al resolutor.

### `Job` — seed `[b"job", client, job_id]`
Campos clave: `client`, `freelancer: Option<Pubkey>`,
`amount`, `fee_amount`, `status`, `title`, `description`, `deadline`,
`created_at`, `updated_at`, `submitted_at: Option<i64>`,
`milestones_total`, `milestones_approved`, `milestones_amount_total`,
`applications: Vec<Application>`, `bump`.

> **Nota de diseño (arbitraje):** el `Job` **no** guarda un árbitro. El árbitro
> es neutral y lo asigna la **plataforma** (`config.authority`) únicamente cuando
> se abre una disputa, para no cargar a los árbitros con "trabajos fantasma"
> (jobs que nunca entran en disputa). El árbitro vive en la cuenta `Dispute`.

### `ArbiterPool` — seed `[b"arbiter_pool"]`
`authority`, `arbiters: Vec<Pubkey>`, `bump`.

### `Dispute` — seed `[b"dispute", job]`
`job`, `raised_by`, `arbiter: Option<Pubkey>` (asignado por la plataforma en
`assign_arbiter`), `status`, `evidence: Vec<Evidence>`, `reason`,
`created_at`, `deadline`, `resolved_at`, `resolution`,
`client_payout_percent`, `freelancer_payout_percent`, `bump`.

### `Milestone` — seed `[b"milestone", job, index]`
`job`, `title`, `description`, `amount`, `deadline`, `status`, `index`,
`submitted_at`, `approved_at`, `bump`, `created_at`.

### Estructuras auxiliares
- `Application { applicant, proposal, applied_at, status }`
- `Evidence { submitter, content, submitted_at }`

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
