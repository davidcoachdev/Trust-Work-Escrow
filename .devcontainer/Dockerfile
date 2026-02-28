# ═══════════════════════════════════════════════════════════════════
#  Solana Dev Container — Rust + Solana CLI + Anchor + Node.js
#  Todo se instala en BUILD TIME → el contenedor arranca listo.
#  Ubuntu 24.04: incluye GLIBC_2.39, requerida por Anchor ≥ 0.32
# ═══════════════════════════════════════════════════════════════════
FROM ubuntu:24.04

# ─────────────────────────────────────────────────────────────────
# Versiones (cámbialas aquí si necesitas actualizar)
# ─────────────────────────────────────────────────────────────────
ARG SOLANA_CLI_VERSION="stable"
ARG NODE_VERSION="20"

ENV DEBIAN_FRONTEND=noninteractive \
    TZ=UTC \
    LANG=C.UTF-8

# ─────────────────────────────────────────────────────────────────
# 1. Dependencias del sistema
# ─────────────────────────────────────────────────────────────────
RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    pkg-config \
    libudev-dev \
    libssl-dev \
    clang \
    cmake \
    libclang-dev \
    python3 \
    python3-pip \
    curl \
    wget \
    git \
    jq \
    sudo \
    ca-certificates \
    unzip \
    && rm -rf /var/lib/apt/lists/*

# ─────────────────────────────────────────────────────────────────
# 2. Node.js LTS (via NodeSource) + utilidades JS globales
# ─────────────────────────────────────────────────────────────────
RUN curl -fsSL https://deb.nodesource.com/setup_${NODE_VERSION}.x | bash - \
    && apt-get install -y nodejs \
    && npm install -g yarn typescript ts-node \
    && rm -rf /var/lib/apt/lists/*

# ─────────────────────────────────────────────────────────────────
# 3. Usuario no-root "vscode"
#    Ubuntu 24.04 ya tiene un usuario con UID 1000 (ubuntu), así que
#    lo renombramos o creamos vscode si no existe.
# ─────────────────────────────────────────────────────────────────
RUN if id -u 1000 > /dev/null 2>&1; then \
    existing_user=$(getent passwd 1000 | cut -d: -f1); \
    if [ "$existing_user" != "vscode" ]; then \
    usermod -l vscode -d /home/vscode -m "$existing_user" 2>/dev/null || true; \
    groupmod -n vscode "$existing_user" 2>/dev/null || true; \
    fi; \
    else \
    useradd -m -u 1000 -s /bin/bash vscode; \
    fi \
    && echo "vscode ALL=(ALL) NOPASSWD:ALL" > /etc/sudoers.d/vscode \
    && chmod 0440 /etc/sudoers.d/vscode \
    && mkdir -p /home/vscode \
    && chown -R 1000:1000 /home/vscode

USER vscode
ENV HOME=/home/vscode

# ─────────────────────────────────────────────────────────────────
# 4. Rust (stable) + componentes esenciales
# ─────────────────────────────────────────────────────────────────
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
    | sh -s -- -y --default-toolchain stable --profile default \
    && ~/.cargo/bin/rustup component add rustfmt clippy

ENV PATH="${HOME}/.cargo/bin:${PATH}"

# ─────────────────────────────────────────────────────────────────
# 5. Solana CLI (instalación oficial vía script de Anza)
# ─────────────────────────────────────────────────────────────────
RUN sh -c "$(curl -sSfL https://release.anza.xyz/${SOLANA_CLI_VERSION}/install)"

ENV PATH="/home/vscode/.local/share/solana/install/active_release/bin:${PATH}"

# Keygen + config para desarrollo local
RUN mkdir -p /home/vscode/.config/solana \
    && solana-keygen new --no-bip39-passphrase --silent \
    --outfile /home/vscode/.config/solana/id.json \
    && solana config set --url localhost

# ─────────────────────────────────────────────────────────────────
# 6. Anchor CLI (via AVM – compila desde fuente, ~10 min build)
#    Se hace aquí para que quede en la imagen y no en postCreate.
# ─────────────────────────────────────────────────────────────────
RUN cargo install --git https://github.com/solana-foundation/anchor avm --force \
    && avm install 0.32.1 \
    && avm use 0.32.1

ENV PATH="${HOME}/.avm/bin:${PATH}"

# ─────────────────────────────────────────────────────────────────
# 7. Workspace
# ─────────────────────────────────────────────────────────────────
WORKDIR /workspace