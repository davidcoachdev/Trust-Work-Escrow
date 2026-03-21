# Phase 2: Core Implementation - Trust Work Escrow v2

## Descripción

Implementación de las 17 instrucciones del smart contract.

## Fecha

2026-03-21

## Estado

✅ Completado

---

## Instrucciones Implementadas

### Config Instructions (4)

| Instrución | Descripción |
|------------|-------------|
| `initialize_config` | Inicializa la configuración global con admin, treasury, multisig owners y fee_percent |
| `pause` | Pausa el programa (solo admin) |
| `unpause` | Reanuda el programa (solo admin) |
| `withdraw_treasury` | Retira fondos del treasury (solo admin) |

### User Instructions (4)

| Instrución | Descripción |
|------------|-------------|
| `create_user` | Crea una cuenta de usuario como PDA derivada de la wallet |
| `add_wallet` | Agrega una wallet secundaria a la cuenta del usuario |
| `set_active_wallet` | Cambia la wallet activa para la sesión |
| `update_user` | Actualiza el perfil del usuario (bio) |

### Job Instructions (7)

| Instrución | Descripción |
|------------|-------------|
| `create_job` | Crea un nuevo trabajo/escrow |
| `deposit_funds` | Deposita fondos en el escrow |
| `accept_job` | El freelancer acepta el trabajo |
| `submit_work` | El freelancer envía el trabajo completado |
| `approve_work` | El cliente aprueba y libera fondos al freelancer |
| `reject_work` | El cliente rechaza y abre disputa |
| `cancel_job` | El cliente cancela antes de aceptación |

### Arbiter Instructions (3)

| Instrución | Descripción |
|------------|-------------|
| `register_arbiters` | Admin registra árbitros en el pool |
| `raise_dispute` | Freelancer eleva una disputa |
| `resolve_dispute` | Arbiter resuelve la disputa (distribución 70-30) |

---

## Features de Seguridad

- Validación de estado del job
- Verificación de ownership (client, freelancer, arbiter)
- Prevention de self-accept (freelancer no puede aceptar su propio job)
- Pause mechanism para emergencias
- Límites en longitudes de campos
- Validación de montos mínimos

---

## Siguiente

Phase 3: Testing - Tests de integración