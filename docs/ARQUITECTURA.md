# Arquitectura - Trust Work Escrow

## 📐 Visión General

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           Trust Work Escrow                                 │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌─────────────┐       ┌─────────────┐       ┌─────────────┐                │
│  │   Cliente   │       │  Freelancer │       │   Árbitro   │                │
│  └──────┬──────┘       └──────┬──────┘       └──────┬──────┘                │
│         │                     │                     │                       │
│         └─────────────────────┼─────────────────────┘                       │
│                               │                                             │
│              ┌────────────────┼────────────────┐                            │
│              ▼                                 ▼                            │
│  ┌────────────────────┐            ┌────────────────────┐                   │
│  │   CLI (Clap)       │            │   TUI (Ratatui)    │                   │
│  │   13 comandos      │            │   Menús + Forms    │                   │
│  └─────────┬──────────┘            └──────────┬─────────┘                   │
│            │                                  │                             │
│            └──────────────┬───────────────────┘                             │
│                           ▼                                                 │
│  ┌─────────────────────────────────────────────────────────────┐            │
│  │                  escrow-core (librería Rust)                │            │
│  │   helpers • PDAs • 13 operaciones • 14 tests unitarios      │            │
│  └──────────────────────────┬──────────────────────────────────┘            │
│                             │                                               │
│                             ▼                                               │
│  ┌─────────────────────────────────────────────────────────────┐            │
│  │                    RPC Connection                           │            │
│  │              (Solana Localnet/Devnet)                       │            │
│  └──────────────────────────┬──────────────────────────────────┘            │
│                             │                                               │
│         ┌──────────────────┼──────────────────┐                             │
│         ▼                  ▼                  ▼                             │
│  ┌─────────────┐   ┌─────────────┐   ┌─────────────┐                        │
│  │   Program   │   │   System    │   │   Config    │                        │
│  │   (Anchor)  │   │   Program   │   │    PDA      │                        │
│  └──────┬──────┘   └─────────────┘   └─────────────┘                        │
│         │                                                                   │
│         ▼                                                                   │
│  ┌─────────────┐   ┌─────────────┐                                          │
│  │  Job PDA    │   │  Treasury   │                                          │
│  │  (State)    │   │  (Fees 5%)  │                                          │
│  └─────────────┘   └─────────────┘                                          │
│                                                                             Lo híkon.│
└─────────────────────────────────────────────────────────────────────────────┘
```

## 🏗️ Componentes

### 1. Smart Contract (Anchor)

#### Responsabilidades
- Gestionar el estado de los trabajos
- Manejar depósitos y retiros de fondos
- Validar reglas de negocio
- Emitir eventos

#### Cuentas
- **Config**: Configuración global (fee, treasury)
- **Job**: Cada trabajo/escrow

### 2. CLI (Rust + Clap)

#### Responsabilidades
- Interfaz de usuario por terminal
- Parsear comandos y argumentos
- Comunicarse con el programa via RPC (usa escrow-core)

#### Estructura
```
cli/
├── src/
│   └── main.rs          # Entry point + 13 subcomandos Clap
└── Cargo.toml           # Deps: escrow-core, clap, anyhow
```

### 3. TUI (Ratatui)

#### Responsabilidades
- Interfaz gráfica interactiva de terminal
- Gestión multi-wallet con roles
- Temas y configuración persistente
- Comunicarse con el programa via RPC (usa escrow-core)

#### Estructura
```
tui/
├── src/
│   ├── main.rs          # Entry point, setup terminal
│   ├── app.rs           # Estado, eventos, navegación, forms
│   ├── ui.rs            # Renderizado de pantallas
│   └── config.rs        # Temas, settings, persistencia
└── Cargo.toml           # Deps: escrow-core, ratatui, crossterm
```

### 4. escrow-core (Librería compartida)

#### Responsabilidades
- Centralizar lógica de interacción con el smart contract
- Derivación de PDAs
- Construcción y envío de transacciones
- Eliminación de duplicación entre CLI y TUI

#### Estructura
```
escrow-core/
├── src/
│   └── lib.rs           # Helpers + 13 operaciones + 14 tests
└── Cargo.toml           # Deps: anchor-client, solana-rpc-client, borsh
```

### 5. Wallet

#### Tipos soportados
- CLI keypair (archivo JSON)
- Hardware wallet (Ledger)
- Wallet browser (via RPC)

## 💰 Flujo de Fondos

```
┌─────────────────────────────────────────────────────────────────────────┐
│                          FLUJO DE FONDOS                                 │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│   CLIENTE                      PROGRAMA                    FREELANCER    │
│                                                                          │
│   10 SOL                         │                            │          │
│      │                           │                            │          │
│      ├── create_job ───────────► │                            │          │
│      │   (10 SOL)               │                            │          │
│      │                           │  (10 SOL en vault)         │          │
│      │                           │                            │          │
│      │                           │                            │          │
│      │                           │◄──── accept_job ──────────┤          │
│      │                           │                            │          │
│      │                           │                            │          │
│      │                           │◄──── submit_work ─────────┤          │
│      │                           │                            │          │
│      │                           │                            │          │
│      ├── approve ───────────────►│                            │          │
│      │                           │                            │          │
│      │                           │   (9.5 SOL neto) ─────────►│          │
│      │                           │   (1.0 SOL fee = 5%×2) ──► treasury  │
│      │                           │                            │          │
│                                                                          │
│   ═══════════════════════════════════════════════════════════════════    │
│                                                                          │
│   DISPUTA (70-30)                                                         │
│                                                                          │
│      ├── dispute ────────────►│◄──── dispute ───────────┤             │
│      │                        │                            │             │
│      │                        │◄──── resolve ─────────────┤             │
│      │                        │                            │             │
│      │                        │   (6.65 SOL neto×70%) ───►│             │
│      │                        │   (2.85 SOL neto×30%) ────►client       │
│      │                        │   (1.0 SOL fee = 5%×2) ──► treasury    │
│      │                        │                            │             │
└─────────────────────────────────────────────────────────────────────────┘
```

## 🔄 Flujo de Estados

```
                    ┌─────────────┐
                    │   Created   │
                    └──────┬──────┘
                           │ create_job
                           ▼
┌─────────────┐      ┌─────────────┐      ┌─────────────┐
│  Cancelled  │◄─────┤ InProgress  │─────►│  Submitted  │
└─────────────┘      └──────┬──────┘      └──────┬──────┘
        │                   │                     │
        │ cancel_job        │ accept_job          │ submit_work
        │                   │                     │
        │                   │                     ▼
        │                   │              ┌─────────────┐      ┌─────────────┐
        │                   │              │   Released  │      │  Disputed   │
        │                   │              └─────────────┘◄────┴──────┬──────┘
        │                   │                     ▲                     │
        │                   │                     │                     │
        │                   │            approve_work            resolve_dispute
        │                   │                     │                     │
        │                   │                     │                     ▼
        │                   │                     │              ┌─────────────┐
        │                   │                     │              │  Resolved   │
        │                   │                     │              └─────────────┘
        │                   │                     │
        └───────────────────┴─────────────────────┘
```

## 📦 Estructura de Datos

### Job Account

```
┌─────────────────────────────────────────────────────────────────────────┐
│                            Job Account                                    │
├─────────────────────────────────────────────────────────────────────────┤
│  Offset  │  Size  │  Field              │  Description                   │
├──────────┼────────┼────────────────────┼────────────────────────────────┤
│  0       │  8     │  discriminator      │  Anchor account discriminator  │
│  8       │  32    │  client            │  Pubkey del cliente            │
│  40      │  32    │  freelancer        │  Pubkey del freelancer         │
│  72      │  32    │  arbiter           │  Pubkey del árbitro            │
│  104     │  8     │  amount            │  Monto total (lamports)        │
│  112     │  8     │  fee               │  Comisión del programa         │
│  120     │  1     │  status            │  Estado del trabajo            │
│  121     │  4     │  title_len         │  Longitud del título           │
│  125     │  100   │  title             │  Título (max 100 bytes)        │
│  225     │  4     │  desc_len          │  Longitud de descripción       │
│  229     │  500   │  description       │  Descripción (max 500 bytes)  │
│  729     │  8     │  deadline          │  Timestamp deadline            │
│  737     │  8     │  created_at       │  Timestamp creación            │
│  745     │  8     │  updated_at        │  Timestamp actualización       │
│  753     │  4     │  dispute_len       │  Longitud razón disputa        │
│  757     │  200   │  dispute_reason    │  Razón de disputa              │
│  957     │  1     │  bump              │  Bump para PDA                 │
└─────────────────────────────────────────────────────────────────────────┘
```

## 🔌 Integración

### Conexión RPC

```rust
// Configuración de conexión
let rpc_url = "https://api.devnet.solana.com";
let connection = Connection::new(rpc_url);
```

### Anchor Client

```rust
// Cargar programa
let program = anchor::Client::new(
    anchor::Provider::new(
        connection,
        wallet,
        anchor::CommitmentConfig::confirmed(),
    ),
    program_id,
);
```

## 📡 Eventos

### On-Chain Events

```
Transaction Log:
├─ Program consumed 12345 compute units
├─ Program data: [event data]
└─ Program returned success
```

### Off-Chain Events (via RPC)

```typescript
// Suscribirse a cambios de cuenta
program.account.job.subscribe(jobId, (job) => {
    console.log('Job updated:', job.status);
});
```

## 🔒 Modelo de Seguridad

### Capas de Seguridad

1. **Validación de Entrada**
   - Tipos de datos correctos
   - Rangos válidos
   - Longitudes máximas

2. **Verificación de Firmas**
   - Solo firmantes autorizados
   - Validación de ownership

3. **Restricciones de Estado**
   - Transiciones de estado válidas
   - Flags de autorización

4. **Gestión de Fondos**
   - Transfers atómicos
   - Cálculo correcto de fees

## 📊 Métricas

### Límites del Programa

| Recurso | Límite |
|---------|--------|
| Título | 100 bytes |
| Descripción | 500 bytes |
| Razón disputa | 200 bytes |
| Máximo fee | 10% |
| deadline | Timestamp futuro |

### Costos Estimados

| Operación | Compute Units |
|-----------|---------------|
| create_job | ~10,000 |
| accept_job | ~5,000 |
| submit_work | ~5,000 |
| approve_work | ~10,000 |
| dispute | ~5,000 |
| resolve | ~15,000 |

## 🚀 Escalabilidad

### Phase 2 (Hackatón)

- [ ] Múltiples vault accounts por job
- [ ] Sistema de milestones
- [ ] Registro de árbitros
- [ ] Tokens SPL (USDC)

### Phase 3 (Producción)

- [ ] Gasless transactions (relayers)
- [ ] Off-chain metadata (IPFS)
- [ ] Sistema de reputación
- [ ] Notifications
