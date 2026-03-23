# Phase 3: Disputes, Milestones & Treasury - Trust Work Escrow v2

## Descripción

Implementación del sistema de disputas, milestones y treasury management.

## Fecha

2026-03-23

## Estado

✅ Completado

---

## Instrucciones Implementadas

### Arbiter Pool Instructions (3)

| Instrucción | Descripción |
|------------|-------------|
| `create_arbiter_pool` | Crea pool de árbitros autorizados |
| `add_arbiter` | Admin agrega árbitro al pool (max 50) |
| `remove_arbiter` | Admin elimina árbitro del pool |

### Dispute Instructions (5)

| Instrucción | Descripción |
|------------|-------------|
| `raise_dispute` | Cliente o Freelancer abre disputa con razón y deadline |
| `submit_evidence` | Cualquiera de las partes submits evidencia |
| `assign_arbiter` | Admin asigna árbitro del pool a la disputa |
| `resolve_dispute` | Árbitro resuelve con distribución % |
| `finalize_dispute_payouts` | Ejecuta los pagos según resolución |

### Milestone Instructions (4)

| Instrucción | Descripción |
|------------|-------------|
| `create_milestone` | Cliente crea hito con título, descripción, monto y deadline |
| `submit_milestone` | Freelancer entrega milestone |
| `approve_milestone` | Cliente aprueba y paga el milestone |
| `reject_milestone` | Cliente rechaza milestone |

### Treasury Instructions (1)

| Instrucción | Descripción |
|------------|-------------|
| `update_treasury` | Admin actualiza dirección del treasury |

*(Nota: `withdraw_treasury` ya estaba en Phase 02)*

---

## Estructuras de Datos

### ArbiterPool
```rust
pub struct ArbiterPool {
    pub authority: Pubkey,           // Admin que puede agregar/quitar
    pub arbiters: Vec<Pubkey>,        // max 50
    pub bump: u8,
}
```

### Dispute
```rust
pub struct Dispute {
    pub job: Pubkey,
    pub raised_by: Pubkey,
    pub arbiter: Option<Pubkey>,
    pub status: DisputeStatus,        // Open, EvidenceSubmitted, ArbiterAssigned, Resolved, Expired
    pub evidence: Vec<Evidence>,      // max 10 items
    pub reason: String,               // max 500
    pub created_at: i64,
    pub deadline: i64,
    pub resolved_at: Option<i64>,
    pub resolution: Option<String>,   // max 500
    pub client_payout_percent: u8,
    pub freelancer_payout_percent: u8,
    pub bump: u8,
}
```

### Evidence
```rust
pub struct Evidence {
    pub submitter: Pubkey,
    pub content: String,             // max 2048
    pub submitted_at: i64,
}
```

### Milestone
```rust
pub struct Milestone {
    pub job: Pubkey,
    pub title: String,                // max 64
    pub description: String,          // max 1024
    pub amount: u64,
    pub deadline: i64,
    pub status: MilestoneStatus,      // Pending, Submitted, Approved, Rejected
    pub index: u8,
    pub submitted_at: Option<i64>,
    pub approved_at: Option<i64>,
    pub bump: u8,
    pub created_at: i64,
}
```

---

## Seeds de PDAs

| Cuenta | Seed | Creador |
|--------|------|---------|
| ArbiterPool | `b"arbiter_pool"` | create_arbiter_pool |
| Dispute | `b"dispute", job` | raise_dispute |
| Milestone | `b"milestone", job, index` | create_milestone |

---

## Flujo de Disputa

```
OPEN → EVIDENCE_SUBMITTED → ARBITER_ASSIGNED → RESOLVED → FINALIZED
                        ↓
                    EXPIRED (si pasa deadline)
```

### Pagos en Resolución
- Árbitro define `%` para cliente y freelancer
- `finalize_dispute_payouts` ejecuta:
  - Client recibe: `amount * client_payout_percent / 100`
  - Freelancer recibe: `amount * freelancer_payout_percent / 100`

---

## Flujo de Milestones

```
PENDING → SUBMITTED → APPROVED
                   → REJECTED (vuelve a PENDING)
```

- Cada milestone tiene su propio deadline
- Pagos parciales por milestone
- No requiere que todos estén completos para approve final

---

## Features de Seguridad

- ✅ Solo pool arbiters pueden ser asignados
- ✅ Árbitro debe ser assignee de la disputa
- ✅ Evidencia tiene límite de tamaño (2048 chars)
- ✅ Disputas solo se pueden raise en stages válidos
- ✅ Deadline para resolución de disputas
- ✅ Milestones solo pueden ser submit antes del deadline
- ✅ Solo owner del job puede approve/reject milestones

---

## Siguiente

Phase 4: Tests, IDL & Documentation
