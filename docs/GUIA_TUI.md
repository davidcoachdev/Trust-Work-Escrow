# Guía de Usuario — TUI (Interfaz Gráfica de Terminal)

Instrucciones para usar la interfaz interactiva de Trust Work Escrow en la terminal.

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
cargo run --manifest-path trust-escrow/tui/Cargo.toml

# O desde trust-escrow/
cargo run --manifest-path tui/Cargo.toml

# O con el binario compilado
./trust-escrow/tui/target/debug/escrow-tui
```

---

## Teclado

| Tecla       | Acción                           |
| ----------- | -------------------------------- |
| `↑` / `k`   | Mover arriba                     |
| `↓` / `j`   | Mover abajo                      |
| `Enter`     | Seleccionar / Enviar formulario  |
| `Esc`       | Ir atrás                         |
| `Tab`       | Siguiente campo (en formularios) |
| `Shift+Tab` | Campo anterior (en formularios)  |
| `d`         | Eliminar wallet seleccionada     |
| `q`         | Salir                            |
| `Ctrl+C`    | Salir (desde cualquier pantalla) |

---

## Flujo de pantallas

```
WalletSelect → RoleSelect → MainMenu → [Operación] → Result → MainMenu
                                  │
                                  ├── Settings → Theme / Network / Wallets
                                  ├── Change Role → RoleSelect
                                  └── Change Wallet → WalletSelect
```

---

## Roles y operaciones disponibles

| Rol               | Operaciones                                                                |
| ----------------- | -------------------------------------------------------------------------- |
| 👑 **Admin**      | Initialize Config, Pause Program, Unpause Program, Show Job                |
| 💼 **Client**     | Create Job, Deposit Funds, Approve Work, Reject Work, Cancel Job, Show Job |
| 🔧 **Freelancer** | Accept Job, Submit Work, Raise Dispute, Show Job                           |
| ⚖️ **Arbiter**    | Resolve Dispute, Show Job                                                  |
| 💰 **Treasury**   | Withdraw Funds (retirar fees acumuladas hacia cualquier destino)           |

---

## Gestión de wallets

La TUI soporta múltiples wallets para simular distintos actores sin salir de la app.

1. **Agregar wallet** — En la pantalla de selección de wallet, elige "➕ Add wallet"
2. **Cambiar wallet** — Menú principal → "👛 Change Wallet"
3. **Eliminar wallet** — En la lista de wallets, presiona `d`

### Generar wallets para pruebas

```bash
solana-keygen new --outfile ~/.config/solana/client.json --no-passphrase
solana-keygen new --outfile ~/.config/solana/freelancer.json --no-passphrase
solana-keygen new --outfile ~/.config/solana/arbiter.json --no-passphrase
solana-keygen new --outfile ~/.config/solana/treasury.json --no-passphrase

solana airdrop 5 --keypair ~/.config/solana/client.json
solana airdrop 5 --keypair ~/.config/solana/freelancer.json
solana airdrop 5 --keypair ~/.config/solana/arbiter.json
solana airdrop 5 --keypair ~/.config/solana/treasury.json
```

Luego agrega cada wallet en la TUI con su nombre y rol correspondiente.

> **Seguridad**: El rol **Treasury** está separado del Admin intencionalmente. Para retirar
> fees acumuladas debes usar la wallet `treasury.json`. El Admin no puede mover esos fondos.

---

## Temas

Cambiables desde **Settings → Theme**:

| Tema     | Descripción                              |
| -------- | ---------------------------------------- |
| `dark`   | Fondo oscuro, acentos cyan (por defecto) |
| `light`  | Fondo claro, acentos azules              |
| `hacker` | Verde sobre negro estilo terminal        |
| `ocean`  | Tonos azules marinos                     |
| `dcdev`  | Rojo oscuro / crimson                    |

Los temas se aplican en tiempo real y se persisten entre sesiones.

---

## Configuración persistente

La TUI guarda su configuración automáticamente en:

```
~/.config/trust-escrow-tui/config.toml
```

Ejemplo:

```toml
theme = "dark"
rpc_url = "http://127.0.0.1:8899"
active_wallet = 0

[[wallets]]
name = "Admin"
path = "/home/user/.config/solana/id.json"
role = "admin"

[[wallets]]
name = "Client"
path = "/home/user/.config/solana/client.json"
role = "client"

[[wallets]]
name = "Freelancer"
path = "/home/user/.config/solana/freelancer.json"
role = "freelancer"
```

---

## Flujo típico de prueba

```
1. Admin      → Initialize Config (configurar treasury)
2. Client     → Create Job → Deposit Funds
3. Freelancer → Accept Job → Submit Work
4. Client     → Approve Work   ← pago liberado
   — o —
4. Client     → Reject Work    ← disputa abierta
5. Arbiter    → Resolve Dispute (elige el % para cada parte)
```

Para cambiar de actor: **Menú principal → Change Wallet**.

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
