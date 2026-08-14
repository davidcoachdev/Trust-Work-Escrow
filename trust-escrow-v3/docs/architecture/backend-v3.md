# Backend v3 — contrato arquitectónico

**Estado:** planificado, no implementado. No describe módulos presentes ni
prueba que exista una integración runtime.

## Boundary obligatorio

API y terminal/TUI consumen el mismo application service y usan
`trust-escrow-sdk` como **única frontera blockchain**. El SDK encapsula lecturas,
transacciones, tipos, errores y commitment/finality. API, TUI, workers y
reconciliador no importan Anchor ni clientes RPC directamente.

| Consumidor | Servicio | Boundary | Fuente canónica | Proyección |
|---|---|---|---|---|
| API | application service | `trust-escrow-sdk` | Solana/IDL | DB + auditoría |
| terminal/TUI | application service | `trust-escrow-sdk` | Solana/IDL | DB + auditoría |
| listener/reconciliador | worker in-process | `trust-escrow-sdk` | Solana/IDL | DB + sync/tombstones |

## Autoridad y proyección

Solana es canónica para contrato, ownership, balances y finality. DB conserva
proyecciones enriquecidas, metadata, auditoría de requests/transacciones y
cursores de sync. Una divergencia se marca, se conserva y se repara solo en DB;
no permite autorizar una mutación ni reemplaza el estado on-chain.

## Operación durable

La operación planificada persiste intents antes de enviar, idempotency keys,
firmas, reintentos clasificados, estados de finality, cursores y deduplicación.
Los cierres de cuentas se materializan como tombstones para impedir que una
proyección stale reaparezca. El worker es in-process para MVP; no es un indexer
general ni una garantía de disponibilidad distribuida.

## Signers y evidencia

Cada operación declara `read-only`, `user-signed` o `server-signed`, actor,
autoridad on-chain y signer esperado. El cambio de modo durante retry está
prohibido. Private keys no aparecen en DB, logs, errores, fixtures ni docs.

La evidencia diferencia firma/slot/commitment de un digest externo. No se afirma
un “on-chain evidence hash” porque el contrato actual no almacena uno; esa
capacidad solo podría documentarse si una futura instrucción/cuenta/IDL lo prueba.

## Artefactos verificables de trazabilidad

```text
comando/endpoint → application service → SDK operation
  → instrucción o lectura Solana → estado de finality
  → proyección DB/auditoría/sync
```

La matriz completa está en [route-matrix.md](../backend/route-matrix.md), el
catálogo machine-readable del bridge en
[sdk-operation-inventory.json](../backend/sdk-operation-inventory.json) y el
artefacto documental de proyección en
[db-projection-schema.yaml](../backend/db-projection-schema.yaml). Los tres
son contratos planificados, no evidencia de implementación runtime ni
migraciones. No se agregan endpoints ni comportamiento de producto en este
documento.
