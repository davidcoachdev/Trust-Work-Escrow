# 🏗️ Fase 03: Disputes, Milestones, Treasury - Smart Contract

**Contexto:**  
Esta fase forma parte del Epic #1 - "Smart Contract - Trust Work Escrow v2".

---

## 📋 Descripción

Implementar el sistema de disputas con stake, gestión de hitos y retiros de tesorería.

## 🎯 Objetivo

Permitir abrir disputas, asignar árbitros, resolver conflictos y gestionar hitos y tesorería.

---

## 🔧 Tasks asignadas a esta fase

### Disputes (con stake de 5% total - 2.5% cada parte)
- [ ] `feat(contract): add raise_dispute instruction` - Abrir disputa (requiere stake de ambas partes)
- [ ] `feat(contract): add submit_evidence instruction` - Submitir evidencia a la disputa
- [ ] `feat(contract): add assign_arbitrer instruction` - Asignar árbitro aleatorio del pool
- [ ] `feat(contract): add resolve_dispute instruction` - Resolver disputa (árbitro decide split)
- [ ] `feat(contract): add extend_dispute_time instruction` - Extender tiempo del árbitro (admin)
- [ ] `feat(contract): add penalty_arbitrer instruction` - Aplicar penalty al árbitro (5% si no resuelve)

### Milestones (v2 - no para hackathon)
- [ ] `feat(contract): add create_milestone instruction` - Crear hito dentro de un job
- [ ] `feat(contract): add approve_milestone instruction` - Aprobar hito (pago parcial)
- [ ] `feat(contract): add reject_milestone instruction` - Rechazar hito

### Treasury
- [ ] `feat(contract): add withdraw_treasury instruction` - Retirar fondos de tesorería (admin/treasurer)
- [ ] `feat(contract): add set_treasurer instruction` - Asignar treasurer (admin)
- [ ] `feat(contract): add update_treasury instruction` - Actualizar configuración de tesorería

---

## 📁 Convenciones de entregables

```
programs/trust-escrow-v2/
├── src/
│   ├── instructions/
│   │   ├── dispute.rs             # raise_dispute, submit_evidence, resolve_dispute
│   │   ├── milestone.rs            # create_milestone, approve_milestone, reject_milestone
│   │   └── treasury.rs             # withdraw_treasury, set_treasurer, update_treasury
│   ├── state/
│   │   ├── dispute.rs              # Dispute account
│   │   ├── milestone.rs            # Milestone account
│   │   └── treasury.rs             # Treasury account
├── tests/
│   └── disputes.spec.ts            # Tests de disputas
└── docs/
    └── disputes.md                  # Documentación educativa
```

---

## 🔀 Rama de esta fase

**Rama**: `feat/contract/disputes`  
**Rama padre**: `feat/contract`  
**PR destino**: `feat/contract`

---

## ✅ Checklist de tareas

| Task | Rama | Estado |
|------|------|--------|
| raise_dispute | feat/contract/disputes/raise | ⏳ |
| submit_evidence | feat/contract/disputes/evidence | ⏳ |
| assign_arbitrer | feat/contract/disputes/assign | ⏳ |
| resolve_dispute | feat/contract/disputes/resolve | ⏳ |
| extend_dispute_time | feat/contract/disputes/extend | ⏳ |
| penalty_arbitrer | feat/contract/disputes/penalty | ⏳ |
| create_milestone | feat/contract/disputes/milestone | ⏳ |
| approve_milestone | feat/contract/disputes/milestone | ⏳ |
| reject_milestone | feat/contract/disputes/milestone | ⏳ |
| withdraw_treasury | feat/contract/disputes/treasury | ⏳ |
| set_treasurer | feat/contract/disputes/treasury | ⏳ |
| update_treasury | feat/contract/disputes/treasury | ⏳ |

---

## 🛠️ Validación por tarea

```bash
anchor build
anchor test
```

Cada task debe:
1. Compilar sin warnings
2. Tests pasando
3. Commit con mensaje convencional

---

## 🔗 Reglas de negocio implementadas

**Disputas:**
- Stake: 2.5% cliente + 2.5% freelancer = 5% total
- El stake se paga al árbitro por su trabajo
- Áarbitro tiene 7 días para resolver
- Admin puede extender 7 días más
- Si el árbitro no resuelve: 5% penalty a tesorería + nuevo árbitro
- Áarbitro no puede ser cliente ni freelancer del job

**Milestones:**
- Creados por el cliente al crear el job
- Cada hito tiene monto y deadline
- Aprobación de hito = pago parcial al freelancer

**Treasury:**
- Recupera rent de PDAs cerradas
- Fee de entrada (5%) y salida (5%)
- Penalty de árbitros
- Admin/Treasurer pueden retirar

---

## 🔁 Relacionado con

- Epic #1 - Smart Contract - Trust Work Escrow v2

---

👷‍♂️ **Responsable**: @developer  
📂 **Entregables**: Disputes, Milestones, Treasury  
🔀 **Rama**: `feat/contract/disputes`  
📅 **Estado**: ✅ Completada