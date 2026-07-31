# Escenario 1 · Crear y fondear un job

**Integrantes:** Cliente · Programa (`Config`, `Job`) · (Autoridad ya configuró)

## Precondiciones
- `initialize_config` ya ejecutado: `authority`, `advisor`, `treasury` y `fee_bps` fijos.
- Programa no pausado (`config.paused == false`).

## Flujo
1. **Cliente** → `create_job(job_id, title, description, amount, deadline)`
   - Valida: no pausado, `amount >= MIN_JOB_AMOUNT`, título/descripción dentro de
     límites, `deadline` en el futuro.
   - Calcula `fee_amount = compute_fee(amount, fee_bps)` y crea el PDA `Job`
     (seed `[b"job", client, job_id]`) en estado `Created`.
2. **Cliente** → `deposit_funds(job_id)`
   - Transfiere `amount + fee_amount` desde su wallet al PDA `Job`.
   - Job pasa a `Funded`. Los fondos quedan custodiados.

## Postcondiciones
- Job en `Funded`; `amount` y `fee_amount` guardados en el PDA.
- La comisión de plataforma está reservada en `fee_amount` (aún no enviada a
  `treasury`; se envía al aprobar o en disputa).

## Resumen de fees
- Aún no se paga nada a `treasury` ni a nadie. El `fee_amount` solo está apartado.

## Referencias
- `[../contract/05-jobs.md](../contract/05-jobs.md)`
- `[../contract/04-config.md](../contract/04-config.md)`
