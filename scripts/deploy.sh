#!/usr/bin/env bash
# ═══════════════════════════════════════════════════════════════
# deploy.sh — Deploy automatizado con verificaciones pre-flight
#
# Uso:
#   ./scripts/deploy.sh                  # detecta cluster del env
#   ./scripts/deploy.sh devnet           # deploy a devnet
#   ./scripts/deploy.sh mainnet-beta     # deploy a mainnet (requiere --confirm)
#   ./scripts/deploy.sh mainnet-beta --confirm   # deploy a mainnet confirmado
#
# Pre-flight checks:
#   1. Herramientas instaladas (solana, anchor, cargo)
#   2. Saldo suficiente en la wallet
#   3. Build limpio sin errores
#   4. Tests pasan al 100%
#   5. cargo clippy sin warnings
#   6. Backup automático de keypairs antes de deploy
#
# ⚠️  Para mainnet se requiere --confirm como medida de seguridad
# ═══════════════════════════════════════════════════════════════

set -euo pipefail

# ── Colores ───────────────────────────────────────────────────
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m'

ok()   { echo -e "${GREEN}  ✅ $*${NC}"; }
err()  { echo -e "${RED}  ❌ $*${NC}"; }
warn() { echo -e "${YELLOW}  ⚠️  $*${NC}"; }
info() { echo -e "${CYAN}  ℹ️  $*${NC}"; }
step() { echo -e "\n${BOLD}  ── $* ────────────────────────────${NC}"; }

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
cd "$PROJECT_ROOT"

# ── Args ──────────────────────────────────────────────────────
CLUSTER="${1:-${SOLANA_CLUSTER:-devnet}}"
CONFIRM=false
SKIP_TESTS=false
SKIP_BACKUP=false
ERRORS=0

shift || true
while [[ $# -gt 0 ]]; do
    case $1 in
        --confirm)      CONFIRM=true; shift ;;
        --skip-tests)   SKIP_TESTS=true; shift ;;
        --skip-backup)  SKIP_BACKUP=true; shift ;;
        *) err "Argumento desconocido: $1"; exit 1 ;;
    esac
done

# ── Banner ────────────────────────────────────────────────────
echo ""
echo "═══════════════════════════════════════════════════════════"
echo "  🚀 Deploy — Trust Work Escrow"
echo "═══════════════════════════════════════════════════════════"
echo ""
info "Cluster:  ${CLUSTER}"
info "Fecha:    $(date '+%Y-%m-%d %H:%M:%S')"
echo ""

# ── Protección mainnet ────────────────────────────────────────
if [[ "$CLUSTER" == "mainnet-beta" || "$CLUSTER" == "mainnet" ]]; then
    if [ "$CONFIRM" != true ]; then
        echo ""
        err "Deploy a MAINNET requiere --confirm"
        echo ""
        echo "  Uso: ./scripts/deploy.sh mainnet-beta --confirm"
        echo ""
        warn "⚠️  Esto desplegará código en mainnet con dinero REAL."
        warn "⚠️  Asegúrate de haber probado en devnet primero."
        echo ""
        exit 1
    fi
    echo ""
    warn "╔═══════════════════════════════════════════════════╗"
    warn "║         ⚠️  DEPLOY A MAINNET-BETA ⚠️              ║"
    warn "║   Se desplegará con SOL REAL. Esto es definitivo. ║"
    warn "╚═══════════════════════════════════════════════════╝"
    echo ""
    read -p "  ¿Estás seguro? Escribe 'DEPLOY' para confirmar: " user_input
    if [ "$user_input" != "DEPLOY" ]; then
        info "Deploy cancelado."
        exit 0
    fi
fi

# ══════════════════════════════════════════════════════════════
# PRE-FLIGHT CHECKS
# ══════════════════════════════════════════════════════════════

step "1/7 — Verificando herramientas"

for tool in solana anchor cargo rustc node; do
    if command -v "$tool" >/dev/null 2>&1; then
        ok "$tool → $($tool --version 2>&1 | head -1)"
    else
        err "$tool no está instalado"
        ((ERRORS++))
    fi
done

if [ $ERRORS -gt 0 ]; then
    err "Faltan herramientas. Abortando."
    exit 1
fi

# ── Archivos requeridos ───────────────────────────────────────
step "2/7 — Verificando estructura del proyecto"

for file in Anchor.toml Cargo.toml; do
    if [ -f "$file" ]; then
        ok "$file encontrado"
    else
        err "$file no encontrado en $PROJECT_ROOT"
        ((ERRORS++))
    fi
done

if [ $ERRORS -gt 0 ]; then
    err "Faltan archivos del proyecto. ¿Ya ejecutaste 'anchor init'?"
    exit 1
fi

# ── Configurar cluster ────────────────────────────────────────
step "3/7 — Configurando cluster"

solana config set --url "$CLUSTER" >/dev/null 2>&1
CURRENT_URL=$(solana config get | grep "RPC URL" | awk '{print $3}')
ok "RPC URL: $CURRENT_URL"

WALLET=$(solana config get | grep "Keypair Path" | awk '{print $3}')
ok "Wallet: $WALLET"

PUBKEY=$(solana address 2>/dev/null || echo "N/A")
ok "Dirección: $PUBKEY"

# ── Verificar saldo ──────────────────────────────────────────
step "4/7 — Verificando saldo"

BALANCE=$(solana balance --lamports 2>/dev/null | awk '{print $1}' || echo "0")
BALANCE_SOL=$(echo "scale=4; $BALANCE / 1000000000" | bc 2>/dev/null || echo "0")

if [[ "$CLUSTER" == "mainnet-beta" ]]; then
    MIN_BALANCE=5000000000  # 5 SOL para mainnet
    MIN_DISPLAY="5"
else
    MIN_BALANCE=2000000000  # 2 SOL para devnet/testnet
    MIN_DISPLAY="2"
fi

if [ "$BALANCE" -lt "$MIN_BALANCE" ] 2>/dev/null; then
    err "Saldo insuficiente: ${BALANCE_SOL} SOL (mínimo ${MIN_DISPLAY} SOL)"
    if [[ "$CLUSTER" == "devnet" || "$CLUSTER" == "testnet" ]]; then
        info "Puedes obtener SOL gratis: solana airdrop 2"
    fi
    exit 1
else
    ok "Saldo: ${BALANCE_SOL} SOL"
fi

# ── Build ─────────────────────────────────────────────────────
step "5/7 — Compilando programa"

info "Ejecutando 'anchor build'..."
if anchor build 2>&1; then
    ok "Build exitoso"
else
    err "Build falló. Corrige los errores antes de desplegar."
    exit 1
fi

# Verificar que se generó el .so
SO_FILE=$(find target/deploy -name "*.so" 2>/dev/null | head -1)
if [ -n "$SO_FILE" ]; then
    SO_SIZE=$(du -h "$SO_FILE" | awk '{print $1}')
    ok "Programa compilado: $SO_FILE ($SO_SIZE)"
else
    err "No se encontró archivo .so en target/deploy/"
    exit 1
fi

# ── Tests ─────────────────────────────────────────────────────
step "6/7 — Ejecutando tests"

if [ "$SKIP_TESTS" = true ]; then
    warn "Tests omitidos (--skip-tests)"
else
    info "Ejecutando 'anchor test --skip-deploy'..."
    if anchor test --skip-deploy 2>&1; then
        ok "Todos los tests pasaron"
    else
        err "Tests fallaron. No se desplegará hasta que pasen."
        exit 1
    fi
fi

# ── Clippy ────────────────────────────────────────────────────
info "Ejecutando cargo clippy..."
if cargo clippy --workspace -- -D warnings 2>&1; then
    ok "Clippy: sin warnings"
else
    warn "Clippy reportó warnings (no bloqueante)"
fi

# ── Backup antes de deploy ────────────────────────────────────
if [ "$SKIP_BACKUP" != true ]; then
    step "Backup pre-deploy"
    if [ -f "$SCRIPT_DIR/backup-keys.sh" ]; then
        bash "$SCRIPT_DIR/backup-keys.sh" --output "./backups/pre-deploy" || warn "Backup falló (no bloqueante)"
    else
        warn "Script backup-keys.sh no encontrado"
    fi
fi

# ══════════════════════════════════════════════════════════════
# DEPLOY
# ══════════════════════════════════════════════════════════════

step "7/7 — Desplegando a ${CLUSTER}"

DEPLOY_START=$(date +%s)

info "Ejecutando 'anchor deploy --provider.cluster ${CLUSTER}'..."
if anchor deploy --provider.cluster "$CLUSTER" 2>&1; then
    DEPLOY_END=$(date +%s)
    DEPLOY_TIME=$((DEPLOY_END - DEPLOY_START))
    ok "Deploy exitoso en ${DEPLOY_TIME}s"
else
    err "Deploy falló."
    exit 1
fi

# ── Obtener Program ID ───────────────────────────────────────
PROGRAM_ID=$(anchor keys list 2>/dev/null | head -1 | awk '{print $2}' || echo "N/A")

# ── Verificación post-deploy ─────────────────────────────────
echo ""
echo "═══════════════════════════════════════════════════════════"
echo -e "  ${GREEN}✅ DEPLOY COMPLETADO${NC}"
echo "═══════════════════════════════════════════════════════════"
echo ""
info "Cluster:    ${CLUSTER}"
info "Program ID: ${PROGRAM_ID}"
info "Tiempo:     ${DEPLOY_TIME}s"
info "Fecha:      $(date '+%Y-%m-%d %H:%M:%S')"
echo ""

if [ -f "$SCRIPT_DIR/verify-deploy.sh" ]; then
    info "Ejecuta la verificación post-deploy:"
    echo "    ./scripts/verify-deploy.sh $CLUSTER"
fi

echo ""

# ── Log del deploy ────────────────────────────────────────────
mkdir -p "$PROJECT_ROOT/backups/deploy-logs"
LOG_FILE="$PROJECT_ROOT/backups/deploy-logs/deploy-${CLUSTER}-$(date +%Y%m%d_%H%M%S).log"
cat > "$LOG_FILE" <<EOF
Deploy Log — Trust Work Escrow
═══════════════════════════════
Fecha:      $(date '+%Y-%m-%d %H:%M:%S')
Cluster:    ${CLUSTER}
Program ID: ${PROGRAM_ID}
Wallet:     ${PUBKEY}
Saldo:      ${BALANCE_SOL} SOL
Build:      ${SO_FILE} (${SO_SIZE})
Tiempo:     ${DEPLOY_TIME}s
Commit:     $(git rev-parse HEAD 2>/dev/null || echo "N/A")
Branch:     $(git branch --show-current 2>/dev/null || echo "N/A")
EOF

ok "Log guardado: ${LOG_FILE}"
echo ""
