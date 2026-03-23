# 🏗️ Fase 01: Config, Users, Wallets - Smart Contract

**Contexto:**  
Esta fase forma parte del Epic #1 - "Smart Contract - Trust Work Escrow v2".

---

## 📋 Descripción

Implementar las instrucciones de configuración del programa, gestión de usuarios y vinculación de wallets múltiples.

## 🎯 Objetivo

Permitir inicializar el programa, crear usuarios con perfiles y vincular hasta 5 wallets por usuario.

---

## 🔧 Tasks asignadas a esta fase

### Config
- [x] `feat(contract): add initialize_config instruction` - Inicializar configuración global del programa
- [x] `feat(contract): add update_config instruction` - Actualizar parámetros de configuración
- [x] `feat(contract): add pause instruction` - Pausar el programa (admin)
- [x] `feat(contract): add unpause instruction` - Reanudar el programa (admin)

### Users
- [x] `feat(contract): add create_user instruction` - Crear perfil de usuario
- [x] `feat(contract): add update_user instruction` - Actualizar perfil (username, bio, skills)

### Wallets (Multi-wallet)
- [x] `feat(contract): add add_wallet instruction` - Agregar wallet secundaria (sign-message verification)
- [x] `feat(contract): add set_active_wallet instruction` - Cambiar wallet activa
- [x] `feat(contract): add remove_wallet instruction` - Eliminar wallet secundaria

---

## 📁 Convenciones de entregables

```
programs/trust-escrow-v2/
├── src/
│   ├── lib.rs                     # Entry point + instructions
│   ├── instructions/
│   │   ├── mod.rs
│   │   ├── config.rs              # initialize_config, update_config, pause, unpause
│   │   ├── user.rs                # create_user, update_user
│   │   └── wallet.rs              # add_wallet, set_active_wallet, remove_wallet
│   ├── state/
│   │   ├── mod.rs
│   │   ├── config.rs              # Config account
│   │   └── user.rs                # User account + Wallet
│   └── errors.rs                  # Custom errors
├── tests/
│   └── config-user.spec.ts        # Tests de integración
└── docs/
    └── instructions.md             # Documentación educativa
```

---

## 🔀 Rama de esta fase

**Rama**: `feat/contract/setup`  
**Rama padre**: `feat/contract`  
**PR destino**: `feat/contract`

---

## ✅ Checklist de tareas

| Task | Rama | Estado |
|------|------|--------|
| initialize_config | feat/contract/setup/config | ✅ |
| update_config | feat/contract/setup/config | ✅ |
| pause | feat/contract/setup/config | ✅ |
| unpause | feat/contract/setup/config | ✅ |
| create_user | feat/contract/setup/user | ✅ |
| update_user | feat/contract/setup/user | ✅ |
| add_wallet | feat/contract/setup/wallet | ✅ |
| set_active_wallet | feat/contract/setup/wallet | ✅ |
| remove_wallet | feat/contract/setup/wallet | ✅ |

---

## 🛠️ Validación por tarea

```bash
# Compilar
anchor build

# Testear
anchor test
```

Cada task debe:
1. Compilar sin warnings
2. Tests pasando
3. Commit con mensaje convencional

---

## 🔁 Relacionado con

- Epic #1 - Smart Contract - Trust Work Escrow v2

---

👷‍♂️ **Responsable**: @developer  
📂 **Entregables**: Config, Users, Wallets  
🔀 **Rama**: `feat/contract/setup`  
📅 **Estado**: ✅ Completada