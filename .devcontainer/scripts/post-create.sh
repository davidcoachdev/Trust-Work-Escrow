#!/usr/bin/env bash
# ═══════════════════════════════════════════════════════════════
# post-create.sh
#
# Se ejecuta UNA SOLA VEZ después de crear el contenedor.
# Solo verifica que las herramientas estén instaladas.
# NO crea carpetas, archivos ni instala dependencias.
#
# Si una herramienta falla, muestra warning pero NO aborta.
# ═══════════════════════════════════════════════════════════════

# No usar set -e: queremos reportar TODAS las herramientas
# que faltan, no abortar en la primera.

export PATH="$HOME/.avm/bin:$HOME/.cargo/bin:$HOME/.local/share/solana/install/active_release/bin:$HOME/.opencode/bin:$PATH"

GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

ok()   { echo -e "${GREEN}  ✅ $*${NC}"; }
warn() { echo -e "${YELLOW}  ⚠️  $*${NC}"; }
err()  { echo -e "${RED}  ❌ $*${NC}"; }

ERRORS=0

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
        err "$name no encontrado"
        ERRORS=$((ERRORS + 1))
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
check "Surfpool"   "surfpool --version"
check "GitHub CLI" "gh --version"
check "OpenCode"   "opencode --version"

echo ""

if [ "$ERRORS" -gt 0 ]; then
    echo "═══════════════════════════════════════════════════"
    warn "$ERRORS herramienta(s) no encontrada(s)."
    echo "  Reconstruye con: Dev Containers → Rebuild Without Cache"
    echo "═══════════════════════════════════════════════════"
    echo ""
    # Exit 0 para no bloquear el inicio del contenedor
    # El usuario puede reconstruir manualmente si necesita la herramienta
    exit 0
else
    echo "═══════════════════════════════════════════════════"
    echo -e "${GREEN}  ✅ Contenedor listo. Todas las herramientas OK.${NC}"
    echo "═══════════════════════════════════════════════════"
    echo ""
    echo "  Ejemplos para comenzar:"
    echo "  ─────────────────────────────────────────────────"
    echo "  anchor init mi-proyecto         → Proyecto Anchor"
    echo "  cargo init backend              → API Rust"
    echo "  npx create-next-app frontend    → Frontend Next.js"
    echo ""
fi

# ── Alias para validador accesible externamente ─────────────────
if ! grep -q "solana-validator" ~/.bashrc 2>/dev/null; then
    echo "" >> ~/.bashrc
    echo "# Alias para solana-test-validator con bind externo" >> ~/.bashrc
    echo "alias solana-validator='solana-test-validator --bind-address 0.0.0.0'" >> ~/.bashrc
fi
