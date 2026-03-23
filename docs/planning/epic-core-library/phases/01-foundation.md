# 🏗️ Phase 1: SDK Foundation & Setup

**Contexto:**  
Esta tarea forma parte del Epic #2 - "Core Library (Rust SDK)".

---

## 📋 Descripción
Establish the fundamental SDK crate structure, Anchor client integration, and core infrastructure needed for Rust SDK development. This phase creates the foundation upon which all other SDK functionality will be built.

## 🎯 Objetivo
Create a working SDK crate with proper workspace integration, Anchor-generated types, comprehensive error handling, and CI-ready configuration. Set up build system that generates client types from v2 contract IDL.

---

## 🔧 Tasks asignadas a este módulo:

### 1.1 Workspace & Crate Setup
- [ ] Create `sdk/` directory with Cargo.toml as workspace member
- [ ] Configure crate metadata: name, version, description, license
- [ ] Add to root workspace Cargo.toml members list

### 1.2 Core Dependencies & Configuration  
- [ ] Set up dependencies: anchor-client, solana-sdk, anchor-lang, tokio
- [ ] Configure features: default tokio, optional serde support
- [ ] Set up lib.rs with module structure and public API exports

### 1.3 Anchor Client Generation
- [ ] Create build.rs script for IDL-based client generation
- [ ] Configure IDL path pointing to v2 contract program
- [ ] Generate and validate Anchor client types compile correctly

### 1.4 Error Handling Foundation
- [ ] Design comprehensive EscrowError enum in error.rs
- [ ] Map Anchor/Solana errors to custom error types with context
- [ ] Create Result type alias and error conversion traits

### 1.5 Type System Extensions
- [ ] Create types.rs extending Anchor-generated account types
- [ ] Add business logic validation methods to account structs
- [ ] Define custom enums matching contract ErrorCode patterns

### 1.6 PDA Infrastructure
- [ ] Establish pda.rs with seed constants matching v2 contract
- [ ] Create derivation functions for all account types (Config, User, Job, etc.)
- [ ] Set up caching infrastructure with lazy_static for performance

### 1.7 Development Tooling
- [ ] Configure clippy lints for Solana development best practices
- [ ] Set up rustfmt configuration matching project conventions
- [ ] Create basic CI checks: cargo check, clippy, fmt

### 1.8 Documentation Foundation
- [ ] Create comprehensive README.md with installation and basic usage
- [ ] Set up doc comments structure for public APIs
- [ ] Configure cargo doc generation with examples

---

## 📁 Convención de entregables para este módulo

```
sdk/
├── Cargo.toml                   # Complete crate manifest
├── build.rs                     # Anchor IDL client generation
├── src/
│   ├── lib.rs                  # Module organization + public exports  
│   ├── error.rs                # EscrowError enum + conversions
│   ├── types.rs                # Extended account types + enums
│   ├── pda.rs                  # PDA derivation + caching infrastructure
│   └── utils.rs                # Helper functions (placeholder)
├── README.md                    # Usage guide + installation
└── .gitignore                  # Rust-specific ignores

Root Changes:
├── Cargo.toml                   # Updated with sdk workspace member
```

---

## 🔀 Rama de este módulo

**Rama**: `feat/epic-core-library/phase-1`  
**Rama padre**: `feat/epic-core-library`  
**PR destino**: `feat/epic-core-library`

---

## ✅ Checklist de este módulo

| Task | Rama | Check |
|-|-|-|
| 1.1 Workspace Setup | `task/epic-core-library/phase-1/workspace-setup` | [ ] |
| 1.2 Dependencies | `task/epic-core-library/phase-1/dependencies` | [ ] |  
| 1.3 Anchor Client | `task/epic-core-library/phase-1/anchor-client` | [ ] |
| 1.4 Error Handling | `task/epic-core-library/phase-1/error-handling` | [ ] |
| 1.5 Type Extensions | `task/epic-core-library/phase-1/type-extensions` | [ ] |
| 1.6 PDA Infrastructure | `task/epic-core-library/phase-1/pda-infrastructure` | [ ] |
| 1.7 Dev Tooling | `task/epic-core-library/phase-1/dev-tooling` | [ ] |
| 1.8 Documentation | `task/epic-core-library/phase-1/documentation` | [ ] |

---

## 🔁 Relacionado con:

- Epic #2 - Core Library (Rust SDK)
- Requires: Completed v2 smart contract with stable IDL
- Enables: Phase 2 (Core Operations implementation)

---

👷‍♂️ **Responsable**: @davidcoachdev  
📂 **Entregables**: Working SDK crate foundation with Anchor integration  
🔀 **Rama**: `feat/epic-core-library/phase-1`  
📅 **Estado**: Ready for development