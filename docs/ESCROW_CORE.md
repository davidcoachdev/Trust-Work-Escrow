# 📦 escrow-core — Librería Compartida

## Descripción

`escrow-core` es un crate de Rust que centraliza toda la lógica de interacción con el smart contract Trust Work Escrow en Solana. Es utilizada tanto por el **CLI** como por el **TUI**, eliminando duplicación de código.

## Ubicación

```
trust-escrow/escrow-core/
├── Cargo.toml
└── src/
    └── lib.rs
```

## Dependencias

| Crate               | Versión | Uso                    |
| ------------------- | ------- | ---------------------- |
| `anchor-client`     | 0.32.1  | SDK de Solana + tipos  |
| `solana-rpc-client` | 2.3.1   | Conexión RPC           |
| `borsh`             | 1       | Serialización de datos |
| `anyhow`            | 1       | Manejo de errores      |

## API Pública

### Constantes

- `PROGRAM_ID` — ID del programa desplegado (`5gu5JCSpB8MKyJzhXpGaCt8SruAMnRD6cTPbwPX6JTYo`)

### Helpers

| Función                                       | Descripción                                               |
| --------------------------------------------- | --------------------------------------------------------- |
| `kp_path(p: &Option<String>) -> String`       | Retorna ruta del keypair (default o custom)               |
| `load_keypair(path: &str) -> Result<Keypair>` | Carga un keypair desde archivo JSON                       |
| `make_rpc(url: &str) -> RpcClient`            | Crea cliente RPC con commitment `confirmed`               |
| `program_id() -> Result<Pubkey>`              | Parsea el PROGRAM_ID a Pubkey                             |
| `config_pda(pid: &Pubkey) -> Pubkey`          | Deriva PDA de configuración (`seeds: [b"config"]`)        |
| `job_pda(pid, client, job_id) -> Pubkey`      | Deriva PDA de trabajo (`seeds: [b"job", client, job_id]`) |
| `disc(name: &str) -> [u8; 8]`                 | Calcula discriminador Anchor para una instrucción         |
| `now_ts() -> i64`                             | Timestamp actual (epoch seconds)                          |
| `send(rpc, payer, ix) -> Result<String>`      | Firma y envía una transacción                             |

### Operaciones (op\_\*)

Cada función corresponde a una instrucción del smart contract:

| Función                                                         | Instrucción         | Descripción                              |
| --------------------------------------------------------------- | ------------------- | ---------------------------------------- |
| `op_init(rpc, payer, treasury)`                                 | `initialize_config` | Inicializa la configuración del programa |
| `op_create_job(rpc, payer, id, title, desc, amount, arbiter)`   | `create_job`        | Crea un nuevo trabajo                    |
| `op_deposit(rpc, payer, client_pk, job_id)`                     | `deposit`           | Deposita fondos en el escrow             |
| `op_accept(rpc, payer, client_pk, job_id)`                      | `accept_job`        | Freelancer acepta el trabajo             |
| `op_submit(rpc, payer, client_pk, job_id)`                      | `submit_work`       | Freelancer envía el trabajo              |
| `op_approve(rpc, payer, client_pk, job_id, freelancer_pk)`      | `approve_work`      | Cliente aprueba el trabajo               |
| `op_reject(rpc, payer, client_pk, job_id, freelancer_pk)`       | `reject_work`       | Cliente rechaza el trabajo               |
| `op_raise_dispute(rpc, payer, client_pk, job_id)`               | `raise_dispute`     | Levanta una disputa                      |
| `op_resolve_dispute(rpc, payer, client_pk, job_id, fl, winner)` | `resolve_dispute`   | Árbitro resuelve disputa                 |
| `op_cancel(rpc, payer, job_id)`                                 | `cancel_job`        | Cancela un trabajo                       |
| `op_show(rpc, client_pk, job_id)`                               | — (lectura)         | Muestra datos del trabajo                |
| `op_pause(rpc, payer)`                                          | `pause`             | Pausa el programa                        |
| `op_unpause(rpc, payer)`                                        | `unpause`           | Reactiva el programa                     |

### Struct JobInfo

Estructura para deserializar datos de una cuenta de trabajo:

```rust
pub struct JobInfo {
    pub discriminator: [u8; 8],
    pub client: Pubkey,
    pub freelancer: Pubkey,
    pub arbiter: Pubkey,
    pub amount: u64,
    pub status: u8,
    pub title: String,
    pub description: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub job_id: u64,
    pub dispute_reason: String,
}
```

### Re-exports

- `Signer` — trait de `solana_sdk` (necesario para `.pubkey()` en `Keypair`)

## Tests

14 tests unitarios cubren:

- Determinismo de PDAs (config y job)
- Determinismo del discriminador
- Ruta por defecto del keypair
- Ruta custom del keypair
- Display de JobInfo (normal y con disputa)
- Validación de program ID
- Error al cargar keypair inválido
- Timestamp positivo
- Creación de RpcClient

```bash
cd trust-escrow/escrow-core && cargo test
```

## Uso desde CLI/TUI

```rust
// En Cargo.toml
[dependencies]
escrow-core = { path = "../escrow-core" }

// En código
use escrow_core::*;

let rpc = make_rpc("http://127.0.0.1:8899");
let kp = load_keypair(&kp_path(&None))?;
let sig = op_create_job(&rpc, &kp, 1, "Mi trabajo", "Descripción", 2_000_000_000, "ArbitroPublicKey")?;
```
