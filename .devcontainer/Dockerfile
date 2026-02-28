# syntax=docker/dockerfile:1.4
# ═══════════════════════════════════════════════════════════════════
#  Solana Dev Container — Multi-stage optimized build
#
#  Stage 1 (avm-builder): Compila AVM desde fuente (cached)
#  Stage 2 (final):       Dev container completo, copia binarios
#
#  Optimizaciones:
#  - BuildKit cache mounts: cargo registry NO se re-descarga
#  - Multi-stage: AVM solo se recompila si cambia ANCHOR_VERSION
#  - COPY --from: binarios AVM+Anchor se copian instantáneamente
#  - Prebuild CI: la imagen se cachea en ghcr.io para Codespaces
#
#  Ubuntu 24.04: incluye GLIBC_2.39, requerida por Anchor ≥ 0.32
# ═══════════════════════════════════════════════════════════════════

# ─────────────────────────────────────────────────────────────────
# Versiones centralizadas (cámbialas aquí)
# ─────────────────────────────────────────────────────────────────
ARG SOLANA_CLI_VERSION="stable"
ARG ANCHOR_VERSION="0.32.1"
ARG NODE_VERSION="20"


# ═════════════════════════════════════════════════════════════════
# STAGE 1: AVM Builder
# Compila AVM + descarga Anchor precompilado.
# Esta stage se cachea por Docker. Solo se re-ejecuta si
# ANCHOR_VERSION cambia o se hace --no-cache.
# ═════════════════════════════════════════════════════════════════
FROM ubuntu:24.04 AS avm-builder

ARG ANCHOR_VERSION

ENV DEBIAN_FRONTEND=noninteractive

# Dependencias mínimas para compilar AVM
RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential pkg-config libssl-dev curl ca-certificates git \
    && rm -rf /var/lib/apt/lists/*

# Rust minimal (solo para compilar AVM, no va al contenedor final)
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
    | sh -s -- -y --default-toolchain stable --profile minimal
ENV PATH="/root/.cargo/bin:${PATH}"

# Compilar AVM con BuildKit cache mounts
# → cargo registry y git index se persisten entre builds
# → rebuild parcial: solo recompila si hay cambios en el repo
RUN --mount=type=cache,target=/root/.cargo/registry,id=cargo-reg \
    --mount=type=cache,target=/root/.cargo/git,id=cargo-git \
    --mount=type=cache,target=/tmp/avm-target,id=avm-target \
    CARGO_TARGET_DIR=/tmp/avm-target \
    cargo install --git https://github.com/solana-foundation/anchor avm --force

# AVM descarga Anchor precompilado (~30s, no compila nada)
RUN avm install ${ANCHOR_VERSION} \
    && avm use ${ANCHOR_VERSION}


# ═════════════════════════════════════════════════════════════════
# STAGE 2: Dev Container final
# Contiene todo lo necesario para desarrollar.
# AVM + Anchor se copian del builder (instantáneo).
# ═════════════════════════════════════════════════════════════════
FROM ubuntu:24.04 AS devcontainer

ARG SOLANA_CLI_VERSION
ARG ANCHOR_VERSION
ARG NODE_VERSION

ENV DEBIAN_FRONTEND=noninteractive \
    TZ=UTC \
    LANG=C.UTF-8

# ─────────────────────────────────────────────────────────────────
# 1. Dependencias del sistema
# ─────────────────────────────────────────────────────────────────
# GitHub CLI repo
RUN curl -fsSL https://cli.github.com/packages/githubcli-archive-keyring.gpg \
    | dd of=/usr/share/keyrings/githubcli-archive-keyring.gpg 2>/dev/null \
    && echo "deb [arch=$(dpkg --print-architecture) signed-by=/usr/share/keyrings/githubcli-archive-keyring.gpg] https://cli.github.com/packages stable main" \
    | tee /etc/apt/sources.list.d/github-cli.list > /dev/null

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
    gh \
    jq \
    gnupg \
    sudo \
    ca-certificates \
    unzip \
    tini \
    && rm -rf /var/lib/apt/lists/*

# ─────────────────────────────────────────────────────────────────
# 2. Node.js LTS + utilidades JS globales
# ─────────────────────────────────────────────────────────────────
RUN curl -fsSL https://deb.nodesource.com/gpgkey/nodesource-repo.gpg.key \
    | gpg --dearmor -o /usr/share/keyrings/nodesource.gpg \
    && echo "deb [signed-by=/usr/share/keyrings/nodesource.gpg] https://deb.nodesource.com/node_${NODE_VERSION}.x nodistro main" \
    | tee /etc/apt/sources.list.d/nodesource.list > /dev/null \
    && apt-get update \
    && apt-get install -y nodejs \
    && npm install -g yarn typescript ts-node \
    && rm -rf /var/lib/apt/lists/*

# ─────────────────────────────────────────────────────────────────
# 3. Usuario no-root "vscode"
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
# 5. Solana CLI (descarga binarios precompilados — rápido)
# ─────────────────────────────────────────────────────────────────
RUN sh -c "$(curl -sSfL https://release.anza.xyz/${SOLANA_CLI_VERSION}/install)"

ENV PATH="${HOME}/.local/share/solana/install/active_release/bin:${PATH}"

RUN mkdir -p ${HOME}/.config/solana \
    && solana-keygen new --no-bip39-passphrase --silent \
    --outfile ${HOME}/.config/solana/id.json \
    && solana config set --url localhost

# ─────────────────────────────────────────────────────────────────
# 6. AVM + Anchor — COPIADOS del builder (instantáneo, 0 compilación)
# ─────────────────────────────────────────────────────────────────
COPY --from=avm-builder --chown=vscode:vscode /root/.cargo/bin/avm ${HOME}/.cargo/bin/avm
COPY --from=avm-builder --chown=vscode:vscode /root/.avm ${HOME}/.avm

ENV PATH="${HOME}/.avm/bin:${PATH}"

# ─────────────────────────────────────────────────────────────────
# 7. Metadata
# ─────────────────────────────────────────────────────────────────
LABEL maintainer="TrustWork" \
    description="Solana Dev Container (multi-stage)" \
    solana.version="${SOLANA_CLI_VERSION}" \
    anchor.version="${ANCHOR_VERSION}" \
    node.version="${NODE_VERSION}"

# ─────────────────────────────────────────────────────────────────
# 8. tini como init — previene procesos zombie
# ─────────────────────────────────────────────────────────────────
ENTRYPOINT ["/usr/bin/tini", "--"]

WORKDIR /workspace