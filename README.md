# Trust Work Escrow 🛡️

> Sistema de pagos escrow para freelancer-cliente en Solana.

## 📋 Descripción

Plataforma descentralizada donde clientes depositan fondos en un vault seguro (PDA) y los freelancers reciben el pago solo cuando el trabajo es aprobado o tras resolución de disputas por un árbitro.

## 🛠️ Tecnologías

| Tecnología | Uso |
|------------|-----|
| **Anchor** | Smart Contract (Rust) |
| **Rust + Clap** | CLI (Interfaz de terminal) |
| **Solana** | Blockchain (Devnet) |
| **SPL Tokens** | Pagos (USDC/SOL) |

## 🚀 Inicio Rápido

### Prerrequisitos

- Rust 1.65+
- Solana CLI 1.16+
- Anchor 0.30+

### Instalación

```bash
# 1. Clonar repositorio
git clone <repo-url>
cd trust-work-escrow

# 2. Instalar dependencias Anchor
npm install

# 3. Build del smart contract
anchor build

# 4. Build de la CLI
cargo build --manifest-path cli/Cargo.toml
```

### Configurar Solana (Devnet)

```bash
solana config set --url devnet
solana airdrop 2
```

## 💻 Uso de la CLI

```bash
# Help
cargo run --manifest-path cli/Cargo.toml -- --help

# Cliente: Crear trabajo
cargo run --manifest-path cli/Cargo.toml -- create "App Web" --amount 2 --arbiter <addr>

# Freelancer: Aceptar trabajo
cargo run --manifest-path cli/Cargo.toml -- accept <job_id>

# Freelancer: Entregar trabajo
cargo run --manifest-path cli/Cargo.toml -- submit <job_id>

# Cliente: Aprobar trabajo
cargo run --manifest-path cli/Cargo.toml -- approve <job_id>

# Abrir disputa
cargo run --manifest-path cli/Cargo.toml -- dispute <job_id> --reason "No entregado"

# Árbitro: Resolver disputa
cargo run --manifest-path cli/Cargo.toml -- resolve <job_id> --winner client

# Listar trabajos
cargo run --manifest-path cli/Cargo.toml -- list
```

## 📁 Estructura del Proyecto

```
trust-work-escrow/
├── programs/          # Smart Contract (Anchor)
│   └── escrow/
├── cli/              # CLI (Rust + Clap)
├── docs/             # Documentación
│   └── PROYECTO.md   # Documentación completa
└── README.md
```

## 📖 Documentación

- [docs/PROYECTO.md](./docs/PROYECTO.md) - Documentación completa del proyecto

## ✅ Certificación

Proyecto para **WayLearn Solana Developer Certification**.

Requisitos cumplidos:
- ✅ Proyecto libre
- ✅ Desarrollado en Rust
- ✅ CRUD + PDA implementado
- ✅ Documentación clara

## 📄 Licencia

MIT
