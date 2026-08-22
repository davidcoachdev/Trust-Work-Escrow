#!/usr/bin/env bash
# ═══════════════════════════════════════════════════════════════
# check-permissions.sh — Audita permisos 0600 en archivos sensibles (T19)
# Uso: ./scripts/check-permissions.sh [--ci] [--fix]
# ═══════════════════════════════════════════════════════════════
set -euo pipefail

GREEN='\033[0;32m'; RED='\033[0;31m'; YELLOW='\033[1;33m'; NC='\033[0m'
ok()   { echo -e "${GREEN}  ✅ $*${NC}"; }
err()  { echo -e "${RED}  ❌ $*${NC}"; }
warn() { echo -e "${YELLOW}  ⚠️  $*${NC}"; }
info() { echo -e "  ℹ️  $*"; }

CI_MODE=false
FIX_MODE=false
for arg in "$@"; do
  case $arg in
    --ci) CI_MODE=true ;;
    --fix) FIX_MODE=true ;;
  esac
done

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
cd "$PROJECT_ROOT"

echo ""
echo "═══════════════════════════════════════════════════"
echo "  🔒 Permission audit 0600 — Trust Work Escrow (T19)"
echo "═══════════════════════════════════════════════════"
echo ""

FAILED=0

is_0600() {
  local file="$1"
  local mode
  mode=$(stat -c "%a" "$file" 2>/dev/null || stat -f "%OLp" "$file" 2>/dev/null || echo "unknown")
  # Debe ser 600 o 400 (sin bits grupo/otros)
  if [[ "$mode" == "600" || "$mode" == "400" ]]; then
    return 0
  fi
  # Check via octal bits: 077 debe ser 0
  local oct
  oct=$(stat -c "%a" "$file" 2>/dev/null || echo "")
  if [[ -n "$oct" ]]; then
    local last_two=${oct: -2}
    # No robust: mejor check permisso raw
    local raw
    raw=$(stat -c "%a" "$file" 2>/dev/null)
    # Si contiene grupo/otros no-0 => fail
    # Simplificado: solo acepta 600/400
    return 1
  fi
  return 1
}

check_file() {
  local file="$1"
  if [ ! -f "$file" ]; then
    return 0
  fi
  if [ ! -r "$file" ]; then
    return 0
  fi
  local mode
  mode=$(stat -c "%a" "$file" 2>/dev/null || stat -f "%OLp" "$file" 2>/dev/null || echo "unknown")
  if [[ "$mode" == "600" || "$mode" == "400" ]]; then
    ok "$file → $mode"
  else
    if [ "$FIX_MODE" = true ]; then
      chmod 600 "$file"
      ok "Fix: $file → 600 (era $mode)"
    else
      err "$file → $mode (esperado 600)"
      info "  Corrige con: chmod 600 $file  o  ./scripts/check-permissions.sh --fix"
      FAILED=1
    fi
  fi
}

# ── Archivos sensibles conocidos ─────────────────────────────
info "Revisando archivos sensibles existentes en disco..."

# .env files
for f in .env .env.local backend/.env trust-escrow/.env; do
  check_file "$PROJECT_ROOT/$f"
done

# Solana keypairs
for f in $(find "$PROJECT_ROOT" -maxdepth 4 -name "*-keypair.json" -o -name "id.json" -o -name "deploy-keypair.json" 2>/dev/null | head -n 20); do
  # Excluir target/deploy keypairs generados (se auditan aparte)
  if [[ "$f" == *"/target/"* ]]; then
    continue
  fi
  check_file "$f"
done

# PEM / key files
for f in $(find "$PROJECT_ROOT" -maxdepth 3 -name "*.pem" -o -name "*.key" 2>/dev/null | head -n 20); do
  check_file "$f"
done

# Backend config que pueda contener secrets
for f in backend/api/.env backend/.env; do
  check_file "$PROJECT_ROOT/$f"
done

echo ""

# ── Test de helper Rust (write_secure_file) ─────────────────
info "Verificando helper Rust write_secure_file (cargo test)..."
if [ "$CI_MODE" = true ]; then
  # En CI ya se ejecuta cargo test; aquí solo informativo
  info "CI mode: validación de permisos delegada a cargo test + clippy"
else
  # Local: probar rápidamente si backend compila
  if command -v cargo >/dev/null 2>&1; then
    if (cd "$PROJECT_ROOT/backend" && cargo test -p trust-escrow-api logging -- --nocapture 2>&1 | tail -n 5); then
      ok "logging tests (permisos 0600) pasan"
    else
      warn "logging tests no pasaron (ejecuta: cargo test -p trust-escrow-api logging)"
    fi
  fi
fi

echo ""
if [ $FAILED -ne 0 ]; then
  err "Permission audit: FAILED — hay archivos con permisos inseguros"
  echo ""
  info "Documentación: backend/api/src/logging.rs :: set_secure_permissions / write_secure_file"
  if [ "$CI_MODE" = true ]; then
    exit 1
  else
    warn "Modo local: no bloqueante, pero corrige antes de commit (--fix)"
    exit 1
  fi
else
  ok "Permission audit: PASSED"
  echo ""
fi
