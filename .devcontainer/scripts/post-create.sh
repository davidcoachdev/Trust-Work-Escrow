#!/bin/bash
# ═══════════════════════════════════════════════════════════════
# post-create.sh
#
# Se ejecuta UNA SOLA VEZ después de crear el contenedor.
# Solo verifica que las herramientas estén instaladas.
# NO crea carpetas, archivos ni instala dependencias.
# ═══════════════════════════════════════════════════════════════
set -e

export PATH="$HOME/.avm/bin:$HOME/.cargo/bin:$HOME/.local/share/solana/install/active_release/bin:$PATH"

GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m'

ok()  { echo -e "${GREEN}✅ $*${NC}"; }
err() { echo -e "${RED}❌ $*${NC}"; exit 1; }

echo ""
echo "═══════════════════════════════════════════════════"
echo "  Solana Dev Container — verificando herramientas"
echo "═══════════════════════════════════════════════════"
echo ""

check() {
    local name="$1"
    local cmd="$2"
    if version=$(eval "$cmd" 2>/dev/null); then
        ok "$name: $version"
    else
        err "$name no encontrado. Reconstruye la imagen con 'Dev Containers: Rebuild Container'."
    fi
}

check "Node.js"    "node --version"
check "npm"        "npm --version"
check "yarn"       "yarn --version"
check "TypeScript" "tsc --version"
check "Rust"       "rustc --version"
check "Cargo"      "cargo --version"
check "Solana"     "solana --version"
check "Anchor"     "anchor --version"

echo ""
echo "═══════════════════════════════════════════════════"
echo "  ✅ Contenedor listo. Empieza a crear tu proyecto."
echo "═══════════════════════════════════════════════════"
echo ""
echo "  Ejemplos para comenzar:"
echo "  ─────────────────────────────────────────────────"
echo "  anchor init mi-proyecto         → Proyecto Anchor"
echo "  cargo init backend              → API Rust"
echo "  npx create-next-app frontend    → Frontend Next.js"
echo ""
