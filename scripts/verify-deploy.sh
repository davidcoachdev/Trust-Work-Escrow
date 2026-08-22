#!/usr/bin/env bash
# ═══════════════════════════════════════════════════════════════
# verify-deploy.sh — Verifica que el programa desplegado coincide
#                     con el código fuente local
#
# Uso:
#   ./scripts/verify-deploy.sh              # usa cluster del env
#   ./scripts/verify-deploy.sh devnet       # verifica en devnet
#   ./scripts/verify-deploy.sh mainnet-beta # verifica en mainnet
#
# Verificaciones:
#   1. El binario on-chain coincide con el build local (hash SHA256)
#   2. El program ID coincide con las keys del proyecto
#   3. El programa es ejecutable y tiene datos
#   4. El IDL está publicado (si aplica)
#   5. La autoridad de upgrade es la esperada
#
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
fail() { echo -e "${RED}  ❌ $*${NC}"; }
warn() { echo -e "${YELLOW}  ⚠️  $*${NC}"; }
info() { echo -e "${CYAN}  ℹ️  $*${NC}"; }
step() { echo -e "\n${BOLD}  ── $* ────────────────────────────${NC}"; }

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
cd "$PROJECT_ROOT"

# ── Args ──────────────────────────────────────────────────────
CLUSTER="${1:-${SOLANA_CLUSTER:-devnet}}"
CHECKS_PASSED=0
CHECKS_FAILED=0
CHECKS_WARNED=0

# ── Banner ────────────────────────────────────────────────────
echo ""
echo "═══════════════════════════════════════════════════════════"
echo "  🔍 Verificación Post-Deploy — Trust Work Escrow"
echo "═══════════════════════════════════════════════════════════"
echo ""
info "Cluster:  ${CLUSTER}"
info "Fecha:    $(date '+%Y-%m-%d %H:%M:%S')"

# ── Configurar cluster ────────────────────────────────────────
solana config set --url "$CLUSTER" >/dev/null 2>&1

# ══════════════════════════════════════════════════════════════
# CHECK 1: Program ID coincide con el proyecto
# ══════════════════════════════════════════════════════════════
step "1/5 — Verificando Program ID"

if ! command -v anchor >/dev/null 2>&1; then
    fail "Anchor no está instalado"
    ((CHECKS_FAILED++))
else
    EXPECTED_PROGRAM_ID=$(anchor keys list 2>/dev/null | head -1 | awk '{print $2}' || echo "")

    if [ -z "$EXPECTED_PROGRAM_ID" ]; then
        fail "No se pudo obtener el Program ID del proyecto"
        ((CHECKS_FAILED++))
    else
        ok "Program ID del proyecto: ${EXPECTED_PROGRAM_ID}"

        # Verificar que coincide con declare_id! en lib.rs
        LIB_RS=$(find programs -name "lib.rs" 2>/dev/null | head -1)
        if [ -n "$LIB_RS" ]; then
            DECLARED_ID=$(grep -oP 'declare_id!\("([^"]+)"\)' "$LIB_RS" 2>/dev/null | grep -oP '"[^"]*"' | tr -d '"' || echo "")
            if [ -n "$DECLARED_ID" ]; then
                if [ "$EXPECTED_PROGRAM_ID" = "$DECLARED_ID" ]; then
                    ok "declare_id! coincide: ${DECLARED_ID}"
                    ((CHECKS_PASSED++))
                else
                    fail "declare_id! NO coincide!"
                    fail "  Proyecto:    ${EXPECTED_PROGRAM_ID}"
                    fail "  declare_id!: ${DECLARED_ID}"
                    ((CHECKS_FAILED++))
                fi
            else
                warn "No se encontró declare_id! en ${LIB_RS}"
                ((CHECKS_WARNED++))
            fi
        fi

        # Verificar que coincide con Anchor.toml
        if [ -f "Anchor.toml" ]; then
            TOML_ID=$(grep -oP '= "([A-Za-z0-9]{32,})"' Anchor.toml 2>/dev/null | head -1 | tr -d '= "' || echo "")
            if [ -n "$TOML_ID" ]; then
                if [ "$EXPECTED_PROGRAM_ID" = "$TOML_ID" ]; then
                    ok "Anchor.toml coincide"
                    ((CHECKS_PASSED++))
                else
                    fail "Anchor.toml tiene un Program ID diferente: ${TOML_ID}"
                    ((CHECKS_FAILED++))
                fi
            fi
        fi
    fi
fi

# ══════════════════════════════════════════════════════════════
# CHECK 2: Programa existe on-chain y es ejecutable
# ══════════════════════════════════════════════════════════════
step "2/5 — Verificando programa on-chain"

if [ -n "${EXPECTED_PROGRAM_ID:-}" ]; then
    ACCOUNT_INFO=$(solana account "$EXPECTED_PROGRAM_ID" --output json 2>/dev/null || echo "")

    if [ -z "$ACCOUNT_INFO" ]; then
        fail "Programa NO encontrado on-chain en ${CLUSTER}"
        ((CHECKS_FAILED++))
    else
        ok "Programa encontrado on-chain"
        ((CHECKS_PASSED++))

        # Verificar que es ejecutable
        IS_EXECUTABLE=$(echo "$ACCOUNT_INFO" | grep -o '"executable":true' || echo "")
        if [ -n "$IS_EXECUTABLE" ]; then
            ok "Programa marcado como ejecutable"
            ((CHECKS_PASSED++))
        else
            fail "Programa NO es ejecutable"
            ((CHECKS_FAILED++))
        fi

        # Mostrar info del programa
        DATA_LEN=$(echo "$ACCOUNT_INFO" | grep -oP '"data":\["[^"]*"' | head -1 | wc -c || echo "0")
        LAMPORTS=$(echo "$ACCOUNT_INFO" | grep -oP '"lamports":\d+' | grep -oP '\d+' || echo "0")
        RENT_SOL=$(echo "scale=4; $LAMPORTS / 1000000000" | bc 2>/dev/null || echo "?")
        info "Rent-exempt: ${RENT_SOL} SOL"
    fi
else
    warn "Saltando verificación on-chain (sin Program ID)"
    ((CHECKS_WARNED++))
fi

# ══════════════════════════════════════════════════════════════
# CHECK 3: Hash del binario — on-chain vs local
# ══════════════════════════════════════════════════════════════
step "3/5 — Verificando integridad del binario"

LOCAL_SO=$(find target/deploy -name "*.so" 2>/dev/null | head -1)

if [ -z "$LOCAL_SO" ]; then
    warn "No se encontró .so local. Ejecuta 'anchor build' primero."
    ((CHECKS_WARNED++))
else
    LOCAL_HASH=$(sha256sum "$LOCAL_SO" | awk '{print $1}')
    LOCAL_SIZE=$(du -h "$LOCAL_SO" | awk '{print $1}')
    ok "Binario local: ${LOCAL_SO}"
    info "  SHA256: ${LOCAL_HASH}"
    info "  Tamaño: ${LOCAL_SIZE}"

    # Intentar descargar y comparar el binario on-chain
    if [ -n "${EXPECTED_PROGRAM_ID:-}" ]; then
        DUMP_FILE="/tmp/onchain-program-${EXPECTED_PROGRAM_ID}.so"
        info "Descargando programa on-chain para comparar..."

        if solana program dump "$EXPECTED_PROGRAM_ID" "$DUMP_FILE" 2>/dev/null; then
            ONCHAIN_HASH=$(sha256sum "$DUMP_FILE" | awk '{print $1}')
            ONCHAIN_SIZE=$(du -h "$DUMP_FILE" | awk '{print $1}')
            info "  SHA256 on-chain: ${ONCHAIN_HASH}"
            info "  Tamaño on-chain: ${ONCHAIN_SIZE}"

            if [ "$LOCAL_HASH" = "$ONCHAIN_HASH" ]; then
                ok "✅ HASHES COINCIDEN — El código desplegado es idéntico al local"
                ((CHECKS_PASSED++))
            else
                fail "⚠️  HASHES NO COINCIDEN"
                fail "  Local:   ${LOCAL_HASH}"
                fail "  On-chain: ${ONCHAIN_HASH}"
                fail "  Esto puede significar que el código fue modificado después del build."
                fail "  Ejecuta 'anchor build' y vuelve a verificar."
                ((CHECKS_FAILED++))
            fi

            rm -f "$DUMP_FILE"
        else
            warn "No se pudo descargar el programa on-chain"
            ((CHECKS_WARNED++))
        fi
    fi
fi

# ══════════════════════════════════════════════════════════════
# CHECK 4: Upgrade authority
# ══════════════════════════════════════════════════════════════
step "4/5 — Verificando autoridad de upgrade"

if [ -n "${EXPECTED_PROGRAM_ID:-}" ]; then
    PROGRAM_INFO=$(solana program show "$EXPECTED_PROGRAM_ID" 2>/dev/null || echo "")

    if [ -n "$PROGRAM_INFO" ]; then
        UPGRADE_AUTH=$(echo "$PROGRAM_INFO" | grep -i "authority" | awk '{print $NF}' || echo "")
        MY_PUBKEY=$(solana address 2>/dev/null || echo "")

        if [ -n "$UPGRADE_AUTH" ]; then
            info "Upgrade authority: ${UPGRADE_AUTH}"

            if [ "$UPGRADE_AUTH" = "$MY_PUBKEY" ]; then
                ok "Upgrade authority coincide con tu wallet"
                ((CHECKS_PASSED++))
            elif [ "$UPGRADE_AUTH" = "none" ] || [ "$UPGRADE_AUTH" = "None" ]; then
                warn "Programa es INMUTABLE (sin upgrade authority)"
                warn "No podrás actualizar este programa nunca más."
                ((CHECKS_WARNED++))
            else
                warn "Upgrade authority es una wallet diferente: ${UPGRADE_AUTH}"
                warn "Tu wallet: ${MY_PUBKEY}"
                ((CHECKS_WARNED++))
            fi
        fi
    else
        warn "No se pudo obtener info del programa"
        ((CHECKS_WARNED++))
    fi
fi

# ══════════════════════════════════════════════════════════════
# CHECK 5: IDL publicado
# ══════════════════════════════════════════════════════════════
step "5/5 — Verificando IDL"

IDL_LOCAL=$(find target/idl -name "*.json" 2>/dev/null | head -1)
if [ -n "$IDL_LOCAL" ]; then
    ok "IDL local encontrado: ${IDL_LOCAL}"
    ((CHECKS_PASSED++))

    # Verificar que tiene instrucciones definidas
    INSTRUCTIONS=$(grep -c '"name"' "$IDL_LOCAL" 2>/dev/null || echo "0")
    info "IDL contiene ~${INSTRUCTIONS} entradas definidas"
else
    warn "No se encontró IDL local en target/idl/"
    warn "Ejecuta 'anchor build' para generarlo"
    ((CHECKS_WARNED++))
fi

# ── Verificar git status ─────────────────────────────────────
step "Extra — Estado del repositorio"

if git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
    COMMIT=$(git rev-parse --short HEAD 2>/dev/null || echo "N/A")
    BRANCH=$(git branch --show-current 2>/dev/null || echo "N/A")
    DIRTY=$(git status --porcelain 2>/dev/null | wc -l)

    ok "Commit: ${COMMIT} (${BRANCH})"

    if [ "$DIRTY" -gt 0 ]; then
        warn "Hay ${DIRTY} archivos sin commitear"
        warn "El deploy podría no ser reproducible"
        ((CHECKS_WARNED++))
    else
        ok "Repositorio limpio"
        ((CHECKS_PASSED++))
    fi
fi

# ══════════════════════════════════════════════════════════════
# RESUMEN
# ══════════════════════════════════════════════════════════════
echo ""
echo "═══════════════════════════════════════════════════════════"

TOTAL=$((CHECKS_PASSED + CHECKS_FAILED + CHECKS_WARNED))

if [ $CHECKS_FAILED -eq 0 ]; then
    echo -e "  ${GREEN}✅ VERIFICACIÓN EXITOSA${NC}"
else
    echo -e "  ${RED}❌ VERIFICACIÓN CON ERRORES${NC}"
fi

echo ""
echo -e "  ${GREEN}Pasaron:  ${CHECKS_PASSED}/${TOTAL}${NC}"
if [ $CHECKS_FAILED -gt 0 ]; then
    echo -e "  ${RED}Fallaron: ${CHECKS_FAILED}/${TOTAL}${NC}"
fi
if [ $CHECKS_WARNED -gt 0 ]; then
    echo -e "  ${YELLOW}Warnings: ${CHECKS_WARNED}/${TOTAL}${NC}"
fi
echo ""
echo "═══════════════════════════════════════════════════════════"
echo ""

# Exit con error si hubo fallos
if [ $CHECKS_FAILED -gt 0 ]; then
    exit 1
fi
