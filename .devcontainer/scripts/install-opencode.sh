#!/bin/bash
set -euo pipefail

# ═══════════════════════════════════════════════════════════════════
# Instala OpenCode CLI
# ═══════════════════════════════════════════════════════════════════

echo "📦 Installing OpenCode CLI..."

# Ensure ~/.local/bin is in PATH
export PATH="$HOME/.local/bin:$PATH"
mkdir -p "$HOME/.local/bin"

# Clean up any existing opencode installation
rm -f "$HOME/.local/bin/opencode"

curl -fsSL https://opencode.ai/install | bash

echo "✅ OpenCode CLI installed"
