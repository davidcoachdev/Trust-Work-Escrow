#!/usr/bin/env bash
# ═══════════════════════════════════════════════════════════════
# backup-keys.sh — Cifra y respalda keypairs críticos del programa
#
# Uso:
#   ./scripts/backup-keys.sh                    # backup interactivo
#   ./scripts/backup-keys.sh --output ./safe/   # directorio destino
#
# Genera un archivo .tar.gz.gpg cifrado con passphrase que contiene:
#   - Program keypair (define el Program ID)
#   - Deploy authority keypair
#   - IDL del programa
#
# ⚠️  NUNCA commitees los keypairs sin cifrar. Este script los cifra
#     con GPG simétrico antes de guardarlos.
# ═══════════════════════════════════════════════════════════════

set -euo pipefail

# ── Colores ───────────────────────────────────────────────────
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

ok()   { echo -e "${GREEN}  ✅ $*${NC}"; }
err()  { echo -e "${RED}  ❌ $*${NC}"; exit 1; }
warn() { echo -e "${YELLOW}  ⚠️  $*${NC}"; }
info() { echo -e "  ℹ️  $*"; }

# ── Args ──────────────────────────────────────────────────────
OUTPUT_DIR="./backups/keys"
while [[ $# -gt 0 ]]; do
    case $1 in
        --output|-o) OUTPUT_DIR="$2"; shift 2 ;;
        *) err "Argumento desconocido: $1" ;;
    esac
done

# ── Verificaciones ────────────────────────────────────────────
command -v gpg >/dev/null 2>&1 || err "gpg no está instalado. Instala con: sudo apt install gnupg"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

echo ""
echo "═══════════════════════════════════════════════════"
echo "  🔐 Backup de keypairs — Trust Work Escrow"
echo "═══════════════════════════════════════════════════"
echo ""

# ── Buscar archivos críticos ──────────────────────────────────
BACKUP_FILES=()

# Program keypair
PROGRAM_KEYPAIR=$(find "$PROJECT_ROOT" -name "*-keypair.json" -path "*/target/*" 2>/dev/null | head -1)
if [ -n "$PROGRAM_KEYPAIR" ]; then
    BACKUP_FILES+=("$PROGRAM_KEYPAIR")
    ok "Program keypair: $PROGRAM_KEYPAIR"
else
    warn "No se encontró program keypair en target/"
fi

# Deploy authority (default Solana keypair)
DEPLOY_KEYPAIR="$HOME/.config/solana/id.json"
if [ -f "$DEPLOY_KEYPAIR" ]; then
    BACKUP_FILES+=("$DEPLOY_KEYPAIR")
    ok "Deploy authority: $DEPLOY_KEYPAIR"
else
    warn "No se encontró deploy authority keypair"
fi

# IDL files
while IFS= read -r idl_file; do
    BACKUP_FILES+=("$idl_file")
    ok "IDL: $idl_file"
done < <(find "$PROJECT_ROOT" -name "*.json" -path "*/idl/*" 2>/dev/null)

# Anchor.toml (contiene Program IDs)
if [ -f "$PROJECT_ROOT/Anchor.toml" ]; then
    BACKUP_FILES+=("$PROJECT_ROOT/Anchor.toml")
    ok "Anchor.toml (contiene Program IDs)"
fi

if [ ${#BACKUP_FILES[@]} -eq 0 ]; then
    err "No se encontraron archivos para respaldar. ¿Ya compilaste el programa?"
fi

# ── Crear backup cifrado ─────────────────────────────────────
mkdir -p "$OUTPUT_DIR"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
ARCHIVE_NAME="trustwork-keys-${TIMESTAMP}.tar.gz"
ENCRYPTED_NAME="${ARCHIVE_NAME}.gpg"

echo ""
info "Creando archivo cifrado..."
info "Se te pedirá una passphrase. NO LA PIERDAS."
echo ""

# Crear tar con rutas relativas
tar -czf "/tmp/${ARCHIVE_NAME}" -C / "${BACKUP_FILES[@]/#\//}"

# Cifrar con GPG simétrico
gpg --symmetric --cipher-algo AES256 --batch --yes \
    --output "${OUTPUT_DIR}/${ENCRYPTED_NAME}" \
    "/tmp/${ARCHIVE_NAME}"

# Limpiar temporal
rm -f "/tmp/${ARCHIVE_NAME}"

echo ""
echo "═══════════════════════════════════════════════════"
ok "Backup cifrado creado: ${OUTPUT_DIR}/${ENCRYPTED_NAME}"
echo ""
info "Para restaurar:"
echo "    gpg --decrypt ${OUTPUT_DIR}/${ENCRYPTED_NAME} | tar -xzf - -C /"
echo ""
warn "Guarda la passphrase en un lugar seguro (password manager)"
warn "NUNCA commitees este archivo al repositorio"
echo "═══════════════════════════════════════════════════"
echo ""
