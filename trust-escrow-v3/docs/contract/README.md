# Trust Work Escrow v3 — Documentación del Contrato

Contrato inteligente de escrow descentralizado en Solana (Anchor 0.32.1). Esta
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

## Flujo actual de build y deploy

Desde esta carpeta, el flujo reproducible usa Anchor 0.32.1, compila el
programa con `cargo-build-sbf --arch v3`, genera el IDL y ejecuta el preflight
local antes de cualquier mutación:

```console
$ yarn build
$ ANCHOR_PROVIDER_URL=http://127.0.0.1:8899 yarn preflight
$ ANCHOR_PROVIDER_URL=http://127.0.0.1:8899 yarn deploy
```

El `Program ID` se mantiene en `Anchor.toml`, `declare_id!` y el keypair de
deploy. `preflight` deriva la pubkey del keypair y rechaza cualquier mismatch;
no hay un paso manual de sincronización de keys previo al deploy.
