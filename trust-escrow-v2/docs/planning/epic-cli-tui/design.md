# Design: Epic #3 - CLI/TUI Applications

## Technical Approach

Hybrid strategy creating new v2 applications leveraging proven UX patterns from legacy implementations. Implements dual-interface architecture with shared core module for Epic #2 SDK integration. CLI provides hierarchical commands for all 51 SDK operations, while TUI offers real-time dashboard interfaces for complex workflow visualization.

Architecture follows Rust workspace pattern with CLI (Clap framework) and TUI (Ratatui 0.28+) as separate binaries sharing common utilities and configuration. Direct Epic #2 SDK integration ensures type-safe operations with comprehensive error handling for professional user experience.

## Architecture Decisions

### Decision: Rust Workspace Structure
**Choice**: Separate CLI and TUI as workspace members under `trust-escrow-v2/`
**Alternatives considered**: Monolithic application with feature flags, separate repositories
**Rationale**: Workspace allows shared dependencies while maintaining clean separation. Follows existing project structure with programs and sdk as workspace members. Enables independent development and testing of CLI vs TUI concerns.

### Decision: Clap v4 for CLI Framework
**Choice**: Clap v4 with derive macros for command structure
**Alternatives considered**: Structopt, manual argument parsing, Commander
**Rationale**: Clap v4 is mature, well-documented, supports hierarchical commands out of the box. Derive macros reduce boilerplate. Already used in legacy CLI implementation, proven pattern.

### Decision: Ratatui for TUI Framework
**Choice**: Ratatui 0.28+ with crossterm backend
**Alternatives considered**: Cursive, FTXUI (via FFI), custom implementation
**Rationale**: Ratatui is actively maintained fork of tui-rs with better performance. Crossterm provides cross-platform terminal support. Legacy TUI already uses compatible version, migration path available.

### Decision: Shared Core Module Architecture
**Choice**: Common `core` module with config, client wrappers, and utilities
**Alternatives considered**: Duplicate implementations, separate core crate
**Rationale**: Eliminates code duplication for SDK integration, configuration management, and error handling. Single source of truth for business logic while keeping presentation separate.

### Decision: Async Runtime Strategy
**Choice**: Tokio runtime with async client operations, sync TUI with channels
**Alternatives considered**: Async-std, fully sync with blocking calls
**Rationale**: Epic #2 SDK is built on tokio async patterns. TUI requires sync main loop but can communicate via channels with async background tasks for real-time updates.

### Decision: Configuration Management
**Choice**: Hierarchical config (CLI args > ENV vars > config file > defaults)
**Alternatives considered**: CLI args only, environment variables only
**Rationale**: Professional CLI behavior expects multiple config sources. Supports both automation (ENV) and interactive use (CLI args). Config file enables profiles for different networks.

## Data Flow

```
User Input ──→ CLI/TUI Interface ──→ Shared Core Module ──→ Epic #2 SDK ──→ Solana
     │                │                      │                    │           │
     │                │                      │                    │           │
     └──────── Config ─┴─ Error Handling ────┴── Type Validation ──┴── Network ┘
```

**CLI Flow**: Command parsing → Core validation → SDK operation → Result formatting → Output
**TUI Flow**: Event loop → State management → Async operations via channels → Display updates → User navigation

## File Changes

| File | Action | Description |
|------|--------|-------------|
| `trust-escrow-v2/Cargo.toml` | Modify | Add cli and tui as workspace members |
| `trust-escrow-v2/cli/Cargo.toml` | Create | CLI dependencies: clap, trust-escrow-sdk, tokio, anyhow |
| `trust-escrow-v2/cli/src/main.rs` | Create | CLI entry point and command routing |
| `trust-escrow-v2/cli/src/commands/` | Create | Hierarchical command implementations |
| `trust-escrow-v2/cli/src/core/` | Create | Shared utilities and SDK wrappers |
| `trust-escrow-v2/tui/Cargo.toml` | Create | TUI dependencies: ratatui, crossterm, tokio |
| `trust-escrow-v2/tui/src/main.rs` | Create | TUI entry point and application loop |
| `trust-escrow-v2/tui/src/ui/` | Create | UI components and layout management |
| `trust-escrow-v2/tui/src/app/` | Create | Application state and event handling |
| `trust-escrow-v2/shared/` | Create | Common configuration and utilities |

## Interfaces / Contracts

```rust
// Shared configuration interface
pub struct EscrowConfig {
    pub network: Network,
    pub wallet_path: PathBuf,
    pub rpc_url: String,
    pub commitment: CommitmentConfig,
}

// Common result type for operations
pub type AppResult<T> = Result<T, AppError>;

// CLI command trait for consistent interface
pub trait Command {
    async fn execute(&self, client: &CofreClient) -> AppResult<()>;
}

// TUI state management
pub struct AppState {
    pub current_user: Option<User>,
    pub jobs: Vec<Job>,
    pub notifications: Vec<Notification>,
    pub network_status: NetworkStatus,
}

// Event system for TUI real-time updates
pub enum AppEvent {
    NetworkUpdate(NetworkStatus),
    JobUpdate(Job),
    UserInput(KeyEvent),
    Tick,
}
```

## Testing Strategy

| Layer | What to Test | Approach |
|-------|-------------|----------|
| Unit | Command parsing, state management, utilities | Mock SDK client, property-based testing |
| Integration | SDK operations, configuration loading | devnet/localnet integration, test fixtures |
| E2E | Complete user workflows via both interfaces | Automated CLI execution, TUI interaction testing |
| Demo | Live devnet transactions, error scenarios | Manual testing protocol, demo scripts |

## Migration / Rollout

**Phase 1**: CLI core commands (user, job basic operations)
**Phase 2**: CLI advanced features (teams, disputes, milestones) 
**Phase 3**: TUI basic dashboard with static data
**Phase 4**: TUI real-time updates and advanced navigation
**Phase 5**: Demo preparation and polish

No data migration required - both applications are new implementations consuming existing SDK.

## Performance Targets

- **CLI Response**: Commands < 2s initial response, progress indicators for blockchain ops
- **TUI Refresh**: Real-time updates within 5s of blockchain events
- **Startup Time**: Both applications < 3s cold start on reasonable hardware
- **Memory Usage**: < 50MB RSS for normal operations
- **Network Efficiency**: Batch operations where possible, intelligent caching

## Open Questions

- [ ] Should TUI support multiple simultaneous user sessions?
- [ ] How detailed should transaction progress indicators be in CLI?
- [ ] What level of offline functionality should be supported?
- [ ] Should we implement custom themes or stick to default terminal colors?