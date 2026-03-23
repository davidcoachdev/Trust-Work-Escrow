# 📦 Epic: Core Library - Trust Work Escrow v2

## 📋 Descripción

Desarrollar la librería SDK en Rust para Trust Work Escrow v2 que reemplaza el legacy `trust-escrow/escrow-core/`. El SDK proporciona una interfaz tipo-segura y de alto nivel para interactuar con el smart contract v2.

**🔄 EN PROGRESO**: Phase 1 completo (SDK Foundation), Phase 2 en implementación (Core Operations).

## 🎯 Objetivo

Crear un SDK completo en Rust que permita integración fácil con el smart contract Trust Work Escrow v2 desde aplicaciones cliente, CLI, TUI y backend.

**🎯 META**: SDK funcional con 8 operaciones core, documentación educativa, y ejemplos comprensivos para el hackathon deadline.

---

## 📁 Fases del Epic

| Fase | Descripción | Estado |
|------|-------------|--------|
| [01-foundation](./phases/01-foundation.md) | SDK Foundation & Setup | ✅ |
| [02-core-operations](./phases/02-core-operations.md) | Core Escrow Operations | 🔄 |
| [03-advanced-features](./phases/03-advanced-features.md) | Advanced Features & Integration | ⏳ |
| [04-testing-docs](./phases/04-testing-docs.md) | Testing & Documentation | ⏳ |

---

## 🔀 Rama de este Epic

**Rama**: `feat/epic-core-library`  
**Rama padre**: `main`  
**PR destino**: `main`

---

## ✅ Checklist de fases

| Fase | Rama | Estado |
|------|------|--------|
| 01-foundation | phase-1-foundation-setup | ✅ |
| 02-core-operations | phase-2-core-operations | 🔄 |
| 03-advanced-features | phase-3-advanced-features | ⏳ |
| 04-testing-docs | phase-4-testing-docs | ⏳ |

---

## 📝 Convenciones de commits

```bash
# Ejemplos
feat(sdk): implement create_escrow operation
feat(sdk): add error handling for invalid amounts
feat(sdk): implement PDA caching system
fix(sdk): correct IDL path for relocated SDK
test(sdk): add unit tests for core operations
docs(sdk): add usage examples for escrow operations
```

---

## 🔗 Relacionado con

- Epic principal: Trust Work Escrow v2
- Epic #1: Smart Contract (COMPLETADO) ✅
- GitHub Issues: #24 (Epic), #25-28 (Phases)

---

👷‍♂️ **Responsable**: @developer  
🔀 **Rama madre**: `main`  
🎯 **Rama destino**: `main`  
📅 **Estado**: 🔄 En progreso (Phase 2/4)