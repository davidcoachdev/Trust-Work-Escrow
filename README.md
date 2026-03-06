# Trust Work Escrow 🛡️

[![Solana](https://img.shields.io/badge/Solana-3.0-9945FF?logo=solana&logoColor=white)](https://solana.com)
[![Anchor](https://img.shields.io/badge/Anchor-0.32-blue)](https://www.anchor-lang.com)
[![Rust](https://img.shields.io/badge/Rust-1.89-orange?logo=rust)](https://www.rust-lang.org)
[![Node.js](https://img.shields.io/badge/Node.js-20_LTS-339933?logo=node.js&logoColor=white)](https://nodejs.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](./LICENSE)

> Sistema de pagos escrow descentralizado para freelancers y clientes, construido sobre Solana.

---

## 🧠 Problemática

En el trabajo remoto, tanto freelancers como clientes enfrentan riesgos:

- El **cliente paga** pero no recibe el trabajo acordado
- El **freelancer entrega** el trabajo pero no recibe el pago
- No existe un mecanismo **neutral y transparente** que proteja a ambas partes
- Los servicios de escrow tradicionales cobran comisiones altas (3-10%) y son lentos

## 💡 Solución

**Trust Work Escrow** es una plataforma de escrow on-chain donde:

1. El **cliente deposita** fondos en un vault seguro (PDA en Solana)
2. El **freelancer trabaja** y marca como entregado
3. El **cliente aprueba** → el pago se libera automáticamente
4. Si hay **desacuerdo** → un **árbitro** resuelve la disputa con distribución porcentual

| Escrow Tradicional           | Trust Work Escrow                     |
| ---------------------------- | ------------------------------------- |
| Centralizado (banco/tercero) | Descentralizado (código = confianza)  |
| Comisiones altas (3-10%)     | Comisión fija del 5%                  |
| Lento (días/semanas)         | Instantáneo en Solana                 |
| Requiere KYC                 | Solo wallet                           |
| Decisión unilateral          | Arbitraje con porcentaje configurable |

---

## 🛠️ Tecnologías

| Tecnología     | Versión | Uso                                 |
| -------------- | ------- | ----------------------------------- |
| **Rust**       | 1.89    | Lenguaje base                       |
| **Anchor**     | 0.32.1  | Smart contract framework            |
| **Solana CLI** | 3.0.15  | Interacción con la blockchain       |
| **Ratatui**    | 0.28    | TUI (interfaz gráfica de terminal)  |
| **Clap**       | 4.5     | CLI (interfaz de línea de comandos) |
| **Node.js**    | 20 LTS  | Tests de integración (TypeScript)   |
| **Borsh**      | 1       | Serialización de datos on-chain     |

---

## 📁 Estructura del Proyecto

```
Trust-Work-Escrow/
├── trust-escrow/                # Proyecto Anchor principal
│   ├── programs/trust-escrow/   # Smart contract (Rust/Anchor)
│   │   └── src/lib.rs           # 12 instrucciones on-chain
│   ├── escrow-core/             # Librería compartida (Rust)
│   │   └── src/lib.rs           # Helpers + 13 operaciones + 14 tests
│   ├── cli/                     # CLI (Clap + escrow-core)
│   │   └── src/main.rs          # 13 comandos
│   ├── tui/                     # TUI (Ratatui + escrow-core)
│   │   └── src/                 # 4 módulos (app, ui, config, main)
│   ├── tests/                   # Tests de integración (TypeScript)
│   │   └── trust-escrow.ts      # 23 tests
│   └── Anchor.toml
├── docs/                        # Documentación completa
├── scripts/                     # Scripts de deploy y backup
└── README.md
```

---

## 🚀 Inicio Rápido

### Prerrequisitos

- Dev Container (recomendado) o:
  - Rust 1.89+, Solana CLI 3.0+, Anchor 0.32+, Node.js 20+

### Instalación

```bash
# 1. Clonar repositorio
git clone https://github.com/tu-usuario/Trust-Work-Escrow.git
cd Trust-Work-Escrow

# 2. Instalar dependencias de tests
cd trust-escrow && yarn install

# 3. Build del smart contract
anchor build

# 4. Build del CLI y TUI
cd cli && cargo build && cd ..
cd tui && cargo build && cd ..
```

### Configurar Solana (Local)

```bash
solana config set --url localhost
solana-test-validator --reset    # En otra terminal
solana airdrop 5
```

### Deploy

```bash
anchor deploy
```

---

## 💻 CLI — Interfaz de Línea de Comandos

```bash
# Desde trust-escrow/
cargo run --manifest-path cli/Cargo.toml -- [COMANDO] [OPCIONES]

# Opciones globales:
#   --keypair <PATH>   Ruta al keypair (default: ~/.config/solana/id.json)
#   --url <URL>        RPC URL (default: http://127.0.0.1:8899)
```

### Comandos disponibles (13)

| Comando                                                                                     | Rol        | Descripción               |
| ------------------------------------------------------------------------------------------- | ---------- | ------------------------- |
| `init --treasury <ADDR>`                                                                    | Admin      | Inicializar configuración |
| `create <TITLE> --amount <SOL> --arbiter <ADDR> --job-id <ID>`                              | Client     | Crear trabajo             |
| `deposit <JOB_ID>`                                                                          | Client     | Depositar fondos          |
| `accept <JOB_ID> --client <ADDR>`                                                           | Freelancer | Aceptar trabajo           |
| `submit <JOB_ID> --client <ADDR>`                                                           | Freelancer | Entregar trabajo          |
| `approve <JOB_ID> --freelancer <ADDR>`                                                      | Client     | Aprobar y pagar           |
| `reject <JOB_ID> <REASON>`                                                                  | Client     | Rechazar → disputa        |
| `raise-dispute <JOB_ID> --client <ADDR> --reason <TEXT>`                                    | Freelancer | Levantar disputa          |
| `resolve-dispute <JOB_ID> --client <ADDR> --freelancer <ADDR> --freelancer-percent <0-100>` | Arbiter    | Resolver disputa          |
| `cancel <JOB_ID>`                                                                           | Client     | Cancelar trabajo          |
| `show <JOB_ID> --client <ADDR>`                                                             | Todos      | Ver detalles              |
| `pause`                                                                                     | Admin      | Pausar programa           |
| `unpause`                                                                                   | Admin      | Reanudar programa         |

### Ejemplo de flujo completo

```bash
# Admin: inicializar
escrow init --treasury <TREASURY_ADDR>

# Cliente: crear y depositar
escrow create "Landing Page" --amount 2 --arbiter <ARB_ADDR> --job-id 1
escrow deposit 1

# Freelancer: aceptar y entregar
escrow accept 1 --client <CLIENT_ADDR>
escrow submit 1 --client <CLIENT_ADDR>

# Cliente: aprobar (pago automático)
escrow approve 1 --freelancer <FL_ADDR>
```

---

## 🖥️ TUI — Interfaz Gráfica de Terminal

Interfaz interactiva con menús, formularios, 4 roles y temas visuales.

```bash
cargo run --manifest-path tui/Cargo.toml
```

### Características

- **4 roles** con menú contextual: Admin, Client, Freelancer, Arbiter
- **Multi-wallet** — switch entre wallets sin salir
- **4 temas** — dark, light, hacker, ocean
- **Configuración persistente** en `~/.config/trust-escrow-tui/config.toml`
- Navegación con flechas, vim (`hjkl`), Tab en formularios

---

## 🔗 Smart Contract

**Program ID:** `5gu5JCSpB8MKyJzhXpGaCt8SruAMnRD6cTPbwPX6JTYo`

### 12 Instrucciones

| Instrucción         | Descripción                                   |
| ------------------- | --------------------------------------------- |
| `initialize_config` | Configurar treasury y fee (5%)                |
| `create_job`        | Crear trabajo con PDA único                   |
| `deposit_funds`     | Depositar SOL en el vault PDA                 |
| `accept_job`        | Freelancer acepta el trabajo                  |
| `submit_work`       | Freelancer marca como entregado               |
| `approve_work`      | Cliente aprueba → pago al freelancer - 5% fee |
| `reject_work`       | Cliente rechaza → abre disputa                |
| `raise_dispute`     | Freelancer levanta disputa                    |
| `resolve_dispute`   | Árbitro resuelve con % configurable           |
| `cancel_job`        | Cancelar y devolver fondos                    |
| `pause_program`     | Admin pausa todo el programa                  |
| `unpause_program`   | Admin reactiva el programa                    |

### Estados del Trabajo

```
Created → Funded → InProgress → Submitted → Completed
                                     ↓
                                 Disputed → Resolved
     ↓
  Cancelled
```

---

## 🧪 Tests

```bash
# Tests de integración (TypeScript) — 23 tests
cd trust-escrow && anchor test

# Tests unitarios de escrow-core (Rust) — 14 tests
cd escrow-core && cargo test
```

### Cobertura de tests

- ✅ Inicialización de config (happy + duplicate)
- ✅ Crear trabajo (happy + validación título/monto)
- ✅ Depósito de fondos
- ✅ Aceptar trabajo (happy + auto-accept bloqueado)
- ✅ Submit work
- ✅ Approve + verificación de balances y fee
- ✅ Reject → dispute flow completo
- ✅ Raise dispute por freelancer (happy + impostor + razón vacía)
- ✅ Resolve dispute por árbitro (70/30 + 100/0)
- ✅ Cancel (sin fondos + con fondos/refund)
- ✅ Pause/Unpause + bloqueo de operaciones
- ✅ PDAs, discriminadores, keypaths (unitarios)

---

## 📖 Documentación

| Documento                                          | Contenido                             |
| -------------------------------------------------- | ------------------------------------- |
| [docs/PROYECTO.md](./docs/PROYECTO.md)             | Ideación, problemática, MVP           |
| [docs/ARQUITECTURA.md](./docs/ARQUITECTURA.md)     | Diagramas, flujo de fondos, seguridad |
| [docs/SMARTCONTRACT.md](./docs/SMARTCONTRACT.md)   | Cuentas, instrucciones, errores, PDAs |
| [docs/CLI.md](./docs/CLI.md)                       | 13 comandos con ejemplos              |
| [docs/TUI.md](./docs/TUI.md)                       | Navegación, roles, temas, config      |
| [docs/ESCROW_CORE.md](./docs/ESCROW_CORE.md)       | API de la librería compartida         |
| [docs/LIBRERIA.md](./docs/LIBRERIA.md)             | Explicación del smart contract        |
| [docs/INSTALL.md](./docs/INSTALL.md)               | Guía de instalación                   |
| [docs/FASE3_TREASURY.md](./docs/FASE3_TREASURY.md) | Roadmap fase 3                        |

### 🖥️ Guías de usuario

| Guía                                           | Contenido                                                   |
| ---------------------------------------------- | ----------------------------------------------------------- |
| [docs/DEPLOY_LOCAL.md](./docs/DEPLOY_LOCAL.md) | Deploy e inicialización del programa en local (obligatorio) |
| [docs/GUIA_CLI.md](./docs/GUIA_CLI.md)         | Instrucciones paso a paso para el cliente de terminal (CLI) |
| [docs/GUIA_TUI.md](./docs/GUIA_TUI.md)         | Instrucciones paso a paso para la UI de terminal (TUI)      |

---

## ✅ Certificación

Proyecto para **WayLearn Solana Developer Certification**.

- ✅ Proyecto libre (MIT License)
- ✅ Desarrollado en Rust + Anchor
- ✅ CRUD completo + PDA implementado
- ✅ CLI + TUI funcionales
- ✅ Librería compartida con tests
- ✅ 37 tests totales (23 integración + 14 unitarios)
- ✅ Documentación completa

---

## 📄 Licencia

MIT
