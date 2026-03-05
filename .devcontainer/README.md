# Dev Container — Solana Development Template

Entorno de desarrollo **100% reproducible** para proyectos:

- Rust
- Solana
- Anchor
- Node.js

Compatible con:

- VS Code (local)
- GitHub Codespaces
- Dev Containers Spec (https://containers.dev/)

---

## 🎯 Objetivo

- 🔁 Reproducibilidad total entre equipos
- 🔒 Versionado explícito de toolchains
- 🚀 Onboarding en minutos
- 🧩 Extensibilidad sin modificar el core

Infraestructura versionada como parte del proyecto.

---

## 🏗 Arquitectura del entorno

```text
Ubuntu 24.04 (Dockerfile)
        ↓
  Rust + Solana CLI + Anchor CLI + Node.js
        ↓
  devcontainer.json (configuración VS Code, puertos, extensiones)
        ↓
  postCreateCommand → scripts/post-create.sh (verificación de herramientas)
        ↓
  Contenedor listo para crear proyectos
```

### Archivos

| Archivo                  | Responsabilidad                                           |
| ------------------------ | --------------------------------------------------------- |
| `Dockerfile`             | Instalación de todas las herramientas en build time       |
| `devcontainer.json`      | Configuración de VS Code, puertos, extensiones y settings |
| `scripts/post-create.sh` | Verificación de herramientas instaladas                   |
| `README.md`              | Esta documentación                                        |

---

## 📋 Requisitos

| Herramienta              | Versión mínima |
| ------------------------ | -------------- |
| Docker Desktop           | 4.x            |
| VS Code                  | 1.85+          |
| Dev Containers Extension | última         |

En GitHub Codespaces no necesitás nada local.

---

## 🚀 Cómo abrir el proyecto

### VS Code (local)

1. Clonar el repositorio
2. Abrir en VS Code
3. `Ctrl + Shift + P` → **Dev Containers: Reopen in Container**

Primera build: ~10 minutos (Anchor se compila desde fuente).
Rebuild posterior: mucho más rápido por cache.

### GitHub Codespaces

1. Code → Codespaces → Create
2. El prebuild acelera el arranque (~1 min si ya está cacheado)

---

## 📦 Imagen base

```text
ubuntu:24.04
```

Incluye GLIBC_2.39, requerida por Anchor ≥ 0.32.

---

## 🔧 Toolchains instaladas

| Tool        | Versión                                     |
| ----------- | ------------------------------------------- |
| Rust stable | Última estable                              |
| Node.js     | 20.x LTS                                    |
| Yarn        | 1.x                                         |
| Solana CLI  | v3.0.15 (configurable en devcontainer.json) |
| Anchor CLI  | latest vía AVM                              |
| OpenCode    | latest (AI coding agent)                   |
| Surfpool    | latest (Solana local dev network)          |

---

## 🔌 Puertos expuestos

| Puerto | Servicio                |
| ------ | ----------------------- |
| 3000   | Frontend (Next.js)      |
| 3001   | Frontend alternativo    |
| 8080   | Backend API (Rust/Axum) |
| 8899   | Solana RPC HTTP         |
| 8900   | Solana WebSocket        |
| 9900   | Solana Faucet           |
| 5432   | PostgreSQL (futuro)     |

---

## 🛠 Comandos útiles

```bash
# Verificar herramientas
rustc --version && solana --version && anchor --version && node --version

# Compilar programa Solana
anchor build

# Ejecutar tests
anchor test

# Iniciar validador local (Solana)
solana-test-validator

# Iniciar red local (Surfpool - alternativas a Solana Test Validator)
surfpool start

# Usar OpenCode AI coding agent
opencode

# Crear proyecto Anchor
anchor init mi-proyecto

# Crear API Rust
cargo init backend
```

---

## 🔁 Rebuild del contenedor

Necesario cuando se modifica `Dockerfile` o `devcontainer.json`:

```text
Ctrl+Shift+P → Dev Containers: Rebuild Container
```

Sin cache:

```text
Dev Containers: Rebuild Container Without Cache
```

---

## 🧪 Troubleshooting

| Problema                       | Solución                             |
| ------------------------------ | ------------------------------------ |
| `solana` o `anchor` no en PATH | `source ~/.bashrc`                   |
| Error de permisos en cargo     | `sudo chmod -R a+w /usr/local/cargo` |
| Entorno corrupto               | Rebuild Without Cache                |

---

## 📚 Más información

Ver el `README.md` en la raíz del repositorio para guías de creación de proyectos.
