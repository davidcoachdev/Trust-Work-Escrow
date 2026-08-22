# Scripts de Construcción del Dev Container

Estos scripts modularizan la construcción del contenedor Docker. Son llamados automáticamente desde el `Dockerfile` durante el build.

## 📁 Estructura

```
.devcontainer/scripts/
├── install-base-deps.sh   # Dependencias del sistema (root)
├── install-node.sh        # Node.js LTS (root)
├── install-gh-cli.sh      # GitHub CLI (root)
├── setup-user.sh          # Crear usuario vscode (root)
├── install-rust.sh        # Rust + componentes (vscode)
├── install-solana.sh      # Solana CLI (vscode)
└── post-create.sh         # Ejecuta después de crear el contenedor
```

## 🔧 Scripts

### `install-base-deps.sh`
Instala dependencias base del sistema Ubuntu 24.04:
- Build tools: `build-essential`, `pkg-config`, `cmake`, `clang`
- Librerías: `libssl-dev`, `libudev-dev`, `libclang-dev`
- Utilidades: `curl`, `wget`, `git`, `jq`, `unzip`, `xz-utils`
- Init system: `tini`

**Ejecutado como:** root

---

### `install-node.sh`
Instala Node.js LTS directamente desde nodejs.org.

**Uso:**
```bash
./install-node.sh [VERSION_MAJOR]
```

**Ejemplo:**
```bash
./install-node.sh 20  # Instala Node.js v20.x (última versión)
```

**Instala también:** `yarn`, `typescript`, `ts-node`

**Ejecutado como:** root

---

### `install-gh-cli.sh`
Instala GitHub CLI descargando el binario directamente.

**Uso:**
```bash
./install-gh-cli.sh [VERSION]
```

**Ejemplo:**
```bash
./install-gh-cli.sh 2.67.0
```

**Ejecutado como:** root

---

### `setup-user.sh`
Configura el usuario `vscode` con UID 1000:
- Crea el usuario si no existe
- Configura sudo sin contraseña
- Crea el directorio home

**Ejecutado como:** root

---

### `install-rust.sh`
Instala Rust stable usando rustup.

**Componentes instalados:**
- `rustfmt` - Formateador de código
- `clippy` - Linter

**Ejecutado como:** vscode (usuario no-root)

---

### `install-solana.sh`
Instala Solana CLI y configura el ambiente.

**Uso:**
```bash
./install-solana.sh [VERSION]
```

**Ejemplo:**
```bash
./install-solana.sh stable  # Última versión estable
./install-solana.sh 1.18.0  # Versión específica
```

**También:**
- Genera keypair por defecto en `~/.config/solana/id.json`
- Configura URL a localhost

**Ejecutado como:** vscode (usuario no-root)

---

### `post-create.sh`
Script ejecutado automáticamente después de crear el contenedor (definido en `devcontainer.json`).

Úsalo para:
- Instalar dependencias del proyecto
- Configuraciones iniciales
- Verificaciones de ambiente

---

## ⚠️ Notas Importantes

1. **Finales de línea:** Los scripts DEBEN tener finales de línea Unix (LF). El archivo `.gitattributes` asegura esto automáticamente.

2. **No ejecutar manualmente:** Estos scripts están diseñados para ser ejecutados durante el build de Docker, no directamente en tu máquina.

3. **Orden de ejecución:** El Dockerfile ejecuta los scripts en orden específico. No cambiar el orden sin entender las dependencias.

4. **Depuración:** Si un script falla durante el build:
   ```bash
   # Ver logs detallados
   docker build --progress=plain -t test .
   ```

---

## 🔄 Modificar versiones

Las versiones se definen como ARGs en el Dockerfile:

```dockerfile
ARG SOLANA_CLI_VERSION="stable"
ARG ANCHOR_VERSION="0.32.1"
ARG NODE_VERSION="20"
ARG GH_CLI_VERSION="2.67.0"
```

También puedes sobrescribirlas en `devcontainer.json`:

```json
"build": {
    "args": {
        "NODE_VERSION": "22",
        "SOLANA_CLI_VERSION": "1.18.0"
    }
}
```
