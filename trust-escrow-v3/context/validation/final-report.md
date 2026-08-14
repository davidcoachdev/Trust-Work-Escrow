# trust-escrow-v3 Final validation report

**Date:** 2026-08-06
**Scope:** P2 documentation, operational configuration and tooling remediation
**Commits:** none

## Result

Este archivo conserva un **baseline histórico** de la remediación P2; no es una
certificación runtime vigente. La suite TypeScript tenía **8 tests definidos**,
con evidencia histórica 9/9 en una ejecución anterior; los scripts registraban
15/15 y Rust 3/3. Esa evidencia no se reutiliza como prueba actual. La validación
runtime actual está bloqueada por ausencia del validator y el backend v3 de
proyección/sync todavía está planificado.

## Estado final

| Gate | Resultado |
|---|---|
| Deploy | BLOCKED — sin Program account actual |
| Hash byte-a-byte del programa | BLOCKED — no hay artefacto on-chain actual verificable |
| IDL / Anchor.toml / Program ID | PASS estático; runtime BLOCKED |
| Upgrade authority | BLOCKED — sin ProgramData actual |
| Config on-chain | BLOCKED — sin cuenta Program/Config actual |
| Evidencia local actual | BLOCKED — validator ausente; histórica 9/9 PASS |
| Scripts | 15/15 PASS |
| Rust | 3/3 PASS |

**Resultado de release de este baseline:** `BLOCKED` para claims runtime. Los
checks estáticos/documentales pueden quedar `PASS`; no autorizan declarar deploy,
finality, sincronización backend o evidencia on-chain.

## Passing evidence

- `yarn build`
- `yarn tsc --noEmit`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `yarn check:docs`
- `yarn test:deploy-verifier` (parser/hash tests)
- `yarn test:scripts` (15/15 script tests)
- `TRUST_ESCROW_V3_TEST_GREP='disputa mutua' yarn test:isolated` — evidencia histórica, no ejecución vigente
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
