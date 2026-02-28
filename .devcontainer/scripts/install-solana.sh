#!/bin/bash
set -euo pipefail

# ═══════════════════════════════════════════════════════════════════
# Instala Solana CLI
# Ejecutar como usuario no-root (vscode)
# ═══════════════════════════════════════════════════════════════════

SOLANA_VERSION="${1:-stable}"

echo "☀️ Instalando Solana CLI (${SOLANA_VERSION})..."

sh -c "$(curl -sSfL https://release.anza.xyz/${SOLANA_VERSION}/install)"

export PATH="$HOME/.local/share/solana/install/active_release/bin:$PATH"

echo "🔑 Generando keypair por defecto..."
mkdir -p "$HOME/.config/solana"
solana-keygen new --no-bip39-passphrase --silent \
    --outfile "$HOME/.config/solana/id.json"

echo "⚙️ Configurando URL a localhost..."
solana config set --url localhost

echo "✅ Solana $(solana --version) instalado"
