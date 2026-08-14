# Actores (Integrantes) del sistema

Roles que interactúan con el contrato `trust-escrow-v3`. Cada uno tiene
permisos distintos y paga/no paga según el flujo.

## 1. Cliente (poster / empleador)
- Crea el job (`create_job`) y lo fondea (`deposit_funds`: `amount + fee`).
- Aprueba o rechaza el trabajo (`approve_work` / `reject_work`).
- Puede cancelar (`cancel_job`) antes de que inicie.
- En disputa: firma `raise_dispute` y postea su bono de arbitraje (2.5%).
- **Paga** la comisión de plataforma (vía el `fee_amount` ya depositado) y, si
  hay disputa abierta, su 2.5% de arbitraje "les guste o no".

## 2. Freelancer (trabajador)
- Acepta el job (vía aplicación) y entrega (`submit_work`).
- Cobra `amount` al aprobarse (o su `%` en disputa).
- En disputa: firma `accept_dispute` y postea su bono de arbitraje (2.5%).
- **Paga** su 2.5% de arbitraje si la disputa se abrió.

## 3. Árbitro (neutral)
- Parte del `ArbiterPool`, asignado por la plataforma (`assign_arbiter`).
- Solo interviene en **arbitraje mutuo** (ambas partes aceptaron).
- `resolve_dispute` fija el reparto; `finalize_dispute_payouts` paga.
- **No cobra on-chain:** solo autoriza; el 5% de los bonos va a
  `arbitration_treasury`. Una compensación off-chain es futura/no implementada.

## 4. Asesor de plataforma (`config.advisor`)
- Rol **separado** de `authority`.
- Resuelve `PlatformCase` (cuando una parte no acepta la disputa, o el árbitro
  falla, o el cliente desaparece tras la entrega).
- **No cobra on-chain:** solo autoriza; el 5% de los bonos va a
  `arbitration_treasury`. Una compensación off-chain es futura/no implementada.
- Es **gratis** solo en administración SIN disputa abierta (p.ej. ninguna
  intervención real).

## 5. Autoridad (`config.authority`)
- Inicializa y administra: `pause`/`unpause`, `update_treasury`, define
  `fee_bps`, crea el `ArbiterPool` y asigna árbitros (`assign_arbiter`).
- No participa en los pagos de los jobs salvo configuración.

## 6. Treasury (wallet)
- Destino de la **comisión de plataforma** (`fee_amount`), pagada al aprobar o
  en `finalize`. Quien la retira es `withdraw_treasury` (la propia wallet firma).

## 7. Programa / PDAs (el contrato mismo)
Cuentas que custodian estado y fondos:
- `Config` — parámetros globales.
- `Job` — el escrow de un trabajo (custodia `amount + fee`).
- `Dispute` — metadata de una disputa abierta.
- `ArbitrationEscrow` — bonos de arbitraje (2.5% c/u) posteados al abrir.
- `ArbiterPool` — lista de árbitros neutrales.
- `Milestone` — hitos opcionales del job.

## Principio de cobro (resumen)
- Comisión de plataforma: siempre que el job se paga (treasury).
- Arbitraje 5% (2.5% c/u): **solo si se abrió una disputa**; va a
  `arbitration_treasury`. El resolutor solo autoriza. Sin disputa abierta →
  $0$ de arbitraje.
