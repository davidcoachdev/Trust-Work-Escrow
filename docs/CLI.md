# Guía de Comandos CLI

## Uso General

```bash
# Desde trust-escrow/
cargo run --manifest-path cli/Cargo.toml -- [COMMAND] [OPTIONS]

# O si compilaste el binario:
escrow [COMMAND] [OPTIONS]
```

### Opciones Globales

| Opción | Descripción | Default |
|--------|-------------|---------|
| `--keypair <PATH>` | Ruta al keypair del firmante | `~/.config/solana/id.json` |
| `--url <URL>` | URL del RPC de Solana | `http://127.0.0.1:8899` |

---

## Comandos Disponibles

### init

Inicializa la configuración del programa (solo admin).

```bash
escrow init --treasury <TREASURY_ADDRESS>
```

**Opciones:**
- `--treasury` — Dirección de la cuenta treasury (required)

**Ejemplo:**
```bash
escrow init --treasury 7x8f9KpJ...
```

---

### create

Crea un nuevo trabajo (job).

```bash
escrow create <TITLE> --amount <SOL> --arbiter <ADDRESS> --job-id <ID> [--description <DESC>] [--deadline <TIMESTAMP>]
```

**Argumentos:**
- `TITLE` — Título del trabajo (required)

**Opciones:**
- `--amount` — Monto en SOL (required)
- `--arbiter` — Dirección del árbitro (required)
- `--job-id` — ID numérico del trabajo (required)
- `--description` — Descripción (optional, default: "")
- `--deadline` — Unix timestamp del plazo (optional, default: +7 días)

**Ejemplo:**
```bash
escrow create "Landing Page" --amount 2 --arbiter 7x8f9... --job-id 1 --description "React landing"
```

---

### deposit

Deposita fondos en un trabajo existente.

```bash
escrow deposit <JOB_ID>
```

**Argumentos:**
- `JOB_ID` — ID del trabajo (required)

**Ejemplo:**
```bash
escrow deposit 1
```

---

### accept

Acepta un trabajo como freelancer.

```bash
escrow accept <JOB_ID> --client <CLIENT_ADDRESS>
```

**Argumentos:**
- `JOB_ID` — ID del trabajo (required)

**Opciones:**
- `--client` — Dirección del cliente que creó el trabajo (required)

**Ejemplo:**
```bash
escrow accept 1 --client 4zM3...
```

---

### submit

Marca el trabajo como entregado.

```bash
escrow submit <JOB_ID> --client <CLIENT_ADDRESS>
```

**Argumentos:**
- `JOB_ID` — ID del trabajo (required)

**Opciones:**
- `--client` — Dirección del cliente (required)

**Ejemplo:**
```bash
escrow submit 1 --client 4zM3...
```

---

### approve

Aprueba el trabajo y libera los fondos al freelancer.

```bash
escrow approve <JOB_ID> --freelancer <FREELANCER_ADDRESS>
```

**Argumentos:**
- `JOB_ID` — ID del trabajo (required)

**Opciones:**
- `--freelancer` — Dirección del freelancer (required)

**Ejemplo:**
```bash
escrow approve 1 --freelancer 9aB7...
```

---

### reject

Rechaza el trabajo y abre una disputa (solo cliente).

```bash
escrow reject <JOB_ID> <REASON>
```

**Argumentos:**
- `JOB_ID` — ID del trabajo (required)
- `REASON` — Razón del rechazo (required)

**Ejemplo:**
```bash
escrow reject 1 "Trabajo incompleto"
```

---

### raise-dispute

Levanta una disputa como freelancer.

```bash
escrow raise-dispute <JOB_ID> --client <CLIENT_ADDRESS> --reason <REASON>
```

**Argumentos:**
- `JOB_ID` — ID del trabajo (required)

**Opciones:**
- `--client` — Dirección del cliente (required)
- `--reason` — Razón de la disputa (required)

**Ejemplo:**
```bash
escrow raise-dispute 1 --client 4zM3... --reason "Cliente no responde"
```

---

### resolve-dispute

Resuelve una disputa como árbitro.

```bash
escrow resolve-dispute <JOB_ID> --client <CLIENT_ADDRESS> --freelancer <FREELANCER_ADDRESS> --freelancer-percent <0-100>
```

**Argumentos:**
- `JOB_ID` — ID del trabajo (required)

**Opciones:**
- `--client` — Dirección del cliente (required)
- `--freelancer` — Dirección del freelancer (required)
- `--freelancer-percent` — Porcentaje para el freelancer (0-100, required)

**Ejemplo:**
```bash
escrow resolve-dispute 1 --client 4zM3... --freelancer 9aB7... --freelancer-percent 70
```

---

### cancel

Cancela un trabajo (solo cliente, antes de in-progress).

```bash
escrow cancel <JOB_ID>
```

**Argumentos:**
- `JOB_ID` — ID del trabajo (required)

**Ejemplo:**
```bash
escrow cancel 1
```

---

### show

Muestra los detalles de un trabajo.

```bash
escrow show <JOB_ID> --client <CLIENT_ADDRESS>
```

**Argumentos:**
- `JOB_ID` — ID del trabajo (required)

**Opciones:**
- `--client` — Dirección del cliente que creó el trabajo (required)

**Ejemplo:**
```bash
escrow show 1 --client 4zM3...
```

---

### pause

Pausa el programa (solo admin).

```bash
escrow pause
```

---

### unpause

Reactiva el programa (solo admin).

```bash
escrow unpause
```

---

## Estados del Trabajo

| Estado | Descripción |
|--------|-------------|
| `Created` | Creado, esperando depósito o freelancer |
| `Funded` | Fondos depositados, esperando freelancer |
| `InProgress` | Freelancer aceptó, trabajando |
| `Submitted` | Freelancer entregó el trabajo |
| `Completed` | Completado, fondos liberados |
| `Disputed` | En disputa |
| `Resolved` | Resuelto por árbitro |
| `Cancelled` | Cancelado (refund al cliente) |

---

## Flujo Típico

```
Cliente                       Freelancer                    Árbitro
   │                             │                              │
   ├── init ──────────────────►  │                              │
   │   (solo 1 vez, admin)       │                              │
   │                             │                              │
   ├── create + deposit ───────► │                              │
   │                             │                              │
   │                             ├── accept ──────────────────► │
   │                             │                              │
   │                             ├── submit ──────────────────► │
   │                             │                              │
   ├── approve ─────────────────►│                              │
   │   (libera fondos)           │                              │
   │                             │                              │
   │  ── o en caso de disputa ─  │                              │
   │                             │                              │
   ├── reject ─────────────────► │                              │
   │   (o)                       ├── raise-dispute ───────────► │
   │                             │                              │
   │                             │    ◄── resolve-dispute ──────┤
   │                             │       (árbitro decide %)     │
```

---

## Errores Comunes

| Error Anchor | Descripción |
|--------------|-------------|
| `NotAuthorized` | No tienes permisos para esta acción |
| `InvalidJobStatus` | El trabajo no está en el estado requerido |
| `NotJobFreelancer` | No eres el freelancer asignado |
| `InvalidTreasury` | Treasury no coincide con la configuración |
| `ProgramPaused` | El programa está pausado |
| `TitleTooLong` | Título excede 100 caracteres |
| `AmountTooSmall` | Monto menor al mínimo (0.0001 SOL) |
| `EmptyDisputeReason` | Razón de disputa vacía |
