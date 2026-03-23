# 🏗️ Phase 2: Core Operations Implementation

**Contexto:**  
Esta tarea forma parte del Epic #2 - "Core Library (Rust SDK)".

---

## 📋 Descripción
Implement the CofreClient high-level interface and core instruction builders for user management, team operations, and job lifecycle. Build the primary SDK operations that CLI, TUI, and Backend consumers will use for essential escrow functionality.

## 🎯 Objetivo
Create a fully functional CofreClient with type-safe operations for users, teams, and jobs. Provide instruction builders with validation for all core v2 contract instructions, enabling complete escrow workflow automation.

---

## 🔧 Tasks asignadas a este módulo:

### 2.1 CofreClient Foundation
- [ ] Implement CofreClient::new() with RPC connection management
- [ ] Add keypair/wallet integration with proper signing patterns
- [ ] Configure commitment levels and connection pooling

### 2.2 User Management Operations
- [ ] Implement create_user with username/bio validation  
- [ ] Add update_user operation with field-specific updates
- [ ] Create get_user_by_pubkey with account deserialization
- [ ] Add user wallet association and management methods

### 2.3 Team Management Operations  
- [ ] Implement create_team with name/description validation
- [ ] Add add_team_member with role-based access control
- [ ] Create remove_team_member with authorization checks
- [ ] Implement get_team with member list resolution

### 2.4 Job Lifecycle - Creation & Application
- [ ] Implement create_job with comprehensive parameter validation
- [ ] Add apply_to_job with proposal submission and validation  
- [ ] Create accept_application with freelancer selection logic
- [ ] Add get_job_applications for application management

### 2.5 Job Lifecycle - Execution & Completion
- [ ] Implement start_job with status transition validation
- [ ] Add complete_job with work submission capabilities  
- [ ] Create review_job with rating and feedback systems
- [ ] Implement finalize_payment with escrow release logic

### 2.6 Instruction Builder Infrastructure
- [ ] Create instruction builders in instructions.rs for all implemented operations
- [ ] Add parameter validation before instruction creation
- [ ] Implement proper account resolution and PDA derivation integration
- [ ] Add transaction building utilities with fee estimation

### 2.7 PDA Integration & Caching
- [ ] Integrate PDA derivation helpers for users, teams, jobs  
- [ ] Implement PDA caching for performance optimization
- [ ] Add PDA validation against contract expectations
- [ ] Create bulk PDA derivation for batch operations

### 2.8 Transaction Management Utilities
- [ ] Implement transaction building with proper fee handling
- [ ] Add transaction sending with confirmation logic  
- [ ] Create retry mechanisms for failed transactions
- [ ] Implement transaction simulation for pre-flight validation

---

## 📁 Convención de entregables para este módulo

```
sdk/src/
├── client.rs                    # CofreClient implementation
│   ├── new() + connection management
│   ├── User operations (create, update, get)  
│   ├── Team operations (create, manage members)
│   ├── Job lifecycle (create → apply → complete)
│   └── Transaction utilities
├── instructions.rs              # Instruction builders
│   ├── User instruction builders with validation
│   ├── Team instruction builders  
│   ├── Job instruction builders
│   └── Transaction building utilities
├── pda.rs (enhanced)           # PDA integration
│   ├── Enhanced caching for core operations
│   ├── Bulk derivation methods
│   └── Validation utilities
└── lib.rs (updated)            # Export CofreClient + instruction builders

examples/ (new)
├── user_management.rs           # User operation examples
├── team_workflow.rs            # Team management examples  
└── job_lifecycle.rs            # Complete job workflow example
```

---

## 🔀 Rama de este módulo

**Rama**: `feat/epic-core-library/phase-2`  
**Rama padre**: `feat/epic-core-library`  
**PR destino**: `feat/epic-core-library`

---

## ✅ Checklist de este módulo

| Task | Rama | Check |
|-|-|-|
| 2.1 CofreClient Foundation | `task/epic-core-library/phase-2/cofre-client` | [ ] |
| 2.2 User Operations | `task/epic-core-library/phase-2/user-operations` | [ ] |  
| 2.3 Team Operations | `task/epic-core-library/phase-2/team-operations` | [ ] |
| 2.4 Job Creation & Application | `task/epic-core-library/phase-2/job-creation` | [ ] |
| 2.5 Job Execution & Completion | `task/epic-core-library/phase-2/job-completion` | [ ] |
| 2.6 Instruction Builders | `task/epic-core-library/phase-2/instruction-builders` | [ ] |
| 2.7 PDA Integration | `task/epic-core-library/phase-2/pda-integration` | [ ] |
| 2.8 Transaction Management | `task/epic-core-library/phase-2/transaction-management` | [ ] |

---

## 🔁 Relacionado con:

- Epic #2 - Core Library (Rust SDK)  
- Requires: Phase 1 (SDK Foundation) completed
- Enables: Phase 3 (Advanced Features) and basic CLI/TUI integration
- Provides: Core operations for all escrow workflows

---

👷‍♂️ **Responsable**: @davidcoachdev  
📂 **Entregables**: Complete CofreClient with core operations and instruction builders  
🔀 **Rama**: `feat/epic-core-library/phase-2`  
📅 **Estado**: Ready for development (pending Phase 1 completion)