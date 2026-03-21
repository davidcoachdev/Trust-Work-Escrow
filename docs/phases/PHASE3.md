# Phase 3: Testing - Trust Work Escrow v2

## Descripción

Fase de testing e integración del smart contract.

## Fecha

2026-03-21

## Estado

⏳ En progreso

---

## Tests Incluidos

### Config Tests
- `initialize_config` - Verifica inicialización con multisig

### User Tests
- `create_user` - Crea cuenta de usuario con username
- `add_wallet` - Agrega wallet secundaria
- `set_active_wallet` - Cambia wallet activa

### Job Tests
- `create_job` - Crea trabajo con título, descripción, monto, deadline
- `accept_job` - Freelancer acepta trabajo
- `submit_work` - Freelancer envía trabajo completado
- `approve_work` - Cliente aprueba y transfiere fondos

---

## Cómo Ejecutar Tests

```bash
# Instalar Anchor CLI si no está
avm install latest
avm use latest

# Compilar
anchor build

# Ejecutar tests
anchor test
```

---

## Resultados Esperados

- ✅ Todos los tests pasan
- ✅ Coverage > 80% en funciones core
- ✅ IDL generado en `target/idl/`

---

## Siguiente

Phase 4: Deployment - Desplegar a devnet