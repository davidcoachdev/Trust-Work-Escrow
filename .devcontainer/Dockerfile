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
# Scripts modulares en .devcontainer/scripts/
# ═════════════════════════════════════════════════════════════════
FROM ubuntu:24.04 AS devcontainer

ARG SOLANA_CLI_VERSION
ARG ANCHOR_VERSION
ARG NODE_VERSION
ARG GH_CLI_VERSION="2.67.0"

ENV DEBIAN_FRONTEND=noninteractive \
    TZ=UTC \
    LANG=C.UTF-8

# ─────────────────────────────────────────────────────────────────
# Copiar scripts de instalación y dar permisos
# ─────────────────────────────────────────────────────────────────
COPY scripts/*.sh /tmp/scripts/
RUN chmod +x /tmp/scripts/*.sh

# ─────────────────────────────────────────────────────────────────
# 1. Dependencias base del sistema
# ─────────────────────────────────────────────────────────────────
RUN /tmp/scripts/install-base-deps.sh

# ─────────────────────────────────────────────────────────────────
# 2. Node.js LTS + utilidades globales
# ─────────────────────────────────────────────────────────────────
RUN /tmp/scripts/install-node.sh ${NODE_VERSION}

# ─────────────────────────────────────────────────────────────────
# 3. GitHub CLI
# ─────────────────────────────────────────────────────────────────
RUN /tmp/scripts/install-gh-cli.sh ${GH_CLI_VERSION}

# ─────────────────────────────────────────────────────────────────
# 4. Usuario no-root "vscode"
# ─────────────────────────────────────────────────────────────────
RUN /tmp/scripts/setup-user.sh

# ─────────────────────────────────────────────────────────────────
# Cambiar a usuario vscode para las siguientes instalaciones
# ─────────────────────────────────────────────────────────────────
USER vscode
ENV HOME=/home/vscode

# ─────────────────────────────────────────────────────────────────
# 5. Rust (stable) + componentes esenciales
# ─────────────────────────────────────────────────────────────────
RUN /tmp/scripts/install-rust.sh
ENV PATH="${HOME}/.cargo/bin:${PATH}"

# ─────────────────────────────────────────────────────────────────
# 6. Solana CLI
# ─────────────────────────────────────────────────────────────────
RUN /tmp/scripts/install-solana.sh ${SOLANA_CLI_VERSION}
ENV PATH="${HOME}/.local/share/solana/install/active_release/bin:${PATH}"

# ─────────────────────────────────────────────────────────────────
# 7. AVM + Anchor — COPIADOS del builder
#    Nota: avm use recrea el symlink en la ruta correcta
# ─────────────────────────────────────────────────────────────────
COPY --from=avm-builder --chown=vscode:vscode /root/.cargo/bin/avm ${HOME}/.cargo/bin/avm
COPY --from=avm-builder --chown=vscode:vscode /root/.avm ${HOME}/.avm
ENV PATH="${HOME}/.avm/bin:${PATH}"

# Re-activar Anchor para crear symlink correcto (los del builder apuntan a /root)
RUN avm use ${ANCHOR_VERSION}

# ═══════════════════════════════════════════════════════════════════
# 8. OpenCode CLI
# ═══════════════════════════════════════════════════════════════════
RUN /tmp/scripts/install-opencode.sh

# ═══════════════════════════════════════════════════════════════════
# 9. Surfpool CLI
# ═══════════════════════════════════════════════════════════════════
RUN /tmp/scripts/install-surfpool.sh
ENV PATH="${HOME}/.local/bin:${PATH}"

# ═══════════════════════════════════════════════════════════════════
# 10. Metadata
# ─────────────────────────────────────────────────────────────────
LABEL maintainer="TrustWork" \
    description="Solana Dev Container (modular)" \
    solana.version="${SOLANA_CLI_VERSION}" \
    anchor.version="${ANCHOR_VERSION}" \
    node.version="${NODE_VERSION}"

# ─────────────────────────────────────────────────────────────────
# 9. tini como init — previene procesos zombie
# ─────────────────────────────────────────────────────────────────
ENTRYPOINT ["/usr/bin/tini", "--"]

# CMD: mantiene el contenedor vivo (requerido cuando overrideCommand: false)
CMD ["sleep", "infinity"]

WORKDIR /workspace