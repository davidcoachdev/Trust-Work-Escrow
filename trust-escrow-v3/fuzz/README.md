# Fuzz — trust-escrow-v3 (V3-TEST-015)

Tres harnesses `cargo fuzz` (libFuzzer) para las superficies críticas de V3-TEST-015:

- `remaining_accounts_malformed` — deserialización `RemainingAccounts` con bytes arbitrarios, invariantes de paginación 10 y `is_writable`.
- `evidence_cursor_overflow` — `evidence_count` / `evidence_cleanup_cursor` con `checked_sub`/`checked_add` y límite `MAX_EVIDENCE_COUNT 10`.
- `max_pause_and_fees` — `MAX_PAUSE_DURATION 30d`, `compute_fee`/`compute_shortfall` sin overflow.

## Requisitos

```bash
cargo install cargo-fuzz
rustup toolchain install nightly
```

## Ejecución (libFuzzer, nightly)

```bash
# desde repo root
cargo fuzz run remaining_accounts_malformed --manifest-path trust-escrow-v3/fuzz/Cargo.toml -- -max_total_time=60
cargo fuzz run evidence_cursor_overflow --manifest-path trust-escrow-v3/fuzz/Cargo.toml -- -max_total_time=60
cargo fuzz run max_pause_and_fees --manifest-path trust-escrow-v3/fuzz/Cargo.toml -- -max_total_time=60
```

## Cobertura complementaria (proptest, stable)

Los mismos invariantes tienen espejo `proptest` en `programs/trust-escrow-v3/src/lib.rs::proptest_fuzz` ejecutable en stable:

```bash
cargo test --manifest-path trust-escrow-v3/programs/trust-escrow-v3/Cargo.toml --lib proptest_fuzz
cargo test --manifest-path trust-escrow-v3/programs/trust-escrow-v3/Cargo.toml --lib
```
