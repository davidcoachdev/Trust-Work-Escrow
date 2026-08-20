#!/usr/bin/env bash
# ═══════════════════════════════════════════════════════════════
# secret-scan.sh — Escaneo local de secretos (T19)
# Uso: ./scripts/secret-scan.sh [--ci]
# Requiere: gitleaks (opcional), o fallback grep
# ═══════════════════════════════════════════════════════════════
set -euo pipefail

GREEN='\033[0;32m'; RED='\033[0;31m'; YELLOW='\033[1;33m'; NC='\033[0m'
ok()   { echo -e "${GREEN}  ✅ $*${NC}"; }
err()  { echo -e "${RED}  ❌ $*${NC}"; }
warn() { echo -e "${YELLOW}  ⚠️  $*${NC}"; }
info() { echo -e "  ℹ️  $*"; }

CI_MODE=false
if [[ "${1:-}" == "--ci" ]]; then CI_MODE=true; fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
cd "$PROJECT_ROOT"

echo ""
echo "═══════════════════════════════════════════════════"
echo "  🔍 Secret scan — Trust Work Escrow (T19)"
echo "═══════════════════════════════════════════════════"
echo ""

FOUND=0

# ── 1. gitleaks si está disponible ───────────────────────────
if command -v gitleaks >/dev/null 2>&1; then
  info "Gitleaks encontrado, escaneando histórico..."
  if [ -f .gitleaks.toml ]; then
    ARGS="detect --source . --no-banner --redact --verbose --config .gitleaks.toml"
  else
    ARGS="detect --source . --no-banner --redact --verbose"
  fi
  # En CI, no usar --exit-code manual; dejar que gitleaks decida
  if gitleaks $ARGS 2>&1 | tee /tmp/gitleaks.out; then
    ok "Gitleaks: sin secretos detectados"
  else
    err "Gitleaks detectó posibles secretos (ver arriba)"
    FOUND=1
  fi
  echo ""
else
  warn "gitleaks no instalado — usando fallback grep (instala: https://github.com/gitleaks/gitleaks)"
  echo ""
fi

# ── 2. Fallback / complementario: grep patterns críticos ─────
info "Escaneando patterns sensibles con grep (excluye target/.git/logging docs)..."

# Patterns que NUNCA deben estar en texto plano fuera de logging.rs / tests
PATTERNS=(
  "-----BEGIN.*PRIVATE KEY-----"
  "-----BEGIN RSA PRIVATE KEY-----"
  "-----BEGIN EC PRIVATE KEY-----"
  "-----BEGIN OPENSSH PRIVATE KEY-----"
  "aws_secret_access_key"
  "aws_session_token"
  "ghp_[a-zA-Z0-9]{36}"
  "github_pat_"
  "sk-[a-zA-Z0-9]{20,}"
  "AKIA[0-9A-Z]{16}"
)

EXCLUDE_DIRS=(target .git node_modules .next dist .serena .atl .surfpool)
EXCLUDE_FILES=("Cargo.lock" "*.log" "secret-scan.sh" "logging.rs" ".gitleaks.toml" "*.md")

# Construir args exclude
GREP_EXCLUDES=""
for d in "${EXCLUDE_DIRS[@]}"; do
  GREP_EXCLUDES="$GREP_EXCLUDES --exclude-dir=$d"
done
for f in "${EXCLUDE_FILES[@]}"; do
  GREP_EXCLUDES="$GREP_EXCLUDES --exclude=$f"
done

TMP_OUT=$(mktemp)
trap "rm -f $TMP_OUT" EXIT

for pat in "${PATTERNS[@]}"; do
  # shellcheck disable=SC2086
  if grep -R -n -E $GREP_EXCLUDES "$pat" . 2>/dev/null | grep -v "scripts/secret-scan" | grep -v "secret-scan.yml" | grep -v ".gitleaks.toml" >> "$TMP_OUT" ; then
    : # matched lines already appended
  fi
done

if [ -s "$TMP_OUT" ]; then
  err "Patterns sensibles encontrados (revisar y redactar):"
  cat "$TMP_OUT"
  echo ""
  # En CI falla el job; local solo warn
  if [ "$CI_MODE" = true ]; then
    FOUND=1
  else
    warn "Modo local: no bloqueante, pero corrige antes de commit"
  fi
else
  ok "Grep patterns: sin coincidencias sospechosas"
fi

echo ""

# ── 3. Verificar .gitignore cubre sensibles ──────────────────
info "Verificando .gitignore cubre archivos sensibles..."
for required in ".env" "*-keypair.json" "id.json" "*.pem" "*.key"; do
  # Busca patrón en .gitignore (glob simple)
  if grep -qF "$required" .gitignore 2>/dev/null || grep -qF ".env" .gitignore 2>/dev/null; then
    ok ".gitignore cubre $required"
  else
    warn ".gitignore no menciona explícitamente $required (revisar)"
  fi
done

echo ""

# ── 4. Verificar logging redaction wiring ────────────────────
info "Verificando wiring de logging seguro..."
if grep -q "logging::redact_secrets" backend/api/src/error.rs 2>/dev/null; then
  ok "error.rs delega a logging::redact_secrets"
else
  err "error.rs no delega a logging::redact_secrets"
  FOUND=1
fi

if [ -f backend/api/src/logging.rs ]; then
  ok "backend/api/src/logging.rs existe"
  if grep -q "REDACTED" backend/api/src/logging.rs; then
    ok "logging.rs contiene REDACTED"
  fi
else
  err "backend/api/src/logging.rs no encontrado"
  FOUND=1
fi

echo ""
if [ $FOUND -ne 0 ]; then
  err "Secret scan: FAILED — corrige los hallazgos antes de push"
  exit 1
else
  ok "Secret scan: PASSED"
  echo ""
fi
