# Epic #2: Core Library (Rust SDK) - Proposal

**Epic ID**: #24 | **Status**: In Progress | **Date**: 2026-03-23

## Intent

Replace the legacy escrow-core TypeScript library with a modern Rust SDK that provides seamless integration with the Trust Work Escrow v2 smart contract. This SDK will serve as the foundation for all client applications (CLI, TUI, backend services) and provide a production-ready, type-safe interface for interacting with the Solana-based escrow protocol.

## Business Context

The hackathon deadline (March 23, 2026) requires a complete ecosystem showcasing the Trust Work Escrow v2 platform. While Epic #1 (Smart Contract) is complete with 31 instructions and 1,485 lines of production-ready Solana code, the ecosystem needs client-side libraries that match the smart contract's sophistication and reliability.

## Scope

### In Scope ✅

**Core SDK Components**:
- Rust client library with full smart contract integration
- Anchor IDL-based type generation and validation
- All 31 instruction wrappers with proper error handling
- Account state management and PDA derivation utilities
- Transaction building with manual construction (Anchor client issues)
- Comprehensive error handling and logging
- Educational documentation with concept explanations

**Target Architecture**:
- `trust-escrow-v2/sdk/` - Core Rust library (workspace member)
- Full replacement of `trust-escrow/escrow-core/` TypeScript library
- Production-ready code quality with proper testing
- Integration examples for each major operation

**Integration Points**:
- Smart contract: `trust_escrow_v2` (31 instructions, Program ID: `TesT3XPqD3WFFVTY4BTwZ3sJpY7C7hF3Z6K2oX3i7jB`)
- IDL file: `trust-escrow-v2/target/idl/trust_escrow_v2.json`
- Anchor framework v0.32+ compatibility

### Out of Scope ❌

- CLI application (separate future epic)
- TUI application (separate future epic) 
- Backend services (separate future epic)
- Advanced caching or offline functionality
- GUI applications or web interfaces
- Performance optimization beyond MVP requirements

## Affected Areas

```
trust-escrow-v2/
├── sdk/                    # 🆕 NEW - Core Rust SDK
│   ├── src/
│   │   ├── lib.rs         # Main library exports
│   │   ├── client.rs      # CofreClient main interface
│   │   ├── types.rs       # Generated types from IDL
│   │   ├── accounts.rs    # Account derivation utilities
│   │   ├── instructions/  # Instruction builders (31 instructions)
│   │   ├── error.rs       # Error handling
│   │   └── constants.rs   # Protocol constants
│   ├── Cargo.toml         # Dependencies and metadata
│   ├── build.rs           # IDL integration build script
│   └── README.md          # Usage documentation
│
├── trust-escrow/          # 📦 LEGACY (to be replaced)
│   └── escrow-core/       # TypeScript library (deprecated)
│
└── Cargo.toml             # 🔧 MODIFIED - Add sdk workspace member
```

## Approach

### Technical Strategy

1. **Rust-First Architecture**: Complete departure from TypeScript to leverage Rust's type safety, performance, and integration with Solana tooling

2. **Manual Transaction Building**: Due to Anchor client trait bound issues with `dyn Signer`, implement manual transaction construction while maintaining type safety

3. **IDL-Driven Development**: Use the completed smart contract's IDL to generate types and ensure perfect compatibility

4. **Educational Documentation**: Each module includes conceptual explanations to serve as learning material for Solana/escrow development

### Implementation Strategy

**4-Phase Development**:
- **Phase 1**: Foundation & Setup (8 tasks) ✅ **COMPLETE**
- **Phase 2**: Core Operations (8 tasks) 🔄 **IN PROGRESS** 
- **Phase 3**: Advanced Features (8 tasks) ⏳ **PENDING**
- **Phase 4**: Testing & Documentation (8 tasks) ⏳ **PENDING**

**Branching Strategy**:
```
feat/epic-core-library          # Epic branch
├── phase-1-foundation-setup    # ✅ COMPLETE
├── phase-2-core-operations     # 🔄 CURRENT 
├── phase-3-advanced-features   # ⏳ PENDING
└── phase-4-testing-docs        # ⏳ PENDING
```

## Success Criteria

### Functional Requirements

- [ ] Complete SDK with 31 instruction wrappers
- [ ] Full account state management (Config, User, Job, Team, etc.)
- [ ] PDA derivation utilities for all account types
- [ ] Comprehensive error handling with detailed messages
- [ ] Transaction building and submission capabilities
- [ ] Integration examples for major workflows

### Quality Requirements

- [ ] 100% compatibility with smart contract IDL
- [ ] Comprehensive error coverage and logging
- [ ] Educational documentation for each module
- [ ] Clean API design following Rust conventions
- [ ] Production-ready code quality

### Integration Requirements

- [ ] Seamless Anchor framework integration
- [ ] Solana-web3 compatibility
- [ ] Easy installation via Cargo
- [ ] Clear upgrade path from legacy escrow-core

## Risks

### High Risk ⚠️

**Anchor Client Integration Complexity**: Anchor's trait bound requirements with `Arc<dyn Signer>` have already required manual transaction building workarounds. This increases implementation complexity but maintains functionality.

**Timeline Pressure**: Hackathon deadline March 23, 2026 limits implementation and testing time. Mitigation: Focus on core functionality over advanced features.

### Medium Risk ⚡

**IDL Synchronization**: Changes to smart contract must be reflected in SDK. Mitigation: Use build.rs to check IDL consistency.

**Error Handling Completeness**: Solana transaction errors can be complex. Mitigation: Implement comprehensive error mapping and user-friendly messages.

### Low Risk ✅

**Rust Learning Curve**: Team has established Rust expertise from smart contract development.

**Documentation Quality**: Templates and patterns established from Epic #1 documentation.

## Dependencies

### Upstream Dependencies ✅

- **Epic #1 Complete**: Smart contract with 31 instructions deployed and tested
- **IDL Generated**: `trust_escrow_v2.json` with full type definitions
- **Program ID Stable**: `TesT3XPqD3WFFVTY4BTwZ3sJpY7C7hF3Z6K2oX3i7jB`

### External Dependencies

```toml
[dependencies]
anchor-client = "0.32.0"      # Anchor framework integration
anchor-lang = "0.32.0"        # Type definitions and utilities  
solana-sdk = "2.1.0"          # Core Solana functionality
solana-client = "2.1.0"       # RPC client and transactions
tokio = { version = "1.0", features = ["full"] }
anyhow = "1.0"                # Error handling
thiserror = "1.0"             # Custom error types
log = "0.4"                   # Logging framework
```

## Rollback Plan

### Phase-Level Rollback
Each phase maintains separate branches, allowing isolated rollback:
- Phase issues detected → rollback to previous phase branch
- Epic issues detected → rollback to `main` branch

### Complete Rollback
- Revert to legacy `trust-escrow/escrow-core/` TypeScript library
- Continue development with TypeScript until post-hackathon
- Epic #2 becomes post-hackathon priority

### Mitigation Strategy
- Maintain legacy escrow-core compatibility during transition
- Phase-by-phase validation prevents complete implementation loss
- Manual transaction building provides fallback from Anchor client issues

## Timeline

| Phase | Duration | Status | Target Completion |
|-------|----------|--------|------------------|
| Phase 1: Foundation | 1 day | ✅ Complete | March 23, 16:00 |
| Phase 2: Core Operations | 1 day | 🔄 In Progress | March 23, 20:00 |
| Phase 3: Advanced Features | 0.5 day | ⏳ Pending | March 23, 22:00 |
| Phase 4: Testing & Documentation | 0.5 day | ⏳ Pending | March 23, 23:30 |

**Total Duration**: 3 days | **Epic Deadline**: March 23, 2026, 23:30 UTC

---

**Next Action**: Begin Phase 2 Core Operations implementation following detailed task breakdown in `tasks.md`.

**Related Documents**:
- Epic Specifications: `specs.md`
- Technical Design: `design.md`  
- Task Breakdown: `tasks.md`
- GitHub Issues: #24, #26, #27, #28