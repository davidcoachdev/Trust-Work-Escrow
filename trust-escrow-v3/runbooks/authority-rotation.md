# V3-SEC-002 — Authority Rotation Runbook (Squads + Timelock 2d)

## Problema
`INITIAL_AUTHORITY` estaba hardcodeado en `src/lib.rs:14` sin rotación posible y sin timelock/multisig.
Riesgo de centralización instant-takeover si la key se compromete.

## Solución
On-chain 2-step con timelock 2 días + multisig Squads como capa externa.

### On-chain (programa `trust-escrow-v3`)
- `Config` ahora almacena:
  ```rust
  pending_authority: Option<Pubkey>
  pending_authority_at: i64
  ```
  + constante `AUTHORITY_TIMELOCK = 2 * 24 * 60 * 60` (172800s).
- Instrucciones nuevas:
  1. `propose_authority(new_authority: Pubkey)`
     - Solo `config.authority` (Signers check `constraint = config.authority == authority.key()`).
     - Guarda `pending_authority = Some(new_authority)` y `pending_authority_at = Clock.unix_timestamp`.
     - Puede sobrescribir propuesta previa (reinicia timelock).
  2. `update_authority()` (approve/execute)
     - Solo `pending_authority` puede firmar (prueba control de la nueva key).
     - Requiere `Clock.unix_timestamp - pending_authority_at >= 172800` o falla `AuthorityTimelockNotExpired`.
     - Atomiza: `config.authority = pending; pending = None; pending_at = 0`.
  3. `cancel_authority_proposal()`
     - Solo `config.authority` cancela propuesta pendiente.
- `initialize_config` inicializa `pending_authority = None, at = 0`.
- `declare_id!` sincronizado vía `Anchor.toml` + `anchor keys sync`:
  ```
  [programs.localnet]
  trust_escrow_v3 = "7a2YhCd7iivXfyySkp1pf5jjijGqpjNqwQCUS912q5Vh"
  ```
  `anchor keys sync` → "All program id declarations are synced."

### Capa Squads (multisig)
1. Crear Squads multisig (p.ej. 2/3) y obtener su vault PDA `SQUADS_VAULT`.
2. Bootstrap inicial sigue usando `INITIAL_AUTHORITY = 3whY...` (hardcode solo para primera inicialización anti-frontrun).
   - Ejecutar `initialize_config` firmado por `INITIAL_AUTHORITY`.
3. Inmediatamente rotar autoridad on-chain hacia Squads:
   ```bash
   # Propose (firmado por INITIAL_AUTHORITY)
   anchor -- call propose_authority --new-authority $SQUADS_VAULT
   # Esperar 2 días (timelock)
   # Approve (firmado por SQUADS_VAULT — via Squads proposal que invoca update_authority)
   # Squads transaction: threshold 2/3 aprueba, la vault PDA firma update_authority
   anchor -- call update_authority --pending-authority $SQUADS_VAULT
   ```
4. A partir de ahí, toda `propose_authority`/`update_authority` futura requiere Squads proposal (threshold 2/3) + timelock + firma de la nueva autoridad.
   - Si la nueva autoridad también es Squads vault, la segunda tx también pasa por Squads.
   - Efecto: 2-step propose/approve + timelock + multisig.

### Errores nuevos
- `NoPendingAuthority (6051)` — no hay propuesta
- `AuthorityTimelockNotExpired (6052)` — <2d
- `InvalidNewAuthority (6053)` — zero address o igual a current

### Verificación
```bash
cd trust-escrow-v3
anchor keys sync        # PASS: All synced
cargo build             # PASS
cargo clippy --all-targets --all-features -- -D warnings  # PASS
cargo test              # 8/8 PASS (incluye test_id)
cargo test --features solana -p trust-escrow-sdk --test job_compact  # 9/9 PASS
# o con validator up: anchor test / cargo test --features solana (9/9 job_compact + 9/9 t26)
yarn build              # PASS
anchor build            # release + idl con pending_authority fields
```
IDL generado incluye `pendingAuthority: Option<Pubkey>`, `pendingAuthorityAt: i64` y las 3 instrucciones nuevas; SDK `types.rs` alineado.

### Upgrade authority
Identidad de upgrade del programa (`solana program show`) debe mantenerse en multisig Squads separada de `Config.authority`. Ver `solana program show 7a2Yh...` authority.

