# 🏗️ Phase 1: Foundation Setup

**Contexto:**  
Esta tarea forma parte del Epic #3 - "CLI/TUI Applications".

---

## 📋 Descripción
Establish the fundamental workspace structure and shared utilities for CLI and TUI applications. This phase creates the foundational infrastructure needed for both applications including configuration management, SDK integration wrappers, and error handling.

## 🎯 Objetivo
Create a working workspace with CLI and TUI crates, shared utilities, comprehensive configuration management, and Epic #2 SDK integration. Set up build system that enables both applications to consume all 51 SDK operations.

---

## 🔧 Tasks asignadas a este módulo:

### 1.1 Workspace & Crate Setup
- [ ] Update `trust-escrow-v2/Cargo.toml` to add `cli` and `tui` workspace members
- [ ] Create `trust-escrow-v2/cli/Cargo.toml` with clap, tokio, anyhow, trust-escrow-sdk dependencies
- [ ] Create `trust-escrow-v2/tui/Cargo.toml` with ratatui, crossterm, tokio dependencies
- [ ] Create `trust-escrow-v2/shared/Cargo.toml` for common utilities

### 1.2 Shared Configuration Infrastructure
- [ ] Implement `shared/src/config.rs` with `EscrowConfig` struct and hierarchical config loading
- [ ] Implement `shared/src/error.rs` with `AppError` enum and error handling utilities
- [ ] Implement `shared/src/client.rs` wrapper for Epic #2 SDK integration

### 1.3 CLI Foundation
- [ ] Create basic CLI entry point `cli/src/main.rs` with clap command structure
- [ ] Set up module structure with placeholder commands for user, job, milestone, payment, dispute

### 1.4 Development Tooling
- [ ] Configure clippy lints for applications
- [ ] Set up rustfmt configuration matching project conventions
- [ ] Create basic CI checks integration

---

## 📁 Convención de entregables para este módulo

```
trust-escrow-v2/
├── Cargo.toml                      # Updated with cli, tui, shared workspace members
├── cli/
│   ├── Cargo.toml                  # CLI dependencies: clap, tokio, anyhow
│   └── src/
│       ├── main.rs                 # CLI entry point with command structure
│       ├── commands/               # Module placeholders
│       └── lib.rs                  # Module organization
├── tui/
│   ├── Cargo.toml                  # TUI dependencies: ratatui, crossterm, tokio
│   └── src/
│       ├── main.rs                 # TUI entry point placeholder
│       └── lib.rs                  # Module organization
└── shared/
    ├── Cargo.toml                  # Shared utilities crate
    └── src/
        ├── lib.rs                  # Shared module exports
        ├── config.rs               # EscrowConfig + hierarchical loading
        ├── error.rs                # AppError enum + conversions
        └── client.rs               # Epic #2 SDK wrapper
```

---

## 🔀 Rama de este módulo

**Rama**: `feat/epic-cli-tui/phase-1`  
**Rama padre**: `feat/epic-cli-tui`  
**PR destino**: `feat/epic-cli-tui`

---

## ✅ Checklist de este módulo

| Task | Rama | Check |
|-|-|-|
| 1.1 Workspace Setup | `task/epic-cli-tui/phase-1/workspace-setup` | [ ] |
| 1.2 Shared Infrastructure | `task/epic-cli-tui/phase-1/shared-infrastructure` | [ ] |  
| 1.3 CLI Foundation | `task/epic-cli-tui/phase-1/cli-foundation` | [ ] |
| 1.4 Dev Tooling | `task/epic-cli-tui/phase-1/dev-tooling` | [ ] |

---

## 🔁 Relacionado con:

- Epic #3 - CLI/TUI Applications
- Requires: Epic #2 SDK (✅ Complete - 51 operations available)
- Enables: Phase 2 (CLI Core Implementation)
- Enables: Phase 3 (TUI Foundation)

---

👷‍♂️ **Responsable**: @davidcoachdev  
📂 **Entregables**: Working workspace with CLI, TUI, and shared crates foundation  
🔀 **Rama**: `feat/epic-cli-tui/phase-1`  
📅 **Estado**: Ready for development