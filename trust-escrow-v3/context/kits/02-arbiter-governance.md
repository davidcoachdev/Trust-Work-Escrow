# Cavekit R2: Gobernanza de ArbiterPool

## Goal
Garantizar que solo la autoridad de `Config` gobierne el pool global de árbitros y que las asignaciones mantengan neutralidad.

## Constraints
- Calidad: pool único, límites y operaciones deterministas.
- Seguridad: autoridad ligada a `Config.authority`, árbitros no partidarios y no duplicados.
- Strict TDD: cada criterio se prueba primero en RED.

## Requirements

### R1: Creación gobernada por Config
**Description:** `ArbiterPool` solo puede crearse o quedar gobernado por la autoridad efectiva de `Config`.
**Acceptance Criteria:**
- [ ] Crear el pool con `Config.authority` guarda la misma autoridad y la PDA esperada.
- [ ] Un signer distinto, aunque sea el primer caller, no puede crear ni tomar el pool global.
- [ ] Crear el pool por segunda vez falla sin reemplazar autoridad ni lista.
- [ ] El test verifica que `ArbiterPool.authority == Config.authority` después de bootstrap y después de cualquier operación válida.
**Dependencies:** R1 R1–R2.

### R2: Administración de miembros
**Description:** Agregar y quitar árbitros requiere autorización, respeta el límite vigente y conserva invariantes de unicidad.
**Acceptance Criteria:**
- [ ] Add/remove firmado por autoridad válida modifica únicamente el miembro esperado.
- [ ] Add/remove firmado por cualquier otro signer falla y no cambia la lista.
- [ ] Agregar un árbitro duplicado o superar `MAX_ARBITERS` falla de forma determinista.
- [ ] Quitar un árbitro ausente falla sin compactar o corromper otros miembros.
**Dependencies:** R1.

### R3: Asignación neutral
**Description:** Una asignación desde el pool solo puede apuntar a un árbitro válido y distinto de cliente y freelancer.
**Acceptance Criteria:**
- [ ] Un árbitro incluido puede asignarse solo a una disputa/job compatible y deja el estado esperado.
- [ ] Un árbitro que no pertenece al pool es rechazado.
- [ ] Cliente, freelancer, advisor o cuenta no válida como árbitro son rechazados según la regla de neutralidad vigente.
- [ ] Repetir asignación en una disputa que ya tiene árbitro falla sin cambiar el árbitro original.
**Dependencies:** R1, R2, R3 de `03-deadlines-auto-approval.md`.

### R4: Integridad de fee de arbitraje
**Description:** La gobernanza del pool no permite desviar la fee de arbitraje ni mezclarla con fondos personales del resolver.
**Acceptance Criteria:**
- [ ] Un payout resuelto deposita la fee y cualquier shortfall en `arbitration_treasury` configurado.
- [ ] Cambiar el signer resolver no cambia el destino económico.
- [ ] La suma de fee, payout cliente, payout freelancer y shortfall conserva el monto disputado más fee.
- [ ] La cuenta personal del resolver no recibe lamports por el flujo de payout.
**Dependencies:** R1 R3, R3 R4.

## Security Gates
- [ ] Toda instrucción de pool comprueba signer y vínculo con `Config.authority`.
- [ ] No hay PDA global inicializable por permissionless caller.
- [ ] No hay árbitro igual a una parte ni destino de fee controlado por input libre.
- [ ] Límite de miembros y duplicados están cubiertos por tests negativos.

## Verification Plan
- `yarn build`
- Tests de autoridad, doble creación, add/remove, duplicados, límite y neutralidad.
- `anchor test --provider.cluster localnet`
- Inspección del estado de `ArbiterPool` y balances de `arbitration_treasury`.

## Out of Scope
- Selección aleatoria, staking, reputación o compensación off-chain de árbitros.
- Cambiar el modelo de advisor de plataforma.
- Crear múltiples pools por red/job.

## Cross-References
- **Depende de:** [01-config-bootstrap.md](01-config-bootstrap.md) R1–R2.
- **Relacionado:** [03-deadlines-auto-approval.md](03-deadlines-auto-approval.md) R3; [05-security-tests.md](05-security-tests.md) R2/R4.
