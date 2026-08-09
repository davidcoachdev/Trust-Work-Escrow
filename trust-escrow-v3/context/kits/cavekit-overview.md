# Cavekit Overview — Remediación de auditoría trust-escrow-v3

## Goal
Definir el comportamiento verificable necesario para remediar los hallazgos de auditoría de `trust-escrow-v3` sin implementar código en esta fase.

## Decisiones confirmadas
- `Submitted` significa que el freelancer entregó el trabajo mediante `submit_work`; no existe un estado separado `Received`.
- La auto-aprobación ocurre on-chain exactamente después de 7 días desde `submitted_at`.
- Durante esa ventana el cliente puede aprobar, rechazar o abrir una disputa.
- Una disputa abierta bloquea la auto-aprobación.
- Al vencer la ventana sin acción válida, el programa paga `amount` al freelancer, `fee_amount` a `treasury` y cierra el `Job`.
- `pause_job` solo se permite en `Created` o `Funded` cuando `job.freelancer == None`; falla con freelancer asignado o en `Submitted` y no detiene el timer de auto-aprobación.
- La fee de arbitraje permanece separada en `arbitration_treasury`; el resolver firma pero no recibe fondos personales.
- Cada Job admite como máximo 50 postulaciones; cada postulación vive en una PDA individual determinista, validada contra Job, índice y applicant, y su rent/cleanup sigue el ciclo de vida de la postulación.

## Domains

| Kit | Archivo | Alcance |
|---|---|---|
| R1 | [01-config-bootstrap.md](01-config-bootstrap.md) | Bootstrap seguro de `Config`, autoridad, advisor, treasuries y fees |
| R2 | [02-arbiter-governance.md](02-arbiter-governance.md) | Gobernanza de `ArbiterPool` ligada a `Config.authority` |
| R3 | [03-deadlines-auto-approval.md](03-deadlines-auto-approval.md) | Job, postulaciones en PDAs individuales, deadline de 7 días, reloj y excepciones de estado |
| R4 | [04-deploy-runbook.md](04-deploy-runbook.md) | Deploy reproducible y verificación de identidad/autoridades |
| R5 | [05-security-tests.md](05-security-tests.md) | Tests negativos, postulaciones individuales, estados, replay, cleanup, evidencias y payouts |
| R6 | [06-reproducibility.md](06-reproducibility.md) | Toolchain alineada, localnet obligatorio, clippy y advisor sin secretos |
| R7 | [07-docs-idl-sync.md](07-docs-idl-sync.md) | Sincronización de docs, IDL, constantes, estados, cuentas y Application PDAs |
| R8 | [08-final-validation.md](08-final-validation.md) | Validation gate final y security gates de release |

## Dependency graph
1. R1 → R2: el pool debe depender de la autoridad de `Config` ya bootstrappeada.
2. R1 → R4/R6: las identidades y endpoints usados por deploy/reproducibilidad salen del bootstrap seguro.
3. R3 → R5/R7: la máquina de estados, el reloj y la semántica de payouts deben ser testeados y documentados.
4. R3 → R5/R7: `create_job`, `apply_to_job` y `accept_application` deben probarse y documentarse con PDAs individuales, índice, applicant, duplicados, límites de texto, máximo 50 y cleanup/rent.
5. R2/R3 → R5: los tests negativos y de replay cubren autorización, disputas, postulaciones y auto-aprobación.
6. R1–R7 → R8: la validación final no puede declarar release si algún dominio no pasa sus gates.

## Cross-cutting constraints
- **Strict TDD:** para cada criterio se escribe primero una prueba fallida (RED), luego el cambio mínimo (GREEN) y finalmente el refactor; no se acepta cobertura solo nominal.
- **Security first:** no secretos en el repositorio, no autoridades permissionless, no destinos de payout no ligados, no input sin límites, no SQL/shell crudo aplicable, no `unwrap`/catch silencioso en rutas críticas.
- **ROI/YAGNI:** no se agregan features de producto, UI, nuevos roles, nuevos tipos de disputa ni cambios de economía no requeridos por estos kits.

## Verification baseline
Los comandos de verificación se derivan de los artefactos actuales y deben ejecutarse desde `trust-escrow-v3/`:

```bash
yarn build
yarn test
anchor test --provider.cluster localnet
cargo clippy --all-targets --all-features -- -D warnings
surfpool ls
surfpool run deployment
```

El runbook debe documentar cualquier precondición local necesaria, pero no puede sustituir el test localnet obligatorio.

## Known audit baseline and specification gaps
- El documento existente `docs/contract/09-auditoria.md` registra correcciones previas, pero no constituye evidencia de que todos los criterios actuales estén implementados.
- El bootstrap de `Config` todavía requiere especificar/verificar autoridad conocida o multisig, inicialización única y protección anti-frontrun.
- La documentación describe auto-aprobación, pero el contrato/tests deben probar `submit_work → Submitted` sin `Received`, la ventana desde `submitted_at`, acciones del cliente, disputa bloqueante, la autorización acotada de `pause_job`, cancelación y clock boundary.
- El modelo anterior de `Applications` inline puede superar el límite de asignación de cuentas; los kits deben exigir que `create_job`, `apply_to_job` y `accept_application` usen una PDA individual determinista por postulación, con máximo 50, validación de Job/índice/applicant/duplicados/texto/permisos y cleanup/rent por ciclo de vida.
- Hay desalineación conocida entre Anchor JS `0.32.1` y crates Anchor `0.30.0`; R6 exige resolverla y fijarla.
- El runbook actual despliega, pero no demuestra inicialización/verificación completa de `Config`, hash, program ID, upgrade authority ni endpoints.
- El advisor de tests y los signers deben estar provisionados sin secretos persistidos.

## Out of scope global
- Implementación o refactor de código en esta fase Sketch.
- Diseño de UI, indexer, notificaciones, dashboard, tokenomics nueva o migración de fondos histórica.
- Cambio de la regla económica confirmada, salvo corregir su aplicación observable.
- Auditoría formal externa, certificación legal o garantía de seguridad fuera de los gates definidos.

## Phase gate: Sketch → Map
- [ ] Cada kit tiene requirements secuenciales, criterios observables y automatizables.
- [ ] Cada requirement declara dependencias, out-of-scope y cross-references.
- [ ] Cada criterio está vinculado a un gate de validación.
- [ ] Los gaps abiertos están explícitos y no se rellenan con supuestos de implementación.
