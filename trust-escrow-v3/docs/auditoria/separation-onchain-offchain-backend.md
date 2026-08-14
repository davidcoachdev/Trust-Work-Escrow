# Separación on-chain/off-chain del backend v3

**Estado:** arquitectura planificada; la capa backend v3 descrita aquí todavía no
está implementada. Este documento sincroniza el contrato documental y no es
evidencia de runtime.

## Fuente de verdad

- **Solana/IDL** es canónica para estado contractual, ownership, balances,
  instrucciones, firmas y nivel de finality observado.
- **`trust-escrow-sdk`** es la única frontera blockchain compartida por API,
  terminal/TUI, application services, listener y reconciliador. Ninguno de esos
  consumidores habla directamente con Anchor, RPC o primitivas blockchain.
- **DB** es únicamente proyección enriquecida, metadata mutable, auditoría y
  estado de sincronización. No autoriza mutaciones ni sustituye un valor
  contractual de Solana.

## Ruta única

```text
API o terminal/TUI
  → application service compartido
  → trust-escrow-sdk
  → transacción/lectura Solana
  → proyección DB + auditoría/sync
```

La misma operación debe producir los mismos tipos, errores normalizados y estados
de finality en API y terminal/TUI. La API no puede construir transacciones ni
llamar RPC por su cuenta.

## Clasificación de datos

| Dato | Autoridad | Tratamiento DB |
|---|---|---|
| Estado Job/Dispute, ownership, balances, cuentas y payouts | Solana/IDL | Proyección con `program_id`, cluster, PDA, slot, commitment y firma |
| Metadata de usuario, etiquetas y referencias externas | Aplicación/DB | Mutable; nunca se interpreta como estado contractual |
| Requests, intents, retries, firmas y errores | Auditoría/sync | Durable, sanitizado y correlacionable |
| Evidencia adjunta o digest de archivo | Fuente externa | Se etiqueta como `external`; no es compromiso del contrato |
| Cuenta cerrada | Observación canónica de Solana | Tombstone durable que impide resurrección desde datos stale |

Ante una divergencia, se conserva el conflicto, DB queda `stale`/`divergent` y
la lectura contractual usa Solana. La reparación solo modifica DB y auditoría.

## Transacciones, finality e idempotencia

Toda mutación tiene una intención durable antes del envío: idempotency key,
actor/subject autorizado, operación SDK, cluster/program ID y payload sin
secretos. Se conservan firma, intentos, error clasificado y estado:

`intent → submitted → processed → confirmed → finalized`

También se contemplan `failed` y `reorged` cuando la fuente/commitment lo
justifique. Un retry transitorio no crea una segunda intención equivalente y una
request repetida devuelve el resultado durable original o su estado actual.
Reinicios, timeouts después del envío y desconexiones deben ser recuperables sin
doble payout.

El listener y reconciliador son un **worker in-process durable planificado** para
el MVP: tienen cursor/checkpoint, backoff, deduplicación, reanudación y
reconciliación contra Solana a través del SDK. No se introduce un indexer
genérico, microservicio separado, event sourcing general ni bus distribuido.

## Modos de firma

- **`user-signed` (planificado):** la wallet del usuario firma; el backend no
  custodia ni registra la private key y no marca la operación enviada/finalizada
  antes de la autorización verificable.
- **`server-signed` (planificado):** un signer del servidor ejecuta únicamente
  operaciones permitidas por política y autoridad on-chain. No es un fallback
  silencioso de `user-signed`.
- **Lectura/read-only (planificado):** usa RPC de lectura encapsulado por el SDK y
  no implica capacidad de firmar.

Los roles, permisos, secretos y custodia son contratos documentales pendientes de
implementación; ningún ejemplo de este documento debe leerse como capacidad ya
disponible.

## Evidencia y hashes

La evidencia registra tipo, origen, timestamp, operación/transacción y nivel de
verificación (`observed`, `confirmed`, `finalized` o `external`). Un digest SHA-256
de un archivo, payload o reporte es **hash externo** salvo que el contrato/IDL
muestre explícitamente una instrucción y un campo que lo almacenen. El contrato
actual no debe describirse como si comprometiera un hash on-chain de evidencia.

## Cuentas cerradas

Cuando el SDK/listener observa el cierre de una PDA, se conserva un tombstone con
PDA, cluster/program ID, slot/finality, firma si existe, motivo y origen. Lecturas
y reconciliación no recrean una cuenta activa desde metadata stale ni convierten
rent recuperada en payout o balance contractual.

## Alcance y estado

La arquitectura, matrices, contratos de signer, estados de finality, proyección,
reconciliación y tombstones son **planned / not-yet-implemented**. Las validaciones
runtime del backend v3 no existen todavía; los reportes deben mantenerlas como
`BLOCKED` o `NOT IMPLEMENTED`, no como PASS histórico reutilizable.
