# 🖥️ Desarrollo de Backend — Guía del Programa

**Fuente:** WayLearn GitBook — Recursos de Desarrollo  
**URL:** https://waylearn.gitbook.io/solana-latam-labs-program-waylearn/recursos-de-desarrollo/desarrollo-de-backend  
**Relevante para:** Milestone 3+ (Arquitectura y desarrollo)

---

## Lenguajes y SDKs

### Rust

| Crate | Descripción | Docs |
|-------|-------------|------|
| `solana-sdk` | SDK principal | [docs.rs](https://docs.rs/solana-sdk/) |
| `solana-client` | Interactúa con Solana vía RPC | [docs.rs](https://docs.rs/solana-client/) |
| `solana-commitment-config` | Config de nivel de compromiso | [docs.rs](https://docs.rs/solana-commitment-config/) |
| `solana-program` | Construir programas on-chain | [docs.rs](https://docs.rs/solana-program) |

**Frameworks:**
- **Anchor** — https://www.anchor-lang.com/docs (recomendado para MVP)
- **Steel** (Helius) — Ligero, menos boilerplate → https://www.helius.dev/blog/steel

### TypeScript

| Paquete | Descripción |
|---------|-------------|
| `@solana/kit` | SDK moderno recomendado |
| `@solana/web3.js` | SDK legado |
| `@solana/client` | Runtime sin interfaz |
| `gill` | Librería moderna alternativa |
| `kite` | Librería moderna alternativa |

### Otros lenguajes
- **Python:** `solana-py`, `solders`, `anchorpy`
- **Java:** SDK Sava
- **Go:** `solana-go`, `anchor-go`

---

## Ambientes de Desarrollo

| Ambiente | Cuándo usarlo |
|----------|---------------|
| **Local** | Pruebas rápidas, iterar sin red pública |
| **Devnet** | Flujo completo, demos, tests con usuarios |
| **Mainnet** | Solo cuando el MVP esté probado y auditado |

> ⚠️ **No es necesario desplegar en Mainnet para la incubación.**

**Templates oficiales:**
- https://github.com/WayLearnLatam/Solana-starter-kit
- https://github.com/WayLearnLatam/Solana-Hackathon-Template-Backend
- https://github.com/WayLearnLatam/Solana-Hackathon-Template-FullStack

---

## Deploy

- https://solana.com/docs/programs/deploying
- https://solana.com/docs/references/clusters
- https://solana.com/docs/rpc

---

## Buenas Prácticas

- Validar cuentas y firmantes **dentro del programa**, no en el frontend
- Definir seeds y PDAs correctamente
- Usar errores personalizados claros
- Solo guardar on-chain lo necesario para la lógica principal
- Datos pesados/complementarios → off-chain
- Entender límites de espacio y costos de rent

**Links útiles:**
- https://solana.com/es/learn/staying-safe-on-solana
- https://solana.com/es/docs/core/accounts
- https://solana.com/es/docs/core/pda/pda-accounts
- https://solana.com/es/developers/cookbook/transactions/calculate-cost
- https://solana.com/es/docs/core/fees/compute-budget
