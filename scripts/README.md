# Scripts de Operaciones

Scripts para operar el proyecto Trust Work Escrow: deploy, verificación, backups.

## 📁 Estructura

```
scripts/
├── deploy.sh         # Deploy automatizado con pre-flight checks
├── verify-deploy.sh  # Verificar programa desplegado
├── backup-keys.sh    # Backup cifrado de keypairs
└── setup-github.sh   # Configurar GitHub en el contenedor
```

---

## 🚀 `deploy.sh`

Deploy automatizado con verificaciones pre-flight completas.

### Uso

```bash
# Detecta cluster de la variable de entorno
./scripts/deploy.sh

# Deploy a devnet
./scripts/deploy.sh devnet

# Deploy a mainnet (requiere confirmación)
./scripts/deploy.sh mainnet-beta --confirm
```

### Flags

| Flag | Descripción |
|------|-------------|
| `--confirm` | Requerido para mainnet |
| `--skip-tests` | Omitir tests (no recomendado) |
| `--skip-backup` | Omitir backup de keys |

### Pre-flight Checks

1. ✅ Herramientas instaladas (solana, anchor, cargo)
2. ✅ Saldo suficiente en la wallet
3. ✅ Build limpio sin errores
4. ✅ Tests pasan al 100%
5. ✅ `cargo clippy` sin warnings
6. ✅ Backup automático de keypairs

---

## 🔍 `verify-deploy.sh`

Verifica que el programa desplegado coincide con el código fuente.

### Uso

```bash
# Usa cluster de la variable de entorno
./scripts/verify-deploy.sh

# Verificar en devnet
./scripts/verify-deploy.sh devnet

# Verificar en mainnet
./scripts/verify-deploy.sh mainnet-beta
```

### Verificaciones

1. ✅ Hash SHA256 del binario on-chain vs build local
2. ✅ Program ID coincide con las keys del proyecto
3. ✅ El programa es ejecutable
4. ✅ IDL publicado (si aplica)
5. ✅ Autoridad de upgrade correcta

---

## 🔐 `backup-keys.sh`

Cifra y respalda keypairs críticos del programa.

### Uso

```bash
# Backup interactivo (pregunta passphrase)
./scripts/backup-keys.sh

# Especificar directorio de salida
./scripts/backup-keys.sh --output ./safe/
```

### Qué respalda

- Program keypair (define el Program ID)
- Deploy authority keypair
- IDL del programa

### Formato de salida

Archivo `.tar.gz.gpg` cifrado con GPG simétrico. Para restaurar:

```bash
gpg --decrypt backup-trustwork-2026-02-28.tar.gz.gpg | tar -xzf -
```

### ⚠️ Importante

- **NUNCA** commitees keypairs sin cifrar
- Guarda la passphrase en un lugar seguro (separado del backup)
- Haz múltiples copias en diferentes ubicaciones

---

## 🔑 `setup-github.sh`

Configura credenciales de GitHub dentro del Dev Container.

### Uso

```bash
./scripts/setup-github.sh
```

### Configuraciones

1. **Identidad de Git** (`user.name`, `user.email`)
2. **Autenticación GitHub CLI** (`gh auth login`)
3. **SSH keys** (opcional)
4. **GPG signing** (opcional)

### Notas

- Ejecutar **dentro** del Dev Container
- Solo necesitas ejecutarlo una vez
- Las credenciales persisten en el volumen de Docker

---

## 🔧 Requisitos

Todos los scripts requieren estar dentro del Dev Container con:

- Solana CLI instalado
- Anchor Framework instalado
- GPG instalado (para backups)
- GitHub CLI instalado (para setup-github)

---

## 📝 Convenciones

- Todos los scripts usan `#!/usr/bin/env bash`
- `set -euo pipefail` para manejo de errores estricto
- Colores para output legible
- Mensajes con emojis para identificar tipo:
  - ✅ Éxito
  - ❌ Error
  - ⚠️ Warning
  - ℹ️ Info
