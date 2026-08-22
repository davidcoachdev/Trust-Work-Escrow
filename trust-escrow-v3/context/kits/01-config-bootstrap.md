# Cavekit R1: Bootstrap seguro de Config

## Goal
Impedir que un caller no autorizado tome control de `Config` o fije destinos económicos maliciosos durante la inicialización.

## Constraints
- Calidad: inicialización determinista, atómica y observable.
- Seguridad: autoridad conocida/multisig, una sola inicialización, validación de cuentas y protección contra frontrun.
- Strict TDD: cada criterio debe tener prueba RED antes de implementación.

## Requirements

### R1: Bootstrap con autoridad conocida
**Description:** La autoridad inicial de `Config` debe estar determinada antes de enviar la transacción, ser verificable contra una identidad esperada y poder representar una multisig.
**Acceptance Criteria:**
- [ ] Una inicialización firmada por la autoridad esperada crea `Config` con `authority` igual a esa identidad; el test falla si difieren.
- [ ] Una inicialización firmada por un signer no permitido es rechazada y `Config` no queda creada ni parcialmente configurada.
- [ ] El artefacto de configuración/runbook registra la identidad esperada sin guardar claves privadas ni secretos.
- [ ] El caso multisig usa la identidad de la multisig como autoridad efectiva y no un signer auxiliar individual.
**Dependencies:** R6 (provisión segura de signers), R4 (runbook de inicialización).

### R2: Inicialización única y anti-frontrun
**Description:** La cuenta `Config` se puede inicializar una sola vez y la transacción no permite que el primer caller público elija autoridad, advisor o treasuries arbitrarios.
**Acceptance Criteria:**
- [ ] Dos intentos concurrentes de inicialización producen como máximo una `Config` válida; el segundo falla de forma determinista.
- [ ] El primer intento no autorizado no puede reservar o dejar una PDA utilizable por el atacante.
- [ ] La transacción válida fija autoridad, advisor, treasury, `arbitration_treasury`, fee y estado inicial en una única operación verificable.
- [ ] Reintentar con la misma identidad después de una inicialización válida falla sin modificar ningún campo existente.
**Dependencies:** R1.

### R3: Validación de advisor, treasuries y fee
**Description:** Los parámetros económicos y de gobernanza de `Config` cumplen invariantes antes de persistirse.
**Acceptance Criteria:**
- [ ] Un fee fuera de `[0, BASIS_POINTS]` es rechazado y no cambia `Config`.
- [ ] `advisor`, `treasury` y `arbitration_treasury` no pueden ser cuentas nulas, no válidas o no compatibles con el rol declarado.
- [ ] `treasury` y `arbitration_treasury` son distintos y los payouts/withdrawals solo aceptan el destino configurado para cada fee.
- [ ] El estado inicial es no pausado y todos los campos leídos por el IDL coinciden con los valores aprobados.
**Dependencies:** R1, R2.

### R4: Rotación y autorización posterior al bootstrap
**Description:** Las mutaciones administrativas posteriores conservan el vínculo con la autoridad de `Config` y no permiten sustitución por cuentas arbitrarias.
**Acceptance Criteria:**
- [ ] `pause`, `unpause`, rotación de treasury y rotación de arbitration treasury rechazan signer que no sea `Config.authority`.
- [ ] Un signer autorizado puede rotar cada destino a una cuenta válida y el nuevo valor se observa inmediatamente en la cuenta.
- [ ] Los retiros rechazan monto cero, fondos insuficientes y treasury que no coincida con el campo configurado.
- [ ] Ninguna operación administrativa modifica `authority` sin el flujo de gobernanza explícitamente aprobado.
**Dependencies:** R1, R3, R2.

## Security Gates
- [ ] No hay autoridad inicial derivada del primer caller sin allowlist/multisig verificable.
- [ ] No hay secretos, seed phrases ni keypair JSON en git, logs o fixtures.
- [ ] La cuenta PDA y todos los destinos están validados antes de cualquier transferencia.
- [ ] Los errores de autorización y validación son explícitos; no hay catch silencioso.

## Verification Plan
- `yarn build`
- Tests Anchor/TypeScript de inicialización autorizada, no autorizada, doble/concurrente y parámetros inválidos.
- `anchor test --provider.cluster localnet`
- Revisión de cuentas/IDL generados y de la transacción de bootstrap del runbook.

## Out of Scope
- Cambiar la política de multisig elegida por la organización.
- Agregar un sistema general de roles más allá de autoridad, advisor y treasuries actuales.
- Migrar una `Config` ya existente sin una especificación separada.

## Cross-References
- **Depende de:** [06-reproducibility.md](06-reproducibility.md) R2 y R4.
- **Relacionado:** [02-arbiter-governance.md](02-arbiter-governance.md) R1; [04-deploy-runbook.md](04-deploy-runbook.md) R2.
- **Verificado por:** [05-security-tests.md](05-security-tests.md) R1 y [08-final-validation.md](08-final-validation.md) R1.
