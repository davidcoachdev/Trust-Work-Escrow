# Cavekit R6: Reproducibilidad y toolchain

## Goal
Permitir que otro operador construya, pruebe y audite el mismo programa en localnet con toolchain alineada y sin secretos en el repo.

## Constraints
- Calidad: versiones explícitas, lockfiles consistentes, clippy limpio y scripts deterministas.
- Seguridad: advisor/signers provisionados por identidad pública o entorno seguro.
- Strict TDD: los checks de reproducibilidad deben fallar ante drift antes de corregirlo.

## Requirements

### R1: Versiones alineadas
**Description:** Anchor CLI, crates Rust, Anchor JS, Solana tooling, Node/Yarn y Rust edition/version compatibles están fijados y documentados.
**Acceptance Criteria:**
- [ ] Un check compara versiones declaradas y efectivas y falla ante incompatibilidad, incluyendo el drift actual JS `0.32.1` vs crates `0.30.0`.
- [ ] `yarn build` completa sin resolver versiones flotantes incompatibles.
- [ ] El lockfile y manifests describen las mismas dependencias efectivas usadas por CI/localnet.
- [ ] Una máquina limpia puede instalar dependencias siguiendo el runbook y obtener el mismo resultado de build.
**Dependencies:** None.

### R2: Advisor y signers sin secretos
**Description:** Los tests y runbooks usan identidades públicas provisionadas de forma segura y no dependen de secretos versionados.
**Acceptance Criteria:**
- [ ] La suite provisiona un advisor conocido para localnet y verifica `Config.advisor` contra su public key.
- [ ] Ningún archivo versionado contiene secret key, mnemonic, seed phrase o wallet personal.
- [ ] Si falta una identidad requerida, el comando falla con instrucción accionable y no usa una identidad aleatoria silenciosa.
- [ ] Los fixtures sanitizados son suficientes para ejecutar tests locales sin acceso a devnet.
**Dependencies:** R1 R1–R2.

### R3: Localnet obligatorio y determinista
**Description:** La validación funcional mínima siempre corre contra localnet reproducible.
**Acceptance Criteria:**
- [ ] `anchor test --provider.cluster localnet` ejecuta la suite completa y no contacta devnet/mainnet.
- [ ] La suite inicializa sus cuentas/authorities y no depende de estado persistente de una ejecución anterior.
- [ ] Repetir la suite dos veces produce los mismos resultados y no falla por PDAs o cuentas residuales.
- [ ] El endpoint efectivo se muestra y se valida antes de mutar.
**Dependencies:** R1–R2.

### R4: Calidad estática y artefactos
**Description:** Build, lint y artefactos generados son reproducibles y limpios.
**Acceptance Criteria:**
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` termina con exit code 0.
- [ ] `yarn build` genera IDL/binario sin cambios no explicados entre ejecuciones equivalentes.
- [ ] No quedan warnings nuevos de seguridad, dead code crítico o errores silenciados.
- [ ] El README/runbook documenta comandos exactos, versiones y precondiciones verificadas.
**Dependencies:** R1–R3.

## Security Gates
- [ ] Secret scanning del repositorio no encuentra credenciales.
- [ ] No se usan endpoints públicos ni wallets personales como dependencia implícita del test.
- [ ] Las versiones no se resuelven desde tags flotantes sin lock.
- [ ] El advisor no tiene privilegios adicionales no requeridos por el flujo.

## Verification Plan
- `yarn install --frozen-lockfile` (o equivalente de lockfile vigente)
- `yarn build`
- `yarn test`
- `anchor test --provider.cluster localnet`
- `cargo clippy --all-targets --all-features -- -D warnings`

## Out of Scope
- Soporte simultáneo de toolchains incompatibles.
- CI cloud completa o publicación de paquetes.
- Gestión de secretos de producción más allá de documentar la interfaz segura.

## Cross-References
- **Depende de:** [04-deploy-runbook.md](04-deploy-runbook.md) R1/R4.
- **Verifica:** [01-config-bootstrap.md](01-config-bootstrap.md) R1; [08-final-validation.md](08-final-validation.md) R1.
