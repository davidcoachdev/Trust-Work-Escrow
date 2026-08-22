# Solana LATAM Hackathon - WayLearn 2026

## 📅 Fechas Importantes (Hora Ciudad de México GMT-6)

| Evento | Fecha |
|--------|-------|
| **Registro de participantes** | 9 de marzo (00:00 h) al 19 de marzo (23:59 h) |
| **Inicio de desarrollo** | 20 de marzo (00:00 h) |
| **Periodo de construcción** | Del 20 al 23 de marzo |
| **Fecha límite de entrega** | 23 de marzo (23:59 h) |
| **Resolución de dudas** | 21 y 22 de marzo (11:00-12:00 h y 17:00-18:00 h) |
| **Evaluación (Judging)** | 24 de marzo |
| **Anuncio de ganadores** | 25 de marzo |

## 🏆 Premios

| Lugar | Premio |
|-------|--------|
| 1er Lugar | **$2,500 USDC** |
| 2do Lugar | **$1,500 USDC** |
| 3er Lugar | **$1,000 USDC** |

**Bonus**: 10 proyectos seleccionados para programa de incubación (se ejecutará en mayo).

## 📋 Requisitos de Entrega

- ✅ Proyecto construido en Solana durante el periodo de construcción (20-23 marzo)
- ✅ Desplegado en **devnet** (usar [faucet.solana.com](https://faucet.solana.com/))
- ✅ Incluye backend, cliente conectado y frontend (puede ser wireframe/mockup)
- ✅ Video tutorial breve (máx. 3 min) - usar [Loom](https://www.loom.com/)
- ✅ Repositorio público de GitHub con código fuente funcional
- ✅ Equipo de 1-3 miembros

## 📂 Templates para Empezar

### Backend Template
- **Repo**: [Solana-Hackathon-Template-Backend](https://github.com/WayLearnLatam/Solana-Hackathon-Template-Backend)
- **Incluye**: Rust + Anchor + Codespaces

### FullStack Template (RECOMENDADO)
- **Repo**: [Solana-Hackathon-Template-FullStack](https://github.com/WayLearnLatam/Solana-Hackathon-Template-FullStack)
- **Incluye**: React + Vite + Anchor + Docker devcontainer

## 🎨 Herramientas

- **Figma (mockups)**: https://www.figma.com/es-la/
- **Solana Playground**: https://beta.solpg.io/

## 📂 Estructura del Proyecto (Template FullStack)

```
template_codespaces/
├── programs/              # Smart contracts (Anchor/Rust)
│   └── mi_program/
│       └── src/
│           └── lib.rs
├── app/                   # Frontend (React + Vite)
│   ├── src/
│   ├── components/
│   └── App.tsx
├── tests/                 # Tests
└── anchor.toml            # Config Anchor
```

## 🔧 Comandos Principales

```bash
# Configurar PATH
export PATH="$HOME/.local/share/solana/install/active_release/bin:$PATH"

# Ver dirección de wallet
solana address

# Build del programa
anchor build

# Desplegar a devnet
anchor deploy

# Ejecutar frontend
cd template_codespaces
npm install
npm run dev
```

## 🎯 Criterios de Evaluación

| Criterio | Descripción |
|----------|-------------|
| **Viabilidad Técnica** | ¿Qué tan bien se integra la idea con el ecosistema de Solana? |
| **Prototipo Funcional** | ¿Es la aplicación completamente funcional? |
| **Nivel de Complejidad** | ¿Supera los ejemplos básicos del Bootcamp? Lógica de negocio real, gestión avanzada de estados, validaciones de seguridad robustas |
| **Originalidad y Creatividad** | ¿Qué tan innovador y creativo es el enfoque? |

**Jueces**: Integrantes de Solana Foundation y WayLearn

## 🎮 Categorías del Proyecto

Puedes basarte en estas categorías:

1. **Arte y Música** - NFTs, monetización directa, comunidades
2. **Social** - Identidad on-chain, reputación, DAOs
3. **Gaming** - Economías digitales, jugadores poseen activos
4. **Tickets** - Entradas verificables, transferibles, programables
5. **DAOs** - Gobernanza, votaciones on-chain, tesorerías compartidas
6. **Metaverso** - Espacios virtuales, identidad digital
7. **NFTs** - Coleccionables, membresías, certificados
8. **Fidelización** - Programas de puntos, tokens de recompensas
9. **DeSci** - Ciencia blockchain, financiamiento colectivo
10. **Blinks** - Acciones on-chain desde enlaces/QR
11. **Blue Sky** - Proyectos innovadores que no encajan en las demás

## 🔗 Recursos Útiles

- **Discord para dudas**: https://discord.gg/ueySW6AKSn
- **Canal de faucet Discord**: https://discord.com/channels/1036853569711779880/1484597714862997575
- **DoraHacks**: https://dorahacks.io/hackathon/solana-waylearn-2026/detail

## 📝 Notas para Nuestro Proyecto (Trust Work Escrow)

Basado en nuestro proyecto existente,Podemos orientar el hackathon a:

- **Categoría**: DAOs / Social (sistema de freelancers + clientes)
- **Tech Stack**: Anchor (Rust) + React (Frontend) + nuestro escrow-core
- **Diferenciador**: Sistema de reputación, multi-wallet por usuario, roles derivados del smart contract

### Idea para el Hackathon

**Trust Work Escrow v2** - Sistema de freelancing con:

#### Características:
- Login con wallet (sign-message)
- Perfiles de usuario on-chain (User PDA)
- Múltiples wallets por usuario (como multiboot)
- Roles NO encasillados (puede ser client, freelancer y arbiter)
- Pool de árbitros registrados
- Arbiter asignado automáticamente en disputas

#### Arquitectura:
```
├── Smart Contract (Anchor/Rust)  ← Core del escrow
├── Core Library (Rust)            ← SDK复用
├── Web (React + Vite)             ← Landing + Dashboard
└── CLI (Ratatui)                  ← Login + operaciones
```

#### Diferenciación del Original:

| Aspecto | v1 (existente) | v2 (hackathon) |
|---------|-----------------|----------------|
| UI | TUI (Ratatui) | Web (React) + CLI |
| Login | Keypairs en archivo | Wallet connect |
| Roles | Manual por usuario | Derivado de cuentas on-chain |
| Usuarios | No hay cuenta | User PDA + multi-wallet |
| Perfil | No existe | Username, bio, reputación |
| Arbiter | Elegido al crear job | Pool registrado + auto-asign |
| Gobernanza | Single signer | Multisig 2-of-3 |