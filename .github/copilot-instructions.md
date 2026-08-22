# Trust Work Escrow — Copilot Instructions

## 🏗️ ¿Qué es este proyecto?

**Trust Work Escrow** es una plataforma de pagos escrow descentralizada para freelancers y clientes, construida sobre Solana. Incluye un smart contract (Anchor), una librería compartida (escrow-core), un CLI (Clap) y un TUI (Ratatui).

**Program ID:** `5gu5JCSpB8MKyJzhXpGaCt8SruAMnRD6cTPbwPX6JTYo`

El proyecto corre en un **Dev Container** preconfigurado con todas las herramientas.

## 🧱 Herramientas disponibles

| Herramienta    | Versión   | Uso                                 |
| -------------- | --------- | ----------------------------------- |
| **Rust**       | 1.89      | Smart contracts, CLI, TUI, Libs      |
| **Solana CLI** | 3.0.15    | Interacción con la blockchain       |
| **Anchor**     | 0.32.1    | Framework para programas Solana     |
| **Node.js**    | 20.x LTS  | Tests de integración (TypeScript)    |
| **Yarn**       | latest    | Gestión de paquetes                 |
| **Ubuntu**     | 24.04 LTS | Sistema base (GLIBC 2.39 requerido) |

## 📁 Estructura del proyecto

```
/
├── .devcontainer/       # Configuración del Dev Container
│   ├── Dockerfile       # Ubuntu 24.04 + Rust + Solana + Anchor + Node
│   ├── devcontainer.json
│   ├── README.md
│   └── scripts/
│       └── post-create.sh
│
├── .github/             # Configuración GitHub
│   ├── copilot-instructions.md   # ← Este archivo
│   ├── dependabot.yml
│   └── workflows/
│       └── devcontainer-prebuild.yml
│
├── docs/                # Documentación completa (9 archivos)
│
├── trust-escrow/        # Proyecto Anchor principal
│   ├── programs/trust-escrow/src/  # Smart contract (12 instrucciones)
│   ├── escrow-core/     # Librería compartida (13 ops, 14 tests)
│   ├── cli/             # CLI (clap + escrow-core, 13 comandos)
│   ├── tui/             # TUI (ratatui + escrow-core, 4 roles, 4 temas)
│   ├── tests/           # Tests de integración (TypeScript, 23 tests)
│   └── migrations/
│
├── scripts/             # Scripts de deploy y backup
├── .gitignore
├── LICENSE              # MIT License
└── README.md            # Instrucciones del proyecto
```

## ✍️ Convenciones de código

### Rust

- **Funciones**: `snake_case` → `create_account()`
- **Structs/Enums**: `PascalCase` → `AccountState`
- **Constantes**: `SCREAMING_SNAKE_CASE`
- **Errores Anchor**: `#[error_code]` enum
- **Formato**: `rustfmt` (automático)
- **Linter**: `clippy`
- **Nunca** usar `unwrap()` en producción — usar `?` o `Result`

### TypeScript

- **Variables/funciones**: `camelCase`
- **Componentes React**: `PascalCase`
- **Formato**: Prettier (automático)

## 🔐 Seguridad — Principios para Solana/Anchor

1. **Validar inputs**: `require!()` en cada instrucción
2. **Verificar firmantes**: `Signer<'info>` para autorizaciones
3. **Constraints de cuentas**: `has_one`, `seeds`, `bump`
4. **Sin secretos hardcodeados**: usar `.env`
5. **Errores personalizados**: `#[error_code]`, nunca panics
6. **CHECK comments**: documentar `AccountInfo` sin deserializar

## 🧪 Testing — Patrones

- **Happy path**: el caso normal funciona
- **Sad path**: errores esperados
- **Validación de PDA**: seeds derivan correctamente

```bash
anchor test           # Tests on-chain
cargo test            # Tests unitarios Rust
```

## 🔌 Puertos del entorno

| Puerto | Uso típico         |
| ------ | ------------------ |
| 3000   | Frontend (Next.js) |
| 8080   | Backend API (Axum) |
| 8899   | Solana RPC HTTP    |
| 8900   | Solana WebSocket   |
| 9900   | Solana Faucet      |

## 🛠 Comandos del proyecto

```bash
# Verificar herramientas
rustc --version && solana --version && anchor --version && node --version

# Solana
solana config set --url localhost
solana-test-validator --reset
solana airdrop 5

# Smart Contract
anchor build
anchor test              # Tests de integración (23 tests)
anchor deploy
anchor keys list

# escrow-core
cd escrow-core && cargo test    # 14 tests unitarios

# CLI
cd cli && cargo build
cargo run -- --help

# TUI
cd tui && cargo build
cargo run

# Calidad
cargo clippy --workspace
cargo fmt

# Node.js (para tests)
yarn install
```

## 📝 Commits (Conventional Commits)

```
feat: ✨ nueva funcionalidad
fix: 🐛 corrección de bug
test: 🧪 agregar tests
docs: 📖 documentación
refactor: ♻️ reestructuración
chore: 🔧 mantenimiento
```

## 🌿 Ramas

- `main` → producción
- `feature/nombre` → nuevas funcionalidades
- `fix/nombre` → correcciones
- `refactor/nombre` → reestructuración
- `docs/nombre` → documentación

## ⚠️ Pre-commit checklist

1. Código compila sin errores
2. Todos los tests pasan
3. `cargo clippy` sin warnings (si aplica)
4. Tests nuevos para código nuevo
5. Documentación actualizada

## 🚫 Nunca hacer

- No usar `unwrap()` en producción
- No hardcodear Program IDs (usar `declare_id!`)
- No ignorar el bump del PDA
- No crear cuentas sin `Space` calculado
- No commitear sin tests
- No mezclar lógica de negocio con handlers HTTP
- No tocar `.devcontainer/` sin autorización
- No duplicar lógica de Solana en CLI/TUI (usar `escrow-core`)

## 📌 Contexto del proyecto

### Smart Contract (12 instrucciones)
`initialize_config`, `create_job`, `deposit_funds`, `accept_job`, `submit_work`, `approve_work`, `reject_work`, `raise_dispute`, `resolve_dispute`, `cancel_job`, `pause_program`, `unpause_program`

### CLI (13 subcomandos)
`init`, `create`, `deposit`, `accept`, `submit`, `approve`, `reject`, `raise-dispute`, `resolve-dispute`, `cancel`, `show`, `pause`, `unpause`

### TUI (4 roles)
Admin, Client, Freelancer, Arbiter — cada uno con menú contextual de operaciones

### escrow-core (librería compartida)
Toda lógica de interacción con Solana centralizada. CLI y TUI dependen de ella vía `path = "../escrow-core"`.

### Workspace Cargo
- `trust-escrow/Cargo.toml`: `members = ["programs/*"]`, `exclude = ["cli", "tui", "escrow-core"]`
- CLI, TUI y escrow-core tienen su propio `[workspace]` o quedan excluídos

### Certificación
Proyecto para **WayLearn Solana Developer Certification**.
