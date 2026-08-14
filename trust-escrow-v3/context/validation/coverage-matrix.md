# Validation coverage matrix

| Area | Evidence | Status |
|---|---|---|
| Config bootstrap/rotation | `initialize_config`, rotation destination helper and `tests/escrow.ts` rotation cases | PARTIAL (persistent Config blocks clean negative run) |
| Arbiter governance | Config authority checks in create/add/remove/assign | PASS by static/build gates; integration pending |
| Submitted/deadline | `auto_approve_work`, `submitted_at`, 604800 seconds, dispute option | PARTIAL (boundary execution pending) |
| Pause | status plus `freelancer == None` guard | PASS by code/build; negative integration pending |
| Evidence/payout conservation | Evidence PDA x10, cleanup parcial/final, arbitration treasury y conservación descritos en contrato/tests históricos | BLOCKED/PARTIAL (sin evidencia runtime vigente; no reutilizar Surfpool/histórico como PASS) |
| Reproducibility/deploy | Rust/Anchor alignment, localnet preflight, secret-free verifier, IDL/Anchor.toml/program ID, ProgramData byte hash and Config checks | BLOCKED (SBPFv3 inactive; Program account absent) |
| Docs/IDL | generated IDL plus semantic `yarn check:docs` over all Markdown docs; no inline Evidence collection or resolver fee wording; backend boundary docs planned | PASS estático; backend v3 NOT IMPLEMENTED |
| Applications runtime | Job compacto, PDA individual, duplicate-before-limit, 50/51 path | BLOCKED runtime (Program account absent; only PDA derivation test executed) |

No public network or credential was used.
