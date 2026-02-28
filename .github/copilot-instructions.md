# Solana Dev Container — Copilot Instructions

## 🏗️ ¿Qué es este contenedor?

Este es un **Dev Container template** preconfigurado para desarrollar proyectos sobre **Solana**. Viene vacío y listo para que inicies tu proyecto desde cero.

El contenedor incluye todas las herramientas necesarias: Rust, Solana CLI, Anchor Framework y Node.js.

## 🧱 Herramientas disponibles

| Herramienta    | Versión   | Uso                                 |
| -------------- | --------- | ----------------------------------- |
| **Rust**       | stable    | Smart contracts, APIs, CLIs         |
| **Solana CLI** | v2.1.21   | Interacción con la blockchain       |
| **Anchor**     | 0.31.x    | Framework para programas Solana     |
| **Node.js**    | 20.x LTS  | Frontend, tests, tooling            |
| **Yarn**       | latest    | Gestión de paquetes                 |
| **Ubuntu**     | 24.04 LTS | Sistema base (GLIBC 2.39 requerido) |

## 📁 Estructura del template

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
├── .gitignore           # Exclusiones para Git
├── LICENSE              # MIT License
├── README.md            # Instrucciones del template
└── _backup/             # Proyecto anterior (eliminar cuando quieras)
```

## 🚀 Cómo crear un proyecto nuevo

### Proyecto Anchor (Smart Contract Solana)

```bash
anchor init nombre-proyecto
```

### Proyecto Rust puro (API con Axum)

```bash
cargo init backend --name backend
cargo add axum tokio --features tokio/full
```

### Frontend Next.js

```bash
npx create-next-app@latest frontend --typescript --tailwind --app --src-dir
```

### Monorepo completo

```bash
anchor init nombre-proyecto && cd nombre-proyecto
cargo init backend --name backend
npx create-next-app@latest frontend --typescript --tailwind --app --src-dir
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

## 🛠 Comandos útiles

```bash
# Verificar herramientas
rustc --version && solana --version && anchor --version && node --version

# Solana
solana config set --url localhost
solana-test-validator --reset
solana airdrop 2

# Anchor
anchor build
anchor test
anchor deploy
anchor keys list

# Rust
cargo build
cargo test
cargo clippy --workspace
cargo fmt

# Node.js
yarn install
yarn dev
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

## 💡 Nota

Cuando el usuario cree un proyecto nuevo, **actualiza este archivo** con los detalles específicos del proyecto: dominio, estructura, instrucciones del programa, endpoints, etc.
