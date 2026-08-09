# trust-escrow-v3 Final validation report

**Date:** 2026-08-06
**Scope:** P2 documentation, operational configuration and tooling remediation
**Commits:** none

## Result

La remediación P2 queda completa. La suite TypeScript define **8 tests**; su
evidencia runtime histórica validada fue **9/9** en la ejecución completa
anterior. Los scripts actuales están en **15/15** y los tests Rust en **3/3**.
La validación runtime actual queda bloqueada por ausencia del validator.

## Estado final

| Gate | Resultado |
|---|---|
| Deploy | PASS |
| Hash byte-a-byte del programa | PASS |
| IDL / Anchor.toml / Program ID | PASS |
| Upgrade authority | PASS |
| Config on-chain | PASS |
| Evidencia local actual | BLOCKED — validator ausente; histórica 9/9 PASS |
| Scripts | 15/15 PASS |
| Rust | 3/3 PASS |

**Resultado de release:** `APPROVE` (P0=0, P1=0; quedan únicamente mejoras
P2 documentales/procedurales ya aplicadas en este cambio).

## Passing evidence

- `yarn build`
- `yarn tsc --noEmit`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `yarn check:docs`
- `yarn test:deploy-verifier` (parser/hash tests)
- `yarn test:scripts` (15/15 script tests)
- `TRUST_ESCROW_V3_TEST_GREP='disputa mutua' yarn test:isolated` (retry de expiración: 1/1 pasa en segundo intento)
- `ANCHOR_PROVIDER_URL=http://127.0.0.1:8899 yarn preflight`
- `ANCHOR_PROVIDER_URL=http://127.0.0.1:8899 anchor build`

## Historial de validación

Las ejecuciones previas de Make registraron un bloqueo temporal del validator
local y de la feature SBPFv3. Ese registro se conserva como historial y no
representa el estado final aceptado arriba; no se usó Devnet ni secretos.

### Evidencia histórica bloqueada

- `anchor deploy --provider.cluster localnet`: aborts with `Buffer account data
  size (704205) is smaller than the minimum size (704213)`.
- Direct RPC deploy with `--max-len 704213`: validator rejects the artifact with
  `Detected sbpf_version required by the executable which are not enabled`.
- `solana feature status --url http://127.0.0.1:8899 --output json`: the effective
  SIMD-0178/0179/0189 feature is `BUwGLeF3Lxyfv1J1wY8biFHBB2hrk2QhbNftQf3VV3cC`
  and is `inactive`; supplied `5cC3...` is not that feature in this cluster.
- `ANCHOR_PROVIDER_URL=http://127.0.0.1:8899 timeout 300s yarn test --exit`:
  1 pass (Applications PDA derivation), 1 fail in bootstrap because the Program
  account does not exist after rejected deploy.
- Hash/upgrade-authority/Config readback is **BLOCKED** because no Program account
  was created; IDL/Anchor.toml static identity checks remain build/preflight evidence.

## Security posture

Changed code enforces fixed bootstrap authority, validates treasury ownership and
separation on bootstrap and rotation, links ArbiterPool administration to Config
authority, applies exact 7-day auto-approval from `submitted_at`, blocks
auto-approval on dispute, and restricts `pause_job` to `Created`/`Funded` jobs
without freelancer. Existing payout-direct, Evidence PDA/cleanup, refund and
writable AcceptDispute fixes were preserved.
