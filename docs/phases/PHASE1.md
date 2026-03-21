# Phase 1: Foundation - Trust Work Escrow v2

## Descripción

Setup inicial del proyecto - estructura base, dependencias y tipos.

## Fecha

2026-03-21

## Estado

✅ Completado

---

## Archivos Creados

```
trust-escrow-v2/
├── Anchor.toml
└── programs/trust-escrow-v2/
    ├── Cargo.toml
    └── src/
        ├── lib.rs
        ├── error.rs
        ├── state/
        │   ├── mod.rs
        │   ├── config.rs
        │   ├── user.rs
        │   ├── job.rs
        │   └── arbiter_pool.rs
        └── instructions/
            ├── mod.rs
            ├── user.rs
            ├── job.rs
            ├── arbiter.rs
            └── config.rs
```

---

## Detalles

### Cargo.toml

Dependencias principales:
- anchor-lang = "0.32"
- anchor-spl = "0.32"
- solana-program = "1.18"
- borsh = "0.10"
- thiserror = "1.0"

### State

- **Config**: Configuración global (admin, treasury, multisig, fee_percent, paused)
- **User**: Cuenta de usuario con multi-wallet (wallet_principal, wallets_asociadas, active_wallet, username, bio)
- **Job**: Cuenta de trabajo/escrow (client, freelancer, arbiter, amount, status, title, etc.)
- **ArbiterPool**: Pool de árbitros registrados

### Constants

- MAX_WALLETS = 10
- MAX_ARBITERS = 50
- MAX_MULTISIG_OWNERS = 5
- MAX_USERNAME_LENGTH = 32
- MAX_BIO_LENGTH = 500
- MAX_TITLE_LENGTH = 100
- MAX_DESCRIPTION_LENGTH = 500
- MAX_DISPUTE_REASON_LENGTH = 200
- MIN_JOB_AMOUNT = 100_000 lamports

---

## Siguiente

Phase 2: Core Implementation - Instrucciones del programa