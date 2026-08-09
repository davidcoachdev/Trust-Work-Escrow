# Cavekit R7: Documentación e IDL sincronizados

## Goal
Hacer que código, IDL, documentación contractual y runbooks describan exactamente los estados, PDAs, límites, fees y autoridades actuales.

## Constraints
- Calidad: una fuente verificable por dato, enlaces internos válidos y ejemplos ejecutables.
- Seguridad: la documentación no puede inducir a desplegar con autoridad, endpoint o fee equivocados.
- Strict TDD: checks de drift y ejemplos fallan antes de actualizar docs.

## Requirements

### R1: Inventario contractual vigente
**Description:** La documentación enumera el surface actual sin contradicciones con código/IDL.
**Acceptance Criteria:**
- [ ] Se documentan `Config`, `Job`, `ArbiterPool`, `Dispute`, SupportTicket, Milestone y Evidence con ownership y asociación.
- [ ] Se documentan estados válidos y transiciones, incluyendo `Submitted`, terminales y estados de disputa actuales.
- [ ] La transición documentada indica que `submit_work` produce `Submitted` y que no existe un estado separado `Received`.
- [ ] Se documentan fees de protocolo, fee de arbitraje, `treasury` y `arbitration_treasury` como destinos separados.
- [ ] Se documentan `MAX_APPLICATIONS = 50`, límites vigentes de evidence (10 PDAs de hasta 2.048 bytes) y cualquier otra constante usada por validación.
**Dependencies:** R2 R1–R4, R3 R1–R4.

### R2: Escenarios y auto-aprobación
**Description:** Los escenarios describen la decisión confirmada de 7 días y sus excepciones sin contradicción.
**Acceptance Criteria:**
- [ ] El escenario indica que el clock parte de `submitted_at`, no de `deadline` de creación del job.
- [ ] Se muestran aprobar, rechazar y disputar durante la ventana de 7 días.
- [ ] Se muestra que una disputa abierta bloquea auto-aprobación, que `pause_job` solo aplica en `Created`/`Funded` sin freelancer y que no puede detener el timer de `Submitted`.
- [ ] El payout documentado coincide con `amount` a freelancer, `fee_amount` a `treasury` y cierre del Job.
**Dependencies:** R3 R1–R5.

### R3: IDL y referencias reproducibles
**Description:** El IDL generado, ejemplos y docs de cuentas reflejan nombres, argumentos y restricciones efectivas.
**Acceptance Criteria:**
- [ ] El IDL se regenera y el diff se explica; no quedan instrucciones/cuentas documentadas que no existan.
- [ ] Las referencias de Evidence PDAs, seeds y cleanup coinciden con el IDL/código vigente.
- [ ] Los ejemplos identifican correctamente `advisor`, resolver, authorities y treasuries.
- [ ] Los enlaces desde README, contrato, escenarios y runbooks no apuntan a archivos inexistentes.
**Dependencies:** R6 R1/R4.

### R4: Docs como gate de release
**Description:** Una release no puede declararse si docs, IDL y runbook están desincronizados.
**Acceptance Criteria:**
- [ ] Un check automatizado detecta drift de constantes, instrucciones, estados o program ID entre fuentes generadas y docs marcadas.
- [ ] Cada hallazgo de `09-auditoria.md` tiene estado: remediado con evidencia, aceptado explícitamente o gap abierto.
- [ ] La documentación identifica los gaps que no deben interpretarse como implementados.
- [ ] El checklist final enlaza cada requirement de estos kits con al menos un archivo/test/evidencia.
**Dependencies:** R1–R3, R5 R1–R5.

### R5: Modelo documental de postulaciones individuales
**Description:** El IDL, la documentación contractual, los escenarios y los tests describen el modelo vigente de hasta 50 postulaciones por Job, una PDA determinista por postulación y su ciclo de vida.
**Acceptance Criteria:**
- [ ] `create_job`, `apply_to_job` y `accept_application` documentan sus cuentas, argumentos, permisos y postcondiciones sin referenciar una colección `Applications` inline.
- [ ] La documentación identifica la PDA individual de cada postulación, su relación con Job, índice y applicant, y la regla reproducible de derivación/ownership/bump sin discrepancias con el IDL.
- [ ] Se documentan el rango de índices, el máximo exacto de 50, la unicidad por applicant y Job, y los errores esperados para duplicados, Job/índice/applicant inválidos y permisos incorrectos.
- [ ] Se documentan los límites de texto/tamaño para title, description y proposal, incluyendo el comportamiento observable ante vacío, exceso y validación por bytes.
- [ ] Los escenarios y ejemplos cubren crear Job, aplicar, aceptar una aplicación y rechazar intentos cruzados o duplicados; los tests enlazados prueban cada criterio antes de declarar sincronización.
- [ ] El ciclo de vida documenta cuándo una Application PDA se cierra o se retiene, quién recibe su rent y cómo se verifica que no quedan cuentas huérfanas ni rent contabilizada como payout.
- [ ] Un check de drift falla si el IDL, las seeds, el máximo 50, los límites de texto, las instrucciones o los docs describen el modelo inline anterior.
**Dependencies:** R3 R6, R5 R6, R6 R1/R3.

## Security Gates
- [ ] No se documentan secretos ni endpoints de producción como valores por defecto.
- [ ] Fees y destinos están expresados sin ambigüedad de unidades/basis points.
- [ ] La docs no promete una garantía que el IDL/código no verifica.
- [ ] Toda instrucción administrativa documenta signer y constraints.
- [ ] La documentación no permite interpretar el índice o applicant como input confiable: ambos quedan sujetos a constraints on-chain y a la PDA determinista.

## Verification Plan
- `yarn build` para regenerar IDL/binario.
- `yarn test`
- Link checker y script de drift de docs/IDL/constantes.
- Revisión de `docs/contract/09-auditoria.md`, escenarios y `runbooks/README.md`.

## Out of Scope
- Redacción legal de ToS o políticas comerciales.
- Traducción multilingüe.
- Documentar features no existentes para anticipar roadmap.
- Documentar ranking, matching automático, más de 50 postulaciones o una cuenta agregada inline de aplicaciones.

## Cross-References
- **Depende de:** [03-deadlines-auto-approval.md](03-deadlines-auto-approval.md) R1–R4; [06-reproducibility.md](06-reproducibility.md) R1/R4.
- **Relacionado:** [04-deploy-runbook.md](04-deploy-runbook.md) R2/R3.
- **Relacionado:** [03-deadlines-auto-approval.md](03-deadlines-auto-approval.md) R6; [05-security-tests.md](05-security-tests.md) R6.
