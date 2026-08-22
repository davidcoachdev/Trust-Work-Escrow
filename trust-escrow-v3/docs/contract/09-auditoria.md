# 09 · Auditoría de Seguridad (v3)

Revisión de lógica/seguridad del contrato tras la implementación completa.
Lista los hallazgos y su estado.

## Hallazgos y correcciones

### 🔴 Críticos (corregidos)
1. **Destino de pago no validado en `approve_work`**
   `freelancer` (SystemAccount) no estaba atado a `job.freelancer`. El cliente
   firmante podía pasar cualquier wallet y desviar el pago.
   → Añadido `constraint = job.freelancer == Some(freelancer.key())`.

2. **Destino de pago no validado en `approve_milestone`**
   Igual: el pago del milestone podía desviarse.
   → Misma constraint aplicada.

3. **Destino de pago no validado en `finalize_dispute_payouts`**
   El `resolver` podía desviar la parte del freelancer.
   → Misma constraint aplicada (`job.freelancer == Some(freelancer.key())`, lo que
   además exige que `job.freelancer` sea `Some`).

4. **Árbitro podía ser el cliente o el freelancer**
   `assign_arbiter` no verificaba que el árbitro fuera distinto de las partes.
   → Añadido `ArbiterCannotBeParty`: el árbitro (y el asesor en
   `resolve_platform_case`) no puede ser `job.client` ni `job.freelancer`.

### 🟠 Medios (corregidos)
5. **Pagos de milestone durante una disputa**
   `submit/approve/reject_milestone` no chequeaban `job.status`, permitiendo
   pagar desde el PDA `job` mientras estaba en disputa (doble pago con `finalize`).
   → Añadido `require!(job.status == InProgress, ...)` en las 3 instrucciones.

6. **Asesor no podía rescatar disputa con árbitro fallido**
   `resolve_platform_case` exigía `dispute.arbiter.is_none()`, bloqueando el
   rescate cuando el árbitro fue asignado pero no resolvió.
   → Ahora permite también `dispute.status == ArbiterAssigned` (fallback de
   plataforma).

### 🟡 Recomendaciones (pendientes / decisión de diseño)
9. **Fuga de la fee de arbitraje al cliente del job (CRÍTICO, corregido)**
     `finalize_dispute_payouts` cubre el bono no posteado desde el reparto de la
     parte correspondiente y transfiere el faltante explícito desde el PDA
     `job`, respetando la regla de oro (5% "les guste o no").
    → Reescrito: la fee de arbitraje es `5% de lo disputado` (`amount`), el
    `ArbitrationEscrow` paga lo posteado vía `close = arbitration_treasury`, y el faltante
    (`shortfall = 5% − posteado`) se transfiere del PDA `job` a
    `arbitration_treasury`. Conservación: `fee + cliente + freelancer +
    arbitration_treasury = amount + fee` siempre (incluso con 0% para una parte o con
   milestones grandes).
   Además los bonos se postean sobre `amount = job.amount − milestones_amount_total`
   (no sobre `job.amount` completo) en `raise_dispute`/`accept_dispute`, para
   que coincidan con lo disputado.

10. **`approve_work` ya cierra el PDA `job`** (rent recuperada en el flujo feliz).
   El PDA `job` tiene `close = client` en el contexto `ApproveWork` (lib.rs:1418),
   así que al aprobarse el trabajo el PDA se cierra y su renta vuelve al cliente.
   El doc "PDAs siempre se cierran al finalizar ✅" es CORRECTO: los caminos
   terminales (`approve_work`, `cancel_job`, `expire_paused_job`, `finalize_dispute`)
   cierran el job. `reject_work` NO lo cierra a propósito (vuelve a `InProgress`).
   *(Nota: este punto fue mal leído en la primera revisión; se corrige aquí.)*

11. **`deadline` de disputa ahora se usa (corregido).** `raise_dispute` fija
   `dispute.deadline` (7 días). `accept_dispute` y `request_platform_intervention`
   lo respetan (`DisputeDeadlinePassed`): pasada la gracia, las partes ya no pueden
   aceptar ni abrir intervención. El asesor (`resolve_platform_case`) resuelve de
   oficio SOLO si no hubo interaccion del arbitro: (a) arbitro asignado pero fallo
   (`ArbiterAssigned`), o (b) ningun arbitro asignado Y vencio la gracia (Open /
   Active / EvidenceSubmitted). No secuestra una disputa que el arbitro esta
   tratando. Así la disputa nunca queda colgada.

   **Qué pasa con la fee de arbitraje cobrada:** NUNCA se pierde ni se reembolsa.
    La "regla de oro" dice que si se abrio disputa se cobra el 5% "les guste o no".
    En el caso de oficio (vencida la gracia sin interaccion), el asesor de plataforma
    solo autoriza: `finalize_dispute_payouts` envía lo posteado y el `shortfall` a
    `arbitration_treasury`. La parte que no posteo su bono lo paga igual (se le
    descuenta de su reparto). Los fondos siempre se liberan.

12. **`create_milestone` valida `_index` (corregido).** Ahora `index` debe ser
   `== job.milestones_total` (`InvalidMilestoneIndex`): los milestones son
   secuenciales (0,1,2,...) y el PDA `milestone` queda alineado con el contador.
   Antes se podían crear índices arbitrarios/saltados.

13. **La fee de arbitraje va a una cuenta SEPARADA de la empresa (corregido).**
   Antes el resolutor (asesor/árbitro) recibía el 5% en su wallet personal vía
   `close = resolver`. Eso mezclaba fondos personales con de la empresa y rompía
   la contabilidad. Ahora:
   - `Config` tiene `arbitration_treasury` (cuenta aparte de `treasury`).
   - En `finalize_dispute_payouts` el `ArbitrationEscrow` cierra hacia
     `arbitration_treasury` y el `shortfall` se transfiere ahí (no al `resolver`).
   - El `resolver` solo FIRMA (autoriza); no recibe lamports.
   - `initialize_config` recibe `arbitration_treasury`; `update_arbitration_treasury`
     y `withdraw_arbitration` permiten rotar y retirar (paralelos al treasury).
   Así la empresa lleva saldos de arbitraje separados de los de protocolo.
   *Nota de diseño:* en arbitraje mutuo el árbitro externo ya no cobra on-chain;
   la empresa retiene la fee en `arbitration_treasury` y compensa al árbitro
   off-chain (modelo de gestión centralizada). Si se prefiere pagar al árbitro
   on-chain, avisar para ajustar.

7. **Pausa global solo bloquea `create_job`**, no las operaciones en curso.
   *Mitigado* con **pausa por job** (`pause_job`/`unpause_job`/`expire_paused_job`):
   el cliente pausa solo en `Created`/`Funded` (sin freelancer), `deposit_funds` y
   `accept_application` se bloquean mientras está pausado, y tras `MAX_PAUSE_DURATION`
   (30 días) cualquiera puede `expire_paused_job` para reembolsar y cerrar (evita
   fondos atrapados para siempre). La pausa global de emergencia (authority) aún
   solo frena `create_job`; extenderla a flujos activos sigue pendiente de decidir.

8. **`cancel_job` tras milestones aprobados**
   Si se aprobaron milestones (ya pagados) y luego `cancel_job` intenta
   reembolsar `amount + fee`, el PDA no tiene fondos suficientes → la transferencia
   falla (no overpay, pero el cancel se bloquea). Aceptable: cancelar solo antes
   de trabajar. Se puede refinar con un chequeo explícito.

## Validación de identidades (resumen)
- Cliente ≠ Freelancer: ✅ en `accept_application` (`CannotWorkOnOwnJob`).
- Árbitro ≠ Cliente/Freelancer: ✅ en `assign_arbiter` y `resolve_platform_case`
  (`ArbiterCannotBeParty`).
- Asesor ≠ Cliente/Freelancer: ✅ en `resolve_platform_case`.
- Destinos de pago (`freelancer`) siempre atados a `job.freelancer`: ✅ en
  `approve_work`, `approve_milestone`, `finalize_dispute_payouts`.
- Pagos desde PDA usan `new_with_signer`: ✅ (sin firmar fallaría en runtime).
- PDAs siempre se cierran al finalizar: ✅ (sin fondos atrapados).
