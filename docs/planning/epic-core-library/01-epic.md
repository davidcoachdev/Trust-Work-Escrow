# 🚀 Epic #2: Core Library (Rust SDK)

## 📋 Descripción
Replace legacy escrow-core library (1,318 lines) with modern Rust SDK crate that leverages the completed v2 smart contract. Provide type-safe, high-level operations through Anchor-generated client types for CLI, TUI, and Backend consumption as a publishable crates.io package.

## 🎯 Objetivo
Build a production-ready Rust SDK that supports all 31 v2 contract instructions while providing multi-wallet architecture support and comprehensive developer experience for Trust Work Escrow integrations.

> ✨ **Contexto**: This Epic replaces the outdated escrow-core library with a Rust-first SDK that matches v2 smart contract patterns. Critical for hackathon deadline March 23, 2026, enabling CLI/TUI/Backend development in subsequent epics.

---

## 🧩 Sub-Issues (Módulos)

### Phase 1: Foundation & Setup
**Issue**: #TBD - SDK Foundation & Setup  
**Objetivo**: Establish SDK crate structure, dependencies, and Anchor client integration  
**Tasks**: 8 foundational tasks (Cargo.toml, lib.rs, build.rs, error handling)  
**Entregables**: Working SDK crate with generated Anchor types

### Phase 2: Core Operations  
**Issue**: #TBD - Core Operations Implementation  
**Objetivo**: Implement user, team, and job lifecycle operations with CofreClient  
**Tasks**: 8 core implementation tasks (user management, job lifecycle, instruction builders)  
**Entregables**: Complete CRUD operations for users, teams, jobs

### Phase 3: Advanced Features
**Issue**: #TBD - Advanced Features & Integration  
**Objetivo**: Add dispute handling, milestones, treasury operations, and integration patterns  
**Tasks**: 8 advanced feature tasks (disputes, milestones, treasury, wallet patterns)  
**Entregables**: Full feature parity with v2 contract capabilities

### Phase 4: Testing & Documentation
**Issue**: #TBD - Testing & Documentation  
**Objetivo**: Comprehensive test coverage, API documentation, and crates.io publication readiness  
**Tasks**: 8 testing/documentation tasks (unit tests, integration tests, examples, publication)  
**Entregables**: >90% test coverage, complete documentation, published crate

---

## 📁 Convención de entregables

```
trust-escrow-v2/sdk/
├── Cargo.toml                    # SDK crate manifest with dependencies
├── build.rs                      # Anchor IDL client generation
├── src/
│   ├── lib.rs                   # Public API exports and organization
│   ├── client.rs                # CofreClient high-level operations
│   ├── instructions.rs          # All 31 instruction builders
│   ├── pda.rs                   # PDA derivation utilities with caching
│   ├── types.rs                 # Account types extending Anchor-generated
│   ├── error.rs                 # Custom error types and conversions
│   └── utils.rs                 # Helper functions and utilities
├── examples/                     # CLI, TUI, Backend integration patterns
├── tests/                       # Integration test suite
└── README.md                    # SDK documentation and usage guide
```

---

## 🔀 Estrategia de ramas y PRs

1. **Rama madre del proyecto**: `feat/epic-core-library`

2. **Rama por módulo**: `feat/epic-core-library/phase-N` (creada desde `feat/epic-core-library`)

3. **Ramas por task**: `task/epic-core-library/phase-N/task-description` (creada desde su módulo)

4. **Flujo de merges**:  
   - `task/...` → **PR a su rama de módulo**  
   - Cuando todas las tasks de un módulo estén listas → **PR del módulo a `feat/epic-core-library`**  
   - Cuando todos los módulos estén listos → **PR de `feat/epic-core-library` → `main`**

---

## ✅ Checklist final

| De | A | Check |
|-|-|-|
| `task/epic-core-library/phase-1/*` | `feat/epic-core-library/phase-1` | [ ] Foundation & Setup completed |
| `task/epic-core-library/phase-2/*` | `feat/epic-core-library/phase-2` | [ ] Core Operations implemented |  
| `task/epic-core-library/phase-3/*` | `feat/epic-core-library/phase-3` | [ ] Advanced Features complete |
| `task/epic-core-library/phase-4/*` | `feat/epic-core-library/phase-4` | [ ] Testing & Documentation finished |
| `feat/epic-core-library/phase-1` | `feat/epic-core-library` | [ ] Phase 1 integration verified |
| `feat/epic-core-library/phase-2` | `feat/epic-core-library` | [ ] Phase 2 integration verified |
| `feat/epic-core-library/phase-3` | `feat/epic-core-library` | [ ] Phase 3 integration verified |  
| `feat/epic-core-library/phase-4` | `feat/epic-core-library` | [ ] Phase 4 integration verified |
| `feat/epic-core-library` | `main` | [ ] Epic #2 complete and ready for production |

---

👷‍♂️ **Responsable**: @davidcoachdev  
🔀 **Rama madre**: `feat/epic-core-library`  
🎯 **Rama final**: `main`  
📅 **Estado**: Planning - Ready for implementation