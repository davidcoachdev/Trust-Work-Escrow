#!/bin/bash
set -euo pipefail

# ═══════════════════════════════════════════════════════════════════
# Configura usuario no-root "vscode" con UID 1000
# ═══════════════════════════════════════════════════════════════════

echo "👤 Configurando usuario vscode..."

if id -u 1000 > /dev/null 2>&1; then
    existing_user=$(getent passwd 1000 | cut -d: -f1)
    if [ "$existing_user" != "vscode" ]; then
        usermod -l vscode -d /home/vscode -m "$existing_user" 2>/dev/null || true
        groupmod -n vscode "$existing_user" 2>/dev/null || true
    fi
else
    useradd -m -u 1000 -s /bin/bash vscode
fi

echo "vscode ALL=(ALL) NOPASSWD:ALL" > /etc/sudoers.d/vscode
chmod 0440 /etc/sudoers.d/vscode

mkdir -p /home/vscode
chown -R 1000:1000 /home/vscode

echo "✅ Usuario vscode configurado"
