# 🏗️ Epic: Smart Contract - Trust Work Escrow v2

## 📋 Descripción

Desarrollar el smart contract en Anchor/Rust para el protocolo de escrow descentralizado Trust Work Escrow v2. El contrato gestionajobs, equipos, disputas y tesorería con 17 instrucciones.

## 🎯 Objetivo

Desplegar un contrato funcional en Solana Devnet con todas las instrucciones core necesarias para el hackathon.

---

## 📁 Fases del Epic

| Fase | Descripción | Estado |
|------|-------------|--------|
| [01-setup](./phases/01-setup.md) | Config, Users, Wallets | ⏳ |
| [02-jobs-teams](./phases/02-jobs-teams.md) | Jobs, Teams, Apply/Accept | ⏳ |
| [03-disputes](./phases/03-disputes.md) | Disputes, Milestones, Treasury | ⏳ |
| [04-tests-idl](./phases/04-tests-idl.md) | Tests, IDL, Documentación | ⏳ |

---

## 🔀 Rama de este Epic

**Rama**: `feat/contract`  
**Rama padre**: `main`  
**PR destino**: `main`

---

## ✅ Checklist de fases

| Fase | Rama | Estado |
|------|------|--------|
| 01-setup | feat/contract/setup | ⏳ |
| 02-jobs-teams | feat/contract/jobs-teams | ⏳ |
| 03-disputes | feat/contract/disputes | ⏳ |
| 04-tests-idl | feat/contract/tests-idl | ⏳ |

---

## 📝 Convenciones de commits

```bash
# Ejemplos
feat(contract): add initialize_config instruction
feat(contract): add create_user instruction
feat(contract): add create_job instruction
fix(contract): resolvePDA derivation error
test(contract): add initialize_config test
docs(contract): update account structs docs
```

---

## 🔗 Relacionado con

- Epic principal: Trust Work Escrow v2

---

👷‍♂️ **Responsable**: @developer  
🔀 **Rama madre**: `main`  
🎯 **Rama destino**: `main`  
📅 **Estado**: Por iniciar