# Guía de Instalación - Trust Work Escrow

## 📋 Requisitos del Sistema

### Requisitos Mínimos

| Requisito | Versión Mínima |
|-----------|----------------|
| **OS** | Ubuntu 20.04+, macOS 11+, Windows 10+ (WSL2) |
| **RAM** | 8 GB |
| **Espacio** | 10 GB libres |
| **CPU** | 4 núcleos |

### Herramientas Requeridas

| Herramienta | Versión | Comando de verificación |
|-------------|---------|--------------------------|
| **Rust** | 1.65+ | `rustc --version` |
| **Cargo** | 1.65+ | `cargo --version` |
| **Solana CLI** | 1.16+ | `solana --version` |
| **Anchor** | 0.30+ | `anchor --version` |
| **Node.js** | 18+ | `node --version` |
| **Yarn** | 1.22+ | `yarn --version` |

## 🛠️ Instalación

### 1. Instalar Rust

```bash
# Descargar e instalar Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Recargar entorno
source $HOME/.cargo/env

# Verificar instalación
rustc --version
cargo --version
```

### 2. Instalar Solana CLI

```bash
# Instalar Solana CLI (versión estable)
sh -c "$(curl -sSfL "https://release.solana.com/stable/install")"

# Recargar entorno
export PATH="$HOME/.local/share/solana/install/active_release/bin:$PATH"

# Verificar instalación
solana --version
```

### 3. Instalar Anchor

```bash
# Instalar Anchor CLI
npm install -g @project-serum/anchor-cli

# O desde cargo
cargo install anchor-cli

# Verificar instalación
anchor --version
```

### 4. Instalar Node.js (opcional, para frontend)

```bash
# Instalar nvm (Node Version Manager)
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash
source ~/.bashrc

# Instalar Node.js 20 LTS
nvm install 20
nvm use 20

# Verificar instalación
node --version
npm --version
```

### 5. Instalar Yarn

```bash
# Instalar Yarn globalmente
npm install -g yarn

# Verificar instalación
yarn --version
```

## ⚙️ Configuración Inicial

### 1. Configurar Solana (Devnet)

```bash
# Configurar para devnet
solana config set --url devnet

# Verificar configuración
solana config get
```

### 2. Crear Wallet (si no tienes)

```bash
# Generar nuevo keypair
solana-keygen new --no-bip39-passphrase

# Ver dirección de wallet
solana address
```

### 3. Obtener SOL de Prueba

```bash
# Solicitar airdrop (2 SOL)
solana airdrop 2

# Verificar balance
solana balance
```

### 4. Configurar Anchor

```bash
# Crear archivo de configuración Anchor
anchor init trust-escrow
cd trust-escrow

# Configurar wallet en Anchor.toml
# (se hace automáticamente con solana config)
```

## 📥 Clonar y Construir el Proyecto

### 1. Clonar Repositorio

```bash
# Clonar el proyecto
git clone <repo-url>
cd trust-work-escrow
```

### 2. Instalar Dependencias

```bash
# Instalar dependencias npm
npm install

# O si usas yarn
yarn install
```

### 3. Build del Smart Contract

```bash
# Compilar el programa Anchor
anchor build

# O simplemente
anchor build
```

### 4. Build de la CLI

```bash
# Compilar la CLI
cargo build --manifest-path cli/Cargo.toml

# O en modo release
cargo build --release --manifest-path cli/Cargo.toml
```

## 🧪 Verificar Instalación

### Verificar Herramientas

```bash
# Script de verificación (si existe)
bash scripts/verify-tools.sh

# O verificar manualmente
echo "=== Herramientas ===" && \
rustc --version && \
cargo --version && \
solana --version && \
anchor --version && \
node --version && \
yarn --version && \
echo "=== Todas las herramientas instaladas ==="
```

### Verificar Proyecto

```bash
# Ver estructura del proyecto
ls -la

# Ver que Anchor build funcione
anchor build

# Ver que CLI build funcione
cargo build --manifest-path cli/Cargo.toml -- --help
```

## 🚀 Primeros Pasos

### Desplegar a Devnet

```bash
# 1. Configurar para devnet
solana config set --url devnet

# 2. Build
anchor build

# 3. Desplegar
anchor deploy

# 4. Obtener Program ID
anchor keys list
```

### Usar la CLI

```bash
# Ver ayuda
cargo run --manifest-path cli/Cargo.toml -- --help

# Crear un trabajo
cargo run --manifest-path cli/Cargo.toml -- create "Mi Trabajo" --amount 2 --arbiter <ARBITRO_ADDR>

# Listar trabajos
cargo run --manifest-path cli/Cargo.toml -- list

# Ver trabajo específico
cargo run --manifest-path cli/Cargo.toml -- show <JOB_ID>
```

## 🔧 Solución de Problemas

### Error: "No such file or directory"

```bash
# Agregar Rust al PATH
export PATH="$HOME/.cargo/bin:$PATH"

# Agregar Solana al PATH
export PATH="$HOME/.local/share/solana/install/active_release/bin:$PATH"

# Agregar permanentemente al ~/.bashrc
echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.bashrc
echo 'export PATH="$HOME/.local/share/solana/install/active_release/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

### Error: "anchor: command not found"

```bash
# Reinstalar anchor
npm uninstall -g @project-serum/anchor-cli
npm install -g @project-serum/anchor-cli

# O instalar desde cargo
cargo install anchor-cli
```

### Error: "Insufficient funds"

```bash
# Solicitar más SOL
solana airdrop 2

# Ver balance
solana balance
```

### Error: "Program not found"

```bash
# Verificar que el programa está desplegado
solana program show <PROGRAM_ID>

# O desplegar de nuevo
anchor deploy
```

### Error: "Connection refused"

```bash
# Verificar conexión a RPC
curl https://api.devnet.solana.com -X POST -H "Content-Type: application/json" -d '{"jsonrpc":"2.0","id":1,"method":"getVersion"}'

# Cambiar RPC
solana config set --url https://api.devnet.solana.com
```

## 📦 Estructura de Archivos

```
trust-work-escrow/
├── .devcontainer/           # Configuración Dev Container
├── .github/                 # GitHub Actions
├── cli/                     # CLI Rust
│   ├── Cargo.toml
│   └── src/
│       └── main.rs
├── docs/                    # Documentación
│   ├── ARQUITECTURA.md
│   ├── CLI.md
│   ├── PROYECTO.md
│   ├── SMARTCONTRACT.md
│   └── INSTALL.md
├── programs/                # Smart Contracts Anchor
│   └── escrow/
│       ├── Cargo.toml
│       └── src/
│           └── lib.rs
├── tests/                   # Tests
├── Anchor.toml
├── package.json
└── README.md
```

## 🔄 Actualizar Proyecto

```bash
# Pull latest changes
git pull origin main

# Reconstruir
anchor build
cargo build --manifest-path cli/Cargo.toml
```

## 📚 Recursos Adicionales

- [Documentación de Anchor](https://www.anchor-lang.com/)
- [Solana Docs](https://docs.solana.com/)
- [Solana Cookbook](https://solanacookbook.com/)
- [Rust Book](https://doc.rust-lang.org/book/)

## ❓ Soporte

Si tienes problemas:

1. Revisa los errores comunes arriba
2. Consulta la documentación en `docs/`
3. Abre un issue en GitHub
