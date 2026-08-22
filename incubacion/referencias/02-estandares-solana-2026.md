# Estándares Solana — Junio 2026

**Fecha de consulta:** 24 de Junio, 2026

---

## 1. Anchor Framework

| Aspecto | Versión Actual |
|---|---|
| **Última versión** | **v1.0.2** (2 Mayo 2026) |
| **Repo** | [otter-sec/anchor](https://github.com/otter-sec/anchor) (migró de coral-xyz) |
| **Solana CLI requerida** | 3.x (recomendado 3.1.10+) |
| **Rust requerido** | 1.79–1.85+ (stable) |
| **Platform Tools** | v1.52 |
| **GLIBC requerido** | ≥ 2.39 |
| **Test runner default** | Surfpool |

### Cambios principales de Anchor 0.30 → 1.0

| Concepto | Antes (0.30) | Después (1.0) |
|---|---|---|
| `CpiContext::new` | `(AccountInfo, accounts)` | `(Pubkey, accounts)` |
| `realloc` | `realloc(new_len, false)` | `resize(new_len)` |
| `#[interface]` attribute | `#[interface(...)]` | `#[instruction(discriminator = ...)]` |
| `declare_program! utils` | `::utils::*` | `::parsers::*` |
| `Context<'a, 'b, 'c, 'info>` | 4 lifetimes | 1 lifetime: `Context<'info>` |
| `try_to_vec()` | `value.try_to_vec()` | `borsh::to_vec(&value)` |
| `AccountInfo` raw | Usado directamente | `UncheckedAccount<'info>` + `/// CHECK:` |
| `solana-program` import | `anchor_lang::solana_program::*` | `solana_program::*` directo |
| TS package | `@coral-xyz/anchor` | `@anchor-lang/core` |
| IDL management | On-program (legacy) | Program Metadata (programa separado) |
| Múltiples `#[error_code]` | Permitido | Compile error (merge en uno) |

---

## 2. Solana Rust SDK — Modular

El monolito `solana-sdk` / `solana-program` se dividió en **~100 crates granulares**.

| Crate | Versión Actual | Propósito |
|---|---|---|
| `solana-program` | **4.0.0** | On-chain programs (contiene sub-crates) |
| `solana-sdk` | **4.0.1** | Off-chain / client-side |
| `solana-pubkey` | 4.1.0 | `Pubkey` type |
| `solana-account-info` | ^3.x | `AccountInfo` type |
| `solana-cpi` | ^3.x | Cross-program invocation |
| `solana-signer` | ^3.x | Signer trait (reemplaza `solana-sdk` para signing) |
| `solana-instruction` | ^3.x | `Instruction` type |
| `solana-msg` | ^3.x | `msg!()` macro |
| `solana-entrypoint` | ^3.x | Program entrypoint |
| `solana-borsh` | ^3.x | Borsh serialization (v0.10 y v1) |
| `solana-keypair` | ^3.x | Keypair generation |
| `solana-account` | 4.3.1 | Account type |
| `solana-transaction` | 4.2.x | Transaction types |
| `solana-system-interface` | ^3.x | System program IDs/instructions |
| `solana-vote-interface` | 6.0.x | Vote program interface |

**Patrón recomendado:** Importar solo lo que necesitás en vez de todo `solana-sdk`:
```rust
// Antes: todo el monolito
use solana_sdk::{pubkey::Pubkey, signature::Keypair, signer::Signer};

// Ahora: granular
use solana_pubkey::Pubkey;
use solana_keypair::Keypair;
use solana_signer::Signer;
```

---

## 3. `@solana/kit` (JS/TS SDK) — v6.10.0

| Package | Versión | Ámbito |
|---|---|---|
| **`@solana/kit`** | **6.10.0** (16 Jun 2026) | Core SDK, tree-shakeable |
| `@solana/client` | 1.7.0 | Orquestación cliente (RPC + wallets + txs) |
| `@solana/react-hooks` | 1.4.1 | Hooks React (useWallet, useBalance, etc.) |
| `@solana/web3-compat` | — | Capa de compatibilidad con web3.js legacy |

### Stack recomendado para frontend

```bash
npm install @solana/kit @solana/client @solana/react-hooks
```

```tsx
import { autoDiscover, createClient } from "@solana/client";
import { SolanaProvider, useBalance, useWallet } from "@solana/react-hooks";

const client = createClient({
  endpoint: "https://api.devnet.solana.com",
  walletConnectors: autoDiscover(),
});
```

**Novedades de v6.x:**
- Fixed-point numbers: `@solana/fixed-points` para SOL/Lamports con precisión
- Reactive stores: `reactiveStore()` para suscripciones RPC
- Compute Unit estimation: `estimateAndSetComputeUnitLimitFactory()`
- TypeScript ≥5.4+ requerido
- `@solana/rpc-graphql` para queries GraphQL sobre RPC

---

## 4. Testing

| Herramienta | Versión | Descripción |
|---|---|---|
| **surfpool** | Default en Anchor 1.0 | Test runner local (reemplaza solana-test-validator) |
| **litesvm** | 0.9.1 | Unit tests rápidos in-process |
| **anchor-litesvm** | 0.3 | Wrapper Anchor para litesvm |
| **Mollusk** | — | Testing lightweight (alternativa) |

### Surfpool config
```toml
# Anchor.toml
[surfpool]
startup_wait = 5000
log_level = "info"
block_production_mode = "clock"
```

### LiteSVM compatibility

| litesvm | Solana crates | Markers clave |
|---|---|---|
| 0.8.2 | ~3.0 | solana-hash ~3.0, vote-interface 4.0 |
| 0.9.1 | ~3.1–3.3 | solana-hash 4.0, vote-interface 5.0 |
| >0.10.0 | 3.3+ | Latest |

---

## 5. Otras herramientas relevantes

| Herramienta | Descripción |
|---|---|
| **Pinocchio** | Framework zero-dependency para programs Solana (alternativa a Anchor) |
| **Codama** | Codegen de clientes desde IDL (reemplaza kinobi) |
| **Program Metadata** | IDL en cuentas separadas del programa (`@solana-program/program-metadata`) |
| **Agave** | Validador Solana (ahora requiere build desde source para v3.x) |

---

## 6. Tabla de compatibilidad rápida

```
Anchor v1.0.2
├── Solana CLI 3.x
├── solana-* crates ^3 (granular)
├── Platform Tools v1.52
├── Rust 1.79–1.85+
├── Node.js ≥20 LTS
├── Surfpool (tests)
├── LiteSVM 0.8.2 / 0.9.1
└── GLIBC ≥2.39

@solana/kit v6.10.0
├── @solana/client v1.7.0
├── @solana/react-hooks v1.4.1
├── TypeScript ≥5.4
└── Node.js ≥20.18.0
```

---

## 7. Links útiles

- Anchor releases: https://github.com/otter-sec/anchor/releases
- Anchor docs: https://www.anchor-lang.com/
- Solana docs: https://solana.com/docs
- `@solana/kit` releases: https://github.com/anza-xyz/kit/releases
- Migración Anchor 0.32 → 1.0: [migrating-v0.32-to-v1.md](https://github.com/solana-foundation/solana-dev-skill/blob/main/skill/references/anchor/migrating-v0.32-to-v1.md)
- Compat matrix: [compatibility-matrix.md](https://github.com/solana-foundation/solana-dev-skill/blob/main/skill/references/compatibility-matrix.md)
- Frontend docs: https://solana.com/docs/frontend
- Anza SDK crates: https://github.com/anza-xyz/solana-sdk/releases
