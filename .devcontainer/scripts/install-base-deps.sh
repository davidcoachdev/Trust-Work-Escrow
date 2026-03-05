#!/bin/bash
set -euo pipefail

# ═══════════════════════════════════════════════════════════════════
# Instala dependencias base del sistema
# ═══════════════════════════════════════════════════════════════════

export DEBIAN_FRONTEND=noninteractive

apt-get update && apt-get install -y --no-install-recommends \
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
    gnupg \
    sudo \
    ca-certificates \
    unzip \
    xz-utils \
    tini \
    protobuf-compiler \
    gzip

rm -rf /var/lib/apt/lists/*

echo "✅ Dependencias base instaladas"
