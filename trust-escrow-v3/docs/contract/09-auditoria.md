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
7. **Pausa global solo bloquea `create_job`**, no las operaciones en curso.
   *Mitigado* con **pausa por job** (`pause_job`/`unpause_job`/`expire_paused_job`):
   el cliente pausa solo en `Created`/`Funded` (sin freelancer), `deposit_funds` y
   `accept_job` se bloquean mientras está pausado, y tras `MAX_PAUSE_DURATION`
   (30 días) cualquiera puede `expire_paused_job` para reembolsar y cerrar (evita
   fondos atrapados para siempre). La pausa global de emergencia (authority) aún
   solo frena `create_job`; extenderla a flujos activos sigue pendiente de decidir.

8. **`cancel_job` tras milestones aprobados**
   Si se aprobaron milestones (ya pagados) y luego `cancel_job` intenta
   reembolsar `amount + fee`, el PDA no tiene fondos suficientes → la transferencia
   falla (no overpay, pero el cancel se bloquea). Aceptable: cancelar solo antes
   de trabajar. Se puede refinar con un chequeo explícito.

## Validación de identidades (resumen)
- Cliente ≠ Freelancer: ✅ en `accept_job` (`CannotWorkOnOwnJob`).
- Árbitro ≠ Cliente/Freelancer: ✅ en `assign_arbiter` y `resolve_platform_case`
  (`ArbiterCannotBeParty`).
- Asesor ≠ Cliente/Freelancer: ✅ en `resolve_platform_case`.
- Destinos de pago (`freelancer`) siempre atados a `job.freelancer`: ✅ en
  `approve_work`, `approve_milestone`, `finalize_dispute_payouts`.
- Pagos desde PDA usan `new_with_signer`: ✅ (sin firmar fallaría en runtime).
- PDAs siempre se cierran al finalizar: ✅ (sin fondos atrapados).
