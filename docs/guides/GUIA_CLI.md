# Guía de Usuario — CLI (Cliente de Terminal)

Instrucciones para usar la interfaz de línea de comandos de Trust Work Escrow.

---

## Requisitos previos

```bash
# 1. Iniciar el validador local de Solana (en una terminal separada)
solana-test-validator --reset

# 2. Configurar la red local
solana config set --url localhost

# 3. Asegurarte de tener SOL en tu wallet
solana airdrop 5
```

---

## Iniciar

```bash
# Desde la raíz del proyecto
cargo run --manifest-path trust-escrow/cli/Cargo.toml -- [COMANDO] [OPCIONES]

# O desde trust-escrow/
cargo run --manifest-path cli/Cargo.toml -- [COMANDO] [OPCIONES]
```

---

## Opciones globales

| Opción             | Descripción                  | Default                    |
| ------------------ | ---------------------------- | -------------------------- |
| `--keypair <PATH>` | Ruta al keypair del firmante | `~/.config/solana/id.json` |
| `--url <URL>`      | URL del RPC de Solana        | `http://127.0.0.1:8899`    |

---

## Comandos

### `init` — Admin

Inicializa la configuración del programa (solo una vez).

```bash
escrow init --treasury <TREASURY_ADDRESS>
```

---

### `create` — Cliente

Crea un nuevo trabajo.

```bash
escrow create <TITLE> --amount <SOL> --arbiter <ADDRESS> --job-id <ID> [--description <DESC>] [--deadline <TIMESTAMP>]
```

---

### `deposit` — Cliente

Deposita fondos en un trabajo existente.

```bash
escrow deposit <JOB_ID>
```

---

### `accept` — Freelancer

Acepta un trabajo.

```bash
escrow accept <JOB_ID> --client <CLIENT_ADDRESS>
```

---

### `submit` — Freelancer

Marca el trabajo como entregado.

```bash
escrow submit <JOB_ID> --client <CLIENT_ADDRESS>
```

---

### `approve` — Cliente

Aprueba el trabajo y libera los fondos al freelancer.

```bash
escrow approve <JOB_ID> --freelancer <FREELANCER_ADDRESS>
```

---

### `reject` — Cliente

Rechaza el trabajo y abre una disputa.

```bash
escrow reject <JOB_ID> "<REASON>"
```

---

### `raise-dispute` — Freelancer

Levanta una disputa (si el cliente no responde).

```bash
escrow raise-dispute <JOB_ID> --client <CLIENT_ADDRESS> --reason "<REASON>"
```

---

### `resolve-dispute` — Árbitro

Resuelve una disputa con distribución porcentual.

```bash
escrow resolve-dispute <JOB_ID> --client <CLIENT_ADDRESS> --freelancer <FREELANCER_ADDRESS> --freelancer-percent <0-100>
```

---

### `cancel` — Cliente

Cancela un trabajo (solo antes de que esté en progreso).

```bash
escrow cancel <JOB_ID>
```

---

### `show` — Todos

Muestra los detalles de un trabajo.

```bash
escrow show <JOB_ID> --client <CLIENT_ADDRESS>
```

---

### `pause` / `unpause` — Admin

Pausa o reactiva el programa.

```bash
escrow pause
escrow unpause
```

---

### `withdraw-treasury` — Treasury

Retira fondos acumulados de la cuenta treasury hacia cualquier destino.
Solo puede ejecutarlo la wallet que fue registrada como treasury en `init`.

```bash
# Retirar 1.5 SOL hacia la misma wallet treasury
escrow --keypair ~/.config/solana/treasury.json withdraw-treasury 1.5

# Retirar hacia otra wallet
escrow --keypair ~/.config/solana/treasury.json withdraw-treasury 1.5 --destination <DEST_ADDRESS>
```

> **Seguridad**: El Admin no puede ejecutar este comando. La separación de roles
> evita que una sola clave comprometida pueda pausar el programa Y drenar los fondos.

---

## Flujo completo de ejemplo

```bash
# 1. Admin: inicializar (solo una vez)
escrow init --treasury $(solana-keygen pubkey ~/.config/solana/treasury.json)

# 2. Cliente: crear trabajo y depositar fondos
escrow create "Landing Page" --amount 2 --arbiter <ARB_ADDR> --job-id 1 --description "React landing"
escrow deposit 1

# 3. Freelancer: aceptar y entregar
escrow accept 1 --client <CLIENT_ADDR>
escrow submit 1 --client <CLIENT_ADDR>

# 4a. Cliente: aprobar (libera el pago automáticamente)
escrow approve 1 --freelancer <FL_ADDR>

# 4b. O en caso de disputa:
escrow reject 1 "Trabajo incompleto"
# Árbitro resuelve: 70% freelancer, 30% cliente
escrow resolve-dispute 1 --client <CLIENT_ADDR> --freelancer <FL_ADDR> --freelancer-percent 70

# 5. Treasury: retirar fees acumuladas
escrow --keypair ~/.config/solana/treasury.json withdraw-treasury 0.5
```

---

## Estados del trabajo

```
Created → Funded → InProgress → Submitted → Completed
                                     ↓
                                 Disputed → Resolved
     ↓
  Cancelled
```

| Estado       | Descripción                                     |
| ------------ | ----------------------------------------------- |
| `Created`    | Trabajo creado, esperando depósito o freelancer |
| `Funded`     | Fondos depositados, esperando freelancer        |
| `InProgress` | Freelancer aceptó, está trabajando              |
| `Submitted`  | Freelancer entregó el trabajo                   |
| `Completed`  | Completado, fondos liberados                    |
| `Disputed`   | En disputa                                      |
| `Resolved`   | Resuelto por el árbitro                         |
| `Cancelled`  | Cancelado (refund al cliente)                   |

---

## Errores comunes

| Error                | Descripción                               |
| -------------------- | ----------------------------------------- |
| `NotAuthorized`      | No tienes permisos para esta acción       |
| `InvalidJobStatus`   | El trabajo no está en el estado requerido |
| `NotJobFreelancer`   | No eres el freelancer asignado            |
| `ProgramPaused`      | El programa está pausado                  |
| `TitleTooLong`       | Título excede 100 caracteres              |
| `AmountTooSmall`     | Monto menor al mínimo (0.0001 SOL)        |
| `EmptyDisputeReason` | Razón de disputa vacía                    |
