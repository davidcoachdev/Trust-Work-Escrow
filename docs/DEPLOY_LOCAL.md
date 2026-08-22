# Deploy Local — Guía de Puesta en Marcha

Antes de usar la TUI o el CLI necesitas tener el smart contract desplegado y configurado en el validador local. Este proceso es obligatorio cada vez que reinicies el validador desde cero.

---

## ⚠️ Error si no lo haces

Si intentas crear un job sin hacer el deploy obtendrás:

```
Error: RPC response error -32002: Transaction simulation failed:
Attempt to load a program that does not exist
```

---

## Pasos (en orden)

### 1. Iniciar el validador local

En una terminal separada que dejes corriendo:

```bash
solana-test-validator --reset
```

> `--reset` limpia el estado anterior. Sin este flag, si el validador ya tenía datos puede haber conflictos.

---

### 2. Configurar Solana CLI para localhost

```bash
solana config set --url localhost
```

---

### 3. Fondear la wallet

```bash
solana airdrop 10
```

Verifica el saldo:

```bash
solana balance
```

---

### 4. Compilar el smart contract

Desde la raíz del proyecto o desde `trust-escrow/`:

```bash
cd trust-escrow
anchor build
```

---

### 5. Desplegar el programa

```bash
anchor deploy
```

Salida esperada:

```
Deploying cluster: http://127.0.0.1:8899
Program Id: 5gu5JCSpB8MKyJzhXpGaCt8SruAMnRD6cTPbwPX6JTYo
Deploy success
```

---

### 6. Inicializar la configuración del programa

Este paso configura el treasury (la cuenta que recibe el 5% de fee). Solo se hace **una vez por despliegue**.

```bash
# Usar la wallet treasury dedicada (recomendado para separación de roles)
cargo run --manifest-path cli/Cargo.toml -- init --treasury $(solana-keygen pubkey ~/.config/solana/treasury.json)
```

Salida esperada:

```
✅ Config initialized!
   Treasury: 7zDZsccCyYYNPNa7VfxNZHxvX4WPUZ7odwMsVz4DTZ8e
   Tx: <signature>
```

> **Seguridad**: Usa una wallet treasury separada (`treasury.json`) y no la misma del admin.
> Así, Admin controla el programa y Treasury controla los fondos — dos claves distintas.

---

## Resumen: comandos en orden

```bash
# Terminal 1 — dejar corriendo
solana-test-validator --reset

# Terminal 2 — ejecutar una vez
solana config set --url localhost
solana airdrop 10
cd trust-escrow
anchor build
anchor deploy
cargo run --manifest-path cli/Cargo.toml -- init --treasury $(solana address)
```

A partir de aquí puedes usar la TUI o el CLI normalmente.

---

## ¿Cuándo repetir este proceso?

| Situación                            | ¿Necesitas repetir?      |
| ------------------------------------ | ------------------------ |
| Reinicias el validador con `--reset` | ✅ Sí, pasos 3 al 6      |
| Reinicias el validador sin `--reset` | Solo si el deploy cambió |
| Solo reinicias la TUI o el CLI       | ❌ No                    |
| Cambias de red (devnet, mainnet)     | ✅ Sí, deploy en esa red |

---

## Verificar que todo está listo

```bash
# Ver el saldo de la wallet
solana balance

# Ver el programa desplegado
solana program show 5gu5JCSpB8MKyJzhXpGaCt8SruAMnRD6cTPbwPX6JTYo

# Intentar un show (debería dar AccountNotFound, no "program does not exist")
cargo run --manifest-path cli/Cargo.toml -- show 1 --client $(solana address)
```

Si el último comando responde `AccountNotFound` (y no `program does not exist`), el deploy e init fueron correctos.

---

## Logs del programa

Los logs de las transacciones se guardan en:

```
trust-escrow/.anchor/program-logs/5gu5JCSpB8MKyJzhXpGaCt8SruAMnRD6cTPbwPX6JTYo.trust_escrow.log
```

Útil para depurar errores de instrucciones.
