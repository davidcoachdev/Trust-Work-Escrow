# Trust Work Escrow — TUI (Terminal User Interface)

Interfaz de usuario interactiva para terminal, construida con **Ratatui** en Rust.

Permite gestionar todo el ciclo de vida de trabajos en el escrow de Solana desde una interfaz visual en la terminal, con soporte para múltiples wallets, roles y temas personalizables.

---

## Requisitos

- Rust 1.65+  
- Solana CLI + validador local o conexión a devnet  
- Smart contract de Trust Work Escrow desplegado  

---

## Instalación

```bash
cd trust-escrow/tui
cargo build
```

El binario se genera en `target/debug/escrow-tui`.

---

## Ejecución

```bash
# Opción 1: Con cargo
cargo run --manifest-path tui/Cargo.toml

# Opción 2: Binario directo
./tui/target/debug/escrow-tui
```

> **Nota:** Necesitas un validador de Solana corriendo. Para pruebas locales:
> ```bash
> solana-test-validator --reset
> ```

---

## Navegación

| Tecla | Acción |
|-------|--------|
| `↑` / `k` | Mover arriba |
| `↓` / `j` | Mover abajo |
| `Enter` | Seleccionar / Enviar formulario |
| `Esc` | Ir atrás |
| `Tab` | Siguiente campo (en formularios) |
| `Shift+Tab` | Campo anterior (en formularios) |
| `d` | Eliminar wallet (en lista de wallets) |
| `q` | Salir |
| `Ctrl+C` | Salir (desde cualquier pantalla) |

---

## Flujo de Pantallas

```
WalletSelect → RoleSelect → MainMenu → [Operación] → Result → MainMenu
                                  │
                                  ├── Settings → Theme / Network / Wallets
                                  ├── Change Role → RoleSelect
                                  └── Change Wallet → WalletSelect
```

---

## Roles y Operaciones

### 👑 Admin
- **Initialize Config** — Configurar el programa con dirección treasury
- **Pause Program** — Pausar el programa  
- **Unpause Program** — Reanudar el programa
- **Show Job** — Ver detalles de un trabajo

### 💼 Client
- **Create Job** — Crear trabajo (título, monto, descripción, árbitro, ID, deadline)
- **Deposit Funds** — Depositar fondos en un trabajo
- **Approve Work** — Aprobar trabajo y liberar pago al freelancer
- **Reject Work** — Rechazar trabajo y abrir disputa
- **Cancel Job** — Cancelar trabajo (solo antes de que esté en progreso)
- **Show Job** — Ver detalles de un trabajo

### 🔧 Freelancer
- **Accept Job** — Aceptar un trabajo
- **Submit Work** — Marcar trabajo como entregado
- **Raise Dispute** — Abrir disputa (si el cliente no responde)
- **Show Job** — Ver detalles de un trabajo

### ⚖️ Arbiter
- **Resolve Dispute** — Resolver disputa con porcentaje (0-100% para freelancer)
- **Show Job** — Ver detalles de un trabajo

---

## Gestión de Wallets

La TUI soporta **múltiples wallets** para simular diferentes actores:

1. **Agregar wallet**: En la pantalla de selección de wallet, selecciona "➕ Add wallet"
2. **Cambiar wallet**: Desde el menú principal → "👛 Change Wallet"
3. **Eliminar wallet**: En la lista de wallets, presiona `d` sobre la wallet a eliminar

Cada wallet tiene: **nombre**, **path al keypair**, y **rol** asociado.

### Ejemplo: Configurar 3 actores para pruebas

```bash
# Generar keypairs
solana-keygen new --outfile ~/.config/solana/client.json --no-passphrase
solana-keygen new --outfile ~/.config/solana/freelancer.json --no-passphrase
solana-keygen new --outfile ~/.config/solana/arbiter.json --no-passphrase

# Airdrop SOL para pruebas
solana airdrop 5 --keypair ~/.config/solana/client.json
solana airdrop 5 --keypair ~/.config/solana/freelancer.json
solana airdrop 5 --keypair ~/.config/solana/arbiter.json
```

Luego agrega cada wallet en la TUI con su nombre y rol correspondiente.

---

## Temas (Themes)

4 temas preconfigurados, cambiables desde Settings → Theme:

| Tema | Descripción |
|------|-------------|
| **dark** | Fondo oscuro, acentos cyan (por defecto) |
| **light** | Fondo claro, acentos azules |
| **hacker** | Verde sobre negro estilo terminal |
| **ocean** | Tonos azules marinos |

Los temas se aplican en tiempo real y se persisten entre sesiones.

---

## Configuración

La configuración se guarda automáticamente en:

```
~/.config/trust-escrow-tui/config.toml
```

Incluye:
- Tema activo
- URL del RPC
- Lista de wallets con nombres, paths y roles
- Wallet activa

### Ejemplo de config.toml

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

## Flujo Típico de Prueba

```
1. Admin:      Init (configurar treasury)
2. Client:     Create Job → Deposit Funds
3. Freelancer: Accept Job → Submit Work
4. Client:     Approve Work (pago liberado)
   — o —
4. Client:     Reject Work (disputa abierta)
5. Arbiter:    Resolve Dispute (reparto con porcentaje)
```

Para cambiar entre actores, usa "Change Wallet" y selecciona la wallet correspondiente.

---

## Diferencia con el CLI

| Característica | CLI | TUI |
|----------------|-----|-----|
| Interfaz | Comandos de texto | Menús interactivos |
| Navegación | Flags y argumentos | Flechas y Enter |
| Múltiples wallets | Manual (--keypair) | Integrado con switch |
| Roles | Implícito | Explícito con filtro de menú |
| Disputas (raise/resolve) | ✅ Implementado | ✅ Implementado |
| Temas | No aplica | 4 temas personalizables |
| Configuración | Flags cada vez | Persistente en archivo |

Ambos pueden usarse en paralelo sin conflicto.

---

## Estructura de Archivos

```
tui/
├── Cargo.toml          # Dependencias
└── src/
    ├── main.rs         # Entry point, setup terminal, loop principal
    ├── app.rs          # Estado, eventos, navegación, formularios
    ├── ui.rs           # Renderizado de todas las pantallas
    └── config.rs       # Themes, settings, persistencia
```
