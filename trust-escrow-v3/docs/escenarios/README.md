# Escenarios de Trust Work Escrow v3

Esta carpeta documenta los **flujos de uso reales** (escenarios) de la obra
(aplicación), con los **integrantes** (actores/roles) que participan en cada uno.
Es el complemento de `[../contract/](../contract/README.md)`: allí está el "qué
hace el código"; aquí está "quién hace qué y cuándo" desde la vista de producto.

## Integrantes del sistema

Ver `actores.md` para el detalle de cada rol (responsabilidades y permisos).

| Rol | En el contrato |
|-----|----------------|
| Cliente (poster) | firmante de `create_job`/`deposit_funds`/`approve_work`/`cancel_job` |
| Freelancer | firmante de `submit_work` |
| Árbitro | del `ArbiterPool`, resuelve disputas mutuas |
| Asesor de plataforma | `config.advisor`, resuelve `PlatformCase` |
| Autoridad (authority) | `config.authority`, administra config/treasury/árbitros |
| Treasury | wallet que recibe la comisión de plataforma |
| Programa (PDAs) | `Config`, `Job`, `Dispute`, `ArbitrationEscrow`, `ArbiterPool`, `Milestone` |

## Escenarios documentados

| # | Escenario | Integrantes | Doc |
|---|-----------|-------------|-----|
| 1 | Crear y fondear un job | Cliente, Programa | [01-crear-fondear.md](./01-crear-fondear.md) |
| 2 | Entrega y aprobación (sin disputa) | Cliente, Freelancer, Treasury | [02-entrega-aprobacion.md](./02-entrega-aprobacion.md) |
| 3 | Disputa con arbitraje mutuo | Cliente, Freelancer, Árbitro, Treasury | [03-disputa-arbitraje-mutuo.md](./03-disputa-arbitraje-mutuo.md) |
| 4 | Disputa resuelta por asesor (parte no acepta) | Cliente, Freelancer, Asesor, Treasury | [04-disputa-asesor.md](./04-disputa-asesor.md) |
| 5 | Auto-aprobación por inactividad del cliente | Cliente, Freelancer, Treasury | [05-auto-aprobacion.md](./05-auto-aprobacion.md) |
| 6 | Cancelación del job | Cliente, Programa | [06-cancelacion.md](./06-cancelacion.md) |

> Cada escenario lista: integrantes, precondiciones, flujo paso a paso
> (instrucciones del contrato), postcondiciones y resumen de fees.
