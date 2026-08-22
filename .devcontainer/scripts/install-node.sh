#!/bin/bash
set -euo pipefail

# ═══════════════════════════════════════════════════════════════════
# Instala Node.js LTS + utilidades globales
# ═══════════════════════════════════════════════════════════════════

NODE_MAJOR="${1:-20}"

echo "📦 Instalando Node.js v${NODE_MAJOR}.x..."

# Obtener la última versión de la rama LTS
NODE_FULL_VERSION=$(curl -fsSL "https://nodejs.org/dist/latest-v${NODE_MAJOR}.x/SHASUMS256.txt" \
    | grep linux-x64.tar.xz \
    | head -1 \
    | awk '{print $2}' \
    | sed 's/node-\(v[0-9.]*\)-.*/\1/')

echo "📥 Descargando Node.js ${NODE_FULL_VERSION}..."

curl -fsSL "https://nodejs.org/dist/${NODE_FULL_VERSION}/node-${NODE_FULL_VERSION}-linux-x64.tar.xz" \
    | tar -xJ -C /usr/local --strip-components=1

echo "📦 Instalando yarn, typescript, ts-node..."
npm install -g yarn typescript ts-node --force

echo "✅ Node.js $(node --version) instalado"
echo "✅ npm $(npm --version)"
echo "✅ yarn $(yarn --version)"
