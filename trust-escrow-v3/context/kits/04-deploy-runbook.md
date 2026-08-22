# Cavekit R4: Runbook de deploy seguro

## Goal
Hacer que deploy, bootstrap y verificación de `trust-escrow-v3` sean reproducibles, auditables y resistentes a identidad o endpoint equivocado.

## Constraints
- Calidad: pasos idempotentes o con abort explícito, evidencia persistida y rollback/stop seguro.
- Seguridad: signers esperados, program ID, hash, upgrade authority y endpoints verificados.
- Strict TDD: validar primero scripts/checks que deben fallar ante mismatch.

## Requirements

### R1: Preflight de identidad y entorno
**Description:** El runbook verifica red, RPC, wallet de pago, autoridad, program ID y artefacto antes de mutar estado.
**Acceptance Criteria:**
- [ ] Un endpoint distinto del esperado detiene el runbook antes de deploy o bootstrap.
- [ ] Un program ID, keypair o autoridad distinta de la declarada produce error no cero y evidencia del mismatch.
- [ ] El preflight muestra cluster/endpoint, program ID, commit/artefacto y signer público sin imprimir secretos.
- [ ] El runbook distingue localnet, devnet y mainnet y no usa devnet accidentalmente desde la configuración localnet.
**Dependencies:** R6 R1–R2.

### R2: Deploy y bootstrap verificables
**Description:** Después del deploy, el runbook inicializa y verifica `Config` con los valores aprobados antes de declarar éxito.
**Acceptance Criteria:**
- [ ] El flujo ejecuta deploy y luego `initialize_config` con autoridad, advisor, treasury, arbitration treasury y fee explícitos.
- [ ] Reejecutar sobre una `Config` ya inicializada aborta o verifica sin mutar; nunca intenta takeover.
- [ ] El estado on-chain leído después coincide campo por campo con el manifiesto aprobado.
- [ ] El runbook falla si `Config.authority` no coincide con la identidad esperada o si alguna treasury está cruzada.
**Dependencies:** R1 R1–R3.

### R3: Hash, upgrade authority y evidencia
**Description:** El deploy produce evidencia suficiente para reproducir y auditar exactamente el programa ejecutado.
**Acceptance Criteria:**
- [ ] Se registra el SHA-256 del binario desplegado y del artefacto local comparado.
- [ ] Se consulta y registra el upgrade authority, distinguiendo autoridad multisig, keypair individual o programa inmutable.
- [ ] El program ID del IDL, `Anchor.toml`, binario y cluster coincide.
- [ ] La evidencia incluye timestamp, commit y endpoint sin incluir secretos privados.
**Dependencies:** R1.

### R4: Operación segura y recuperación
**Description:** El runbook documenta fallos, reintentos y verificación posterior sin pasos destructivos implícitos.
**Acceptance Criteria:**
- [ ] Un fallo en preflight, deploy o bootstrap devuelve exit code no cero y no reporta éxito.
- [ ] Los pasos de pause/upgrade/withdraw requieren confirmación de autoridad y destino verificable.
- [ ] El runbook no presenta `pause_job` como mecanismo para detener el timer de `Submitted`: solo es válido en `Created` o `Funded` cuando `job.freelancer == None`.
- [ ] El operador puede reproducir el estado de una ejecución desde sus artefactos y logs sanitizados.
- [ ] El runbook documenta cómo detener Surfpool/localnet y cómo separar evidencia por red.
**Dependencies:** R1–R3.

## Security Gates
- [ ] No se cargan claves privadas desde archivos versionados ni se imprimen.
- [ ] No se despliega a una red no confirmada por el operador.
- [ ] Toda mutación administrativa tiene preflight y verificación postcondición.
- [ ] Hash/program ID/upgrade authority se validan desde fuente on-chain y no solo desde texto local.

## Verification Plan
- `surfpool ls`
- `surfpool run deployment`
- `anchor deploy` solo en la red explícitamente seleccionada.
- Script/runbook de verificación de `Config`, program ID, hash, upgrade authority y endpoint.

## Out of Scope
- Automatizar releases multi-red sin aprobación humana.
- Custodia de claves, creación de multisig o gestión de proveedores externos.
- Migraciones on-chain históricas.

## Cross-References
- **Depende de:** [01-config-bootstrap.md](01-config-bootstrap.md) R1–R3; [06-reproducibility.md](06-reproducibility.md) R1–R4.
- **Relacionado:** [03-deadlines-auto-approval.md](03-deadlines-auto-approval.md) R4; [08-final-validation.md](08-final-validation.md) R3.
