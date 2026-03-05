#!/bin/bash
set -euo pipefail

# ═══════════════════════════════════════════════════════════════════
# Instala Surfpool CLI
# ═══════════════════════════════════════════════════════════════════

echo "📦 Installing Surfpool CLI..."

# Ensure ~/.local/bin is in PATH
export PATH="$HOME/.local/bin:$PATH"
mkdir -p "$HOME/.local/bin"

# Clean up any existing surfpool installation to avoid symlink errors
rm -f "$HOME/.local/bin/surfpool"
rm -rf "$HOME/.local/share/surfpool"

# Download surfpool installer directly
INSTALLER_URL="https://run.surfpool.run/"
TAR_URL="http://txtx-public.s3.amazonaws.com/releases/surfpool-linux-x64.tar.gz"

echo "Downloading surfpool from ${TAR_URL}..."
curl -sSL -o /tmp/surfpool.tar.gz "$TAR_URL"

echo "Extracting surfpool..."
cd /tmp
gzip -d surfpool.tar.gz
tar -xf surfpool.tar

echo "Installing surfpool to ${HOME}/.local/bin..."
install -m 0755 surfpool "$HOME/.local/bin/surfpool"

# Verify installation
if command -v surfpool &> /dev/null; then
    echo "✅ Surfpool CLI installed: $(surfpool --version)"
else
    echo "❌ Surfpool CLI installation failed"
    exit 1
fi
