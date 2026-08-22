#!/bin/bash
set -euo pipefail

# ═══════════════════════════════════════════════════════════════════
# Instala GitHub CLI (descarga directa)
# ═══════════════════════════════════════════════════════════════════

GH_VERSION="${1:-2.67.0}"

echo "📦 Instalando GitHub CLI v${GH_VERSION}..."

curl -fsSL "https://github.com/cli/cli/releases/download/v${GH_VERSION}/gh_${GH_VERSION}_linux_amd64.tar.gz" \
    | tar -xz -C /tmp

mv "/tmp/gh_${GH_VERSION}_linux_amd64/bin/gh" /usr/local/bin/gh
rm -rf "/tmp/gh_${GH_VERSION}_linux_amd64"

echo "✅ GitHub CLI $(gh --version | head -1) instalado"
