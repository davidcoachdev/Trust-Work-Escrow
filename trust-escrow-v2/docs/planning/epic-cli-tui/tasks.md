# Tasks: Epic #3 - CLI/TUI Applications

## Phase 1: Foundation Setup

- [ ] 1.1 Update `trust-escrow-v2/Cargo.toml` to add `cli` and `tui` workspace members
- [ ] 1.2 Create `trust-escrow-v2/cli/Cargo.toml` with clap, tokio, anyhow, trust-escrow-sdk dependencies
- [ ] 1.3 Create `trust-escrow-v2/tui/Cargo.toml` with ratatui, crossterm, tokio dependencies  
- [ ] 1.4 Create `trust-escrow-v2/shared/Cargo.toml` for common utilities
- [ ] 1.5 Implement `shared/src/config.rs` with `EscrowConfig` struct and hierarchical config loading
- [ ] 1.6 Implement `shared/src/error.rs` with `AppError` enum and error handling utilities
- [ ] 1.7 Implement `shared/src/client.rs` wrapper for Epic #2 SDK integration
- [ ] 1.8 Create basic CLI entry point `cli/src/main.rs` with clap command structure

## Phase 2: CLI Core Implementation  

- [ ] 2.1 Implement `cli/src/commands/user.rs` with create, add-wallet, and list subcommands
- [ ] 2.2 Implement `cli/src/commands/job.rs` with create, list, apply, and view subcommands
- [ ] 2.3 Implement `cli/src/commands/milestone.rs` with create, submit, and complete subcommands
- [ ] 2.4 Implement `cli/src/commands/payment.rs` with process and dispute subcommands
- [ ] 2.5 Implement `cli/src/commands/config.rs` with network switching and wallet management
- [ ] 2.6 Add comprehensive help system and command discovery in all CLI modules
- [ ] 2.7 Implement error handling with clear, actionable messages in CLI commands
- [ ] 2.8 Add progress indicators for blockchain operations in CLI

## Phase 3: TUI Foundation

- [ ] 3.1 Create TUI entry point `tui/src/main.rs` with crossterm terminal setup
- [ ] 3.2 Implement `tui/src/app/state.rs` with `AppState` struct and state management
- [ ] 3.3 Implement `tui/src/app/events.rs` with `AppEvent` enum and event loop
- [ ] 3.4 Create `tui/src/ui/layout.rs` for three-panel dashboard layout
- [ ] 3.5 Implement `tui/src/ui/components/` with reusable UI widgets (job list, user info, notifications)
- [ ] 3.6 Create role-specific dashboards in `tui/src/ui/dashboards/` (freelancer, client, arbiter)
- [ ] 3.7 Implement keyboard navigation and help system in TUI
- [ ] 3.8 Add async background task communication via channels for real-time updates

## Phase 4: Advanced Features

- [ ] 4.1 Implement real-time transaction monitoring in TUI with progress indicators
- [ ] 4.2 Add job application notifications and auto-refresh in TUI dashboards
- [ ] 4.3 Implement contextual actions and interactive job browsing in TUI
- [ ] 4.4 Add milestone interaction workflows with detailed views in TUI
- [ ] 4.5 Implement network status monitoring and display in both CLI and TUI
- [ ] 4.6 Add comprehensive configuration management (network switching, wallet profiles)
- [ ] 4.7 Implement error recovery and retry mechanisms for network issues
- [ ] 4.8 Add transaction history and account balance views

## Phase 5: Integration & Testing

- [ ] 5.1 Write unit tests for all CLI commands with mock SDK client
- [ ] 5.2 Write integration tests for core user workflows on devnet/localnet
- [ ] 5.3 Test complete job lifecycle (create → apply → milestone → payment) via CLI
- [ ] 5.4 Test TUI real-time updates and navigation with live blockchain data  
- [ ] 5.5 Implement E2E testing scenarios from specifications
- [ ] 5.6 Test error handling scenarios (network failures, invalid operations)
- [ ] 5.7 Performance testing for startup time and memory usage
- [ ] 5.8 Validate all 51 SDK operations accessible through both interfaces

## Phase 6: Demo Preparation

- [ ] 6.1 Create demo scripts for end-to-end job workflow demonstration
- [ ] 6.2 Prepare interactive demo mode for judge testing
- [ ] 6.3 Test live devnet transactions with visual feedback
- [ ] 6.4 Create demo data setup and environment configuration
- [ ] 6.5 Prepare error handling demonstrations
- [ ] 6.6 Polish UI/UX for professional presentation
- [ ] 6.7 Create user documentation and help content
- [ ] 6.8 Final demo rehearsal and environment validation