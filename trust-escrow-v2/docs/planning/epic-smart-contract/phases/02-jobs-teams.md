# 🏗️ Fase 02: Jobs, Teams, Apply/Accept - Smart Contract

**Contexto:**  
Esta fase forma parte del Epic #1 - "Smart Contract - Trust Work Escrow v2".

---

## 📋 Descripción

Implementar la gestión de jobs, equipos y el flujo de aplicación/aceptación de trabajos.

## 🎯 Objetivo

Permitir crear jobs, gestionar equipos, y el flujo completo desde que un freelancer aplica hasta que es aceptado.

---

## 🔧 Tasks asignadas a esta fase

### Teams
- [ ] `feat(contract): add create_team instruction` - Crear equipo con owner y miembros
- [ ] `feat(contract): add update_team instruction` - Actualizar info del equipo
- [ ] `feat(contract): add add_member instruction` - Agregar miembro al equipo
- [ ] `feat(contract): add remove_member instruction` - Eliminar miembro del equipo
- [ ] `feat(contract): add update_member instruction` - Actualizar rol/porcentaje de miembro

### Jobs
- [ ] `feat(contract): add create_job instruction` - Crear job (título, descripción, monto, deadline)
- [ ] `feat(contract): add publish_job instruction` - Publicar y fondear job (105% = monto + 5% fee)

### Aplicación
- [ ] `feat(contract): add apply_to_job instruction` - Freelancer aplica al job
- [ ] `feat(contract): add accept_application instruction` - Cliente acepta freelancer/equipo
- [ ] `feat(contract): add reject_application instruction` - Cliente rechaza aplicación
- [ ] `feat(contract): add withdraw_application instruction` - Freelancer retira su aplicación

### Trabajo
- [ ] `feat(contract): add submit_work instruction` - Freelancer entrega trabajo
- [ ] `feat(contract): add approve_work instruction` - Cliente aprueba → pago automático
- [ ] `feat(contract): add reject_work instruction` - Cliente rechaza → iniciar disputa
- [ ] `feat(contract): add cancel_job instruction` - Cancelar job (solo si sin freelancer asignado)
- [ ] `feat(contract): add auto_approve_job instruction` - Auto-aprobar después de 7 días sin respuesta

---

## 📁 Convenciones de entregables

```
programs/trust-escrow-v2/
├── src/
│   ├── instructions/
│   │   ├── team.rs                # create_team, update_team, add_member, remove_member
│   │   ├── job.rs                 # create_job, publish_job, submit_work, approve_work
│   │   ├── application.rs         # apply_to_job, accept_application, reject_application
│   │   └── work.rs                # submit_work, approve_work, reject_work, cancel_job
│   ├── state/
│   │   ├── team.rs                # Team account
│   │   ├── job.rs                 # Job account + Application
│   │   └── mod.rs
├── tests/
│   └── jobs-teams.spec.ts         # Tests de integración
└── docs/
    └── instructions.md
```

---

## 🔀 Rama de esta fase

**Rama**: `feat/contract/jobs-teams`  
**Rama padre**: `feat/contract`  
**PR destino**: `feat/contract`

---

## ✅ Checklist de tareas

| Task | Rama | Estado |
|------|------|--------|
| create_team | feat/contract/jobs-teams/team | ⏳ |
| update_team | feat/contract/jobs-teams/team | ⏳ |
| add_member | feat/contract/jobs-teams/team | ⏳ |
| remove_member | feat/contract/jobs-teams/team | ⏳ |
| update_member | feat/contract/jobs-teams/team | ⏳ |
| create_job | feat/contract/jobs-teams/job | ⏳ |
| publish_job | feat/contract/jobs-teams/job | ⏳ |
| apply_to_job | feat/contract/jobs-teams/application | ⏳ |
| accept_application | feat/contract/jobs-teams/application | ⏳ |
| reject_application | feat/contract/jobs-teams/application | ⏳ |
| withdraw_application | feat/contract/jobs-teams/application | ⏳ |
| submit_work | feat/contract/jobs-teams/work | ⏳ |
| approve_work | feat/contract/jobs-teams/work | ⏳ |
| reject_work | feat/contract/jobs-teams/work | ⏳ |
| cancel_job | feat/contract/jobs-teams/work | ⏳ |
| auto_approve_job | feat/contract/jobs-teams/work | ⏳ |

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

- Job publicado requiere 105% del monto (monto + 5% fee entrada)
- Freelancer no puede aplicar a sus propios jobs
- Team puede aplicar si el owner lo autoriza
- Auto-aprobar después de 7 días de submitted sin respuesta
- Cancelar solo posible si no hay freelancer asignado

---

## 🔁 Relacionado con

- Epic #1 - Smart Contract - Trust Work Escrow v2

---

👷‍♂️ **Responsable**: @developer  
📂 **Entregables**: Teams, Jobs, Aplicaciones, Trabajo  
🔀 **Rama**: `feat/contract/jobs-teams`  
📅 **Estado**: Por iniciar