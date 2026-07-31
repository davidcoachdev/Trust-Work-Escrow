# Trust Work Escrow v3 — Documentación del Contrato

Contrato inteligente de escrow descentralizado en Solana (Anchor 0.30). Esta
carpeta contiene la documentación desglosada por partes. Cada módulo/parte
tiene su propio archivo `.md` que explica **qué hace** y **por qué** se diseñó
así, incluyendo las correcciones de la auditoría de las versiones v1/v2.

## Índice de partes

| Parte | Archivo | Estado |
|-------|---------|--------|
| Visión general y decisiones de diseño | [01-overview.md](./01-overview.md) | ✅ |
| Mensajes de error (EN → ES) | [02-errores.md](./02-errores.md) | ✅ |
| Estado: cuentas, enums, constantes, helpers | [03-estado.md](./03-estado.md) | ✅ |
| Módulo Config | [04-config.md](./04-config.md) | ✅ |
| Módulo Jobs | [05-jobs.md](./05-jobs.md) | ✅ |
| Módulo Disputes | [06-disputes.md](./06-disputes.md) | ✅ |
| Módulo Milestones | [07-milestones.md](./07-milestones.md) | ✅ |
| Arbiter Pool | [08-arbiter-pool.md](./08-arbiter-pool.md) | ✅ |
| Auditoría de seguridad | [09-auditoria.md](./09-auditoria.md) | ✅ |

## Convención de documentación

Cada vez que se agrega una instrucción, cuenta o enum al contrato, se crea (o
actualiza) su `.md` correspondiente con:

- **Qué hace** la función (comportamiento on-chain).
- **Por qué** existe y las decisiones de diseño / correcciones de auditoría.
- Validaciones (`require!`) y cuentas involucradas.
- Riesgos o notas de seguridad.

> Antes de `anchor deploy` ejecutar `anchor keys sync` para reemplazar el
> `declare_id` placeholder.
