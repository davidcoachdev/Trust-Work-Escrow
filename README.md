# 🚀 Dev Container Template — Solana + Anchor + Rust + Node.js

> **Contenedor listo para desarrollar proyectos sobre Solana.**
> Abre este repositorio en un Dev Container y empieza a crear.

---

## 📦 ¿Qué incluye este contenedor?

| Herramienta    | Versión   | Descripción                          |
| -------------- | --------- | ------------------------------------ |
| **Ubuntu**     | 24.04 LTS | Sistema base                         |
| **Rust**       | stable    | Lenguaje para smart contracts y APIs |
| **Solana CLI** | v2.1.21   | Herramientas de línea de comando     |
| **Anchor**     | 0.31.x    | Framework para programas Solana      |
| **Node.js**    | 20.x LTS  | Runtime para frontend y tests        |
| **Yarn**       | latest    | Gestor de paquetes                   |

## 🏁 Inicio rápido

### 1. Verificar que todo funciona

```bash
rustc --version
solana --version
anchor --version
node --version
yarn --version
```

### 2. Configurar Solana para desarrollo local

```bash
solana config set --url localhost
solana-keygen new --no-bip39-passphrase    # Solo si no tienes keypair
```

---

## 📋 Guías para crear proyectos

### Opción A: Proyecto Anchor (Smart Contract Solana)

```bash
# Crear proyecto Anchor desde cero
anchor init mi-proyecto
cd mi-proyecto

# Compilar
anchor build

# Ejecutar tests
anchor test
```

**Estructura generada:**

```
mi-proyecto/
├── Anchor.toml          # Configuración del proyecto
├── Cargo.toml           # Workspace Rust
├── package.json         # Dependencias JS para tests
├── programs/            # Smart contracts (Rust)
│   └── mi-proyecto/
│       └── src/lib.rs
├── tests/               # Tests de integración (TypeScript)
│   └── mi-proyecto.ts
├── app/                 # Cliente (opcional)
└── migrations/          # Scripts de despliegue
```

### Opción B: Proyecto Rust puro (API, CLI, librería)

```bash
# Binario
cargo init mi-api --name mi-api

# Librería
cargo init mi-lib --lib --name mi-lib
```

**Frameworks recomendados para API:**

- **Axum** — async, moderno, ecosistema Tokio
- **Actix-web** — alto rendimiento
- **Rocket** — ergonómico, macros declarativas

```bash
# Ejemplo: API con Axum
cargo init backend --name backend
cd backend
cargo add axum tokio --features tokio/full
cargo add serde --features derive
cargo add serde_json
```

### Opción C: Frontend (Next.js + React)

```bash
npx create-next-app@latest frontend --typescript --tailwind --app --src-dir
cd frontend
yarn dev    # Puerto 3000
```

### Opción D: Monorepo completo (Anchor + Backend + Frontend)

```bash
# 1. Inicializar Anchor
anchor init mi-proyecto
cd mi-proyecto

# 2. Agregar backend Rust
cargo init backend --name backend
# Agregar al Cargo.toml workspace:
# [workspace]
# members = ["programs/*", "backend"]

# 3. Agregar frontend
npx create-next-app@latest frontend --typescript --tailwind --app --src-dir

# 4. Configurar Yarn workspaces en package.json:
# "workspaces": ["app", "frontend"]
```

---

## 🔌 Puertos disponibles

| Puerto | Uso típico         |
| ------ | ------------------ |
| 3000   | Frontend (Next.js) |
| 8080   | Backend API (Axum) |
| 8899   | Solana RPC HTTP    |
| 8900   | Solana WebSocket   |
| 9900   | Solana Faucet      |

## 🛠 Comandos útiles

```bash
# === Solana ===
solana-test-validator --reset        # Validador local
solana balance                       # Ver balance
solana airdrop 2                     # Obtener SOL de prueba

# === Anchor ===
anchor build                         # Compilar programa
anchor test                          # Build + test
anchor deploy                        # Desplegar a la red configurada
anchor keys list                     # Ver Program IDs

# === Rust ===
cargo build                          # Compilar proyecto Rust
cargo test                           # Ejecutar tests
cargo clippy                         # Linter
cargo fmt                            # Formatear código

# === Node.js ===
yarn install                         # Instalar dependencias
yarn dev                             # Servidor de desarrollo
npx ts-mocha -p tsconfig.json tests/**/*.ts   # Tests TypeScript
```

## 📝 Convenciones recomendadas

### Commits (Conventional Commits)

```
feat: ✨ nueva funcionalidad
fix: 🐛 corrección de bug
test: 🧪 agregar tests
docs: 📖 documentación
refactor: ♻️ reestructuración
chore: 🔧 mantenimiento
```

### Ramas

- `main` — producción
- `feature/nombre` — nuevas funcionalidades
- `fix/nombre` — correcciones

---

## 📂 Archivos del template

```
/
├── .devcontainer/       # Configuración del Dev Container
├── .github/             # GitHub Actions, Dependabot, Copilot
├── .gitignore           # Archivos excluidos de Git
├── LICENSE              # MIT License
└── README.md            # ← Este archivo
```

---

## 📄 Licencia

[MIT](LICENSE)
