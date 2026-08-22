# 02 · Mensajes de Error (`ErrorCode`)

Cada variante del enum `#[error_code]` con su mensaje en inglés (on-chain) y su
traducción al español, más el significado y dónde se usa.

| # | Variante | Mensaje EN | Traducción ES | Significado |
|---|----------|-----------|--------------|-------------|
| 1 | `MathOverflow` | "Math overflow" | Desbordamiento aritmético | Una operación numérica se pasó de `u64`/`u128`. |
| 2 | `ProgramPaused` | "Program is paused" | El programa está pausado | Se intentó una instrucción mientras `config.paused == true`. |
| 3 | `AmountTooSmall` | "Amount too small" | El monto es muy pequeño | El monto es menor a `MIN_JOB_AMOUNT`. También usado si `amount == 0` en retiros. |
| 4 | `EmptyTitle` | "Title cannot be empty" | El título no puede estar vacío | Se envió un título vacío. |
| 5 | `TitleTooLong` | "Title exceeds maximum length" | El título excede el largo máximo | Título > `MAX_TITLE_LENGTH` (100). |
| 6 | `DescriptionTooLong` | "Description exceeds maximum length" | La descripción excede el largo máximo | Desc. > `MAX_DESCRIPTION_LENGTH` (500). |
| 7 | `ProposalTooLong` | "Proposal exceeds maximum length" | La propuesta excede el largo máximo | Propuesta > `MAX_PROPOSAL_LENGTH` (512). |
| 8 | `InvalidFeeBps` | "Invalid fee basis points (must be 0-10000)" | Fee en basis points inválida (debe ser 0-10000) | `fee_bps` fuera de rango en `initialize_config`. |
| 9 | `NotAuthorized` | "Not authorized" | No autorizado | El firmante no tiene permiso (p.ej. no es la autoridad del config, o `treasury` no coincide). |
| 10 | `NotJobClient` | "Not authorized - not the job client" | No autorizado - no es el cliente del job | Quien firma no es `job.client`. |
| 11 | `NotJobFreelancer` | "Not authorized - not the job freelancer" | No autorizado - no es el freelancer del job | Quien firma no es `job.freelancer`. |
| 12 | `CannotWorkOnOwnJob` | "Cannot work on your own job" | No puedes trabajar en tu propio job | El cliente intenta aceptar/ser freelancer de su propio job. |
| 13 | `InvalidJobStatus` | "Invalid job status for this operation" | Estado de job inválido para esta operación | El `job.status` no permite la instrucción. |
| 14 | `NoFreelancerAssigned` | "No freelancer assigned" | No hay freelancer asignado | Se requiere `job.freelancer` y es `None`. |
| 15 | `InvalidJob` | "Invalid job id / PDA mismatch" | Job id inválido / PDA no coincide | El PDA derivado no coincide con el job esperado. |
| 16 | `DeadlineMustBeFuture` | "Deadline must be in the future" | La deadline debe estar en el futuro | `deadline <= now` en `create_job`/`create_milestone`. |
| 17 | `InsufficientFunds` | "Insufficient funds in source account" | Fondos insuficientes en la cuenta origen | La cuenta origen no tiene suficientes lamports para transferir. |
| 18 | `CannotDisputeAtStage` | "Cannot raise dispute at this stage" | No se puede abrir disputa en esta etapa | El estado del job no permite `raise_dispute`. |
| 19 | `EmptyDisputeReason` | "Dispute reason cannot be empty" | La razón de la disputa no puede estar vacía | `reason` vacío en `raise_dispute`. |
| 20 | `EvidenceTooLong` | "Evidence exceeds maximum length" | La evidencia excede el largo máximo | Evidencia > `MAX_DISPUTE_EVIDENCE` (2048). |
| 21 | `DisputeAlreadyResolved` | "Dispute already resolved" | La disputa ya fue resuelta | Se intenta una acción sobre una disputa ya resuelta/expirada. |
| 22 | `NotValidArbiter` | "Not a valid arbiter" | No es un árbitro válido | El árbitro no está en el pool / no coincide. |
| 23 | `InvalidPercent` | "Payout percent exceeds 100" | El porcentaje de pago excede 100 | `client_payout_percent > 100` en `resolve_dispute`. |
| 24 | `MilestoneNotFound` | "Milestone not found" | Milestone no encontrado | El índice de milestone no existe. |
| 25 | `MilestoneAlreadyCompleted` | "Milestone already completed" | El milestone ya está completado | Se intenta enviar un milestone ya aprobado. |
| 26 | `MilestoneAmountExceedsFunds` | "Milestone amount exceeds remaining job funds" | El monto del milestone excede los fondos del job | La suma de montos de milestones superaría `job.amount`. |
| 27 | `AllMilestonesRequired` | "All milestones must be completed before release" | Todos los milestones deben completarse antes de liberar | Se intenta liberar sin aprobar todos los milestones. |

## Notas
- Los mensajes en inglés son los que se emiten on-chain (estándar para frontends/
  explorers). La columna ES es la traducción para la documentación y UI.
- `MathOverflow` se usa en `compute_fee` para evitar silenciar desbordamientos.
