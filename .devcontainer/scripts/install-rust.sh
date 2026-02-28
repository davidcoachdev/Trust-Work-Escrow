#!/bin/bash
set -euo pipefail

# ═══════════════════════════════════════════════════════════════════
# Instala Rust + componentes esenciales
# Ejecutar como usuario no-root (vscode)
# ═══════════════════════════════════════════════════════════════════

echo "🦀 Instalando Rust..."

curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
    | sh -s -- -y --default-toolchain stable --profile default

source "$HOME/.cargo/env"

echo "📦 Instalando componentes rustfmt y clippy..."
rustup component add rustfmt clippy

echo "✅ Rust $(rustc --version) instalado"
echo "✅ Cargo $(cargo --version)"
