# Guía de Comandos CLI

## Uso General

```bash
cargo run --manifest-path cli/Cargo.toml -- [COMMAND] [OPTIONS]
```

## Comandos Disponibles

### create

Crea un nuevo trabajo (job) y deposita fondos en escrow.

```bash
escrow create "Título del trabajo" --amount 2.5 --arbiter <ARBITER_ADDRESS>
```

**Opciones:**
- `TITLE` - Título del trabajo (required)
- `--amount` - Monto a depositar en SOL (required)
- `--arbiter` - Dirección del árbitro (required)
- `--description` - Descripción del trabajo (optional)

**Ejemplo:**
```bash
escrow create "Desarrollo de App Web" --amount 2 --arbiter 7x8f9... --description "App en React"
```

---

### list

Lista todos los trabajos disponibles o los tuyos.

```bash
escrow list [OPTIONS]
```

**Opciones:**
- `--all` - Lista todos los trabajos
- `--mine` - Lista solo mis trabajos
- `--available` - Lista trabajos disponibles para aceptar
- `--status` - Filtra por estado (created, in_progress, submitted, released, disputed, resolved, cancelled)

**Ejemplo:**
```bash
escrow list --all
escrow list --available
escrow list --status submitted
```

---

### accept

Acepta un trabajo como freelancer.

```bash
escrow accept <JOB_ID>
```

**Argumentos:**
- `JOB_ID` - ID del trabajo a aceptar (required)

**Ejemplo:**
```bash
escrow accept 1
```

---

### submit

Marca el trabajo como entregado (completado).

```bash
escrow submit <JOB_ID>
```

**Argumentos:**
- `JOB_ID` - ID del trabajo entregado (required)

**Ejemplo:**
```bash
escrow submit 1
```

---

### approve

Aprueba el trabajo y libera los fondos al freelancer.

```bash
escrow approve <JOB_ID>
```

**Argumentos:**
- `JOB_ID` - ID del trabajo a aprobar (required)

**Ejemplo:**
```bash
escrow approve 1
```

---

### reject

Rechaza el trabajo y abre una disputa.

```bash
escrow reject <JOB_ID> --reason <REASON>
```

**Argumentos:**
- `JOB_ID` - ID del trabajo a rechazar (required)

**Opciones:**
- `--reason` - Razón del rechazo (required)

**Ejemplo:**
```bash
escrow reject 1 --reason "Trabajo incompleto"
```

---

### dispute

Abre una disputa (solo después de submit).

```bash
escrow dispute <JOB_ID> --reason <REASON>
```

**Argumentos:**
- `JOB_ID` - ID del trabajo en disputa (required)

**Opciones:**
- `--reason` - Razón de la disputa (required)

**Ejemplo:**
```bash
escrow dispute 1 --reason "Cliente no responde"
```

---

### resolve

Resuelve una disputa (solo árbitro).

```bash
escrow resolve <JOB_ID> --winner <WINNER>
```

**Argumentos:**
- `JOB_ID` - ID del trabajo a resolver (required)

**Opciones:**
- `--winner` - Ganador: `client` o `freelancer` (required)
- `--split` - División personalizada (optional, ej: 50-50)

**Ejemplo:**
```bash
escrow resolve 1 --winner freelancer
escrow resolve 1 --split 70-30  # 70% freelancer, 30% cliente
```

---

### cancel

Cancela un trabajo (solo si no ha sido iniciado).

```bash
escrow cancel <JOB_ID>
```

**Argumentos:**
- `JOB_ID` - ID del trabajo a cancelar (required)

**Ejemplo:**
```bash
escrow cancel 1
```

---

### show

Muestra los detalles de un trabajo específico.

```bash
escrow show <JOB_ID>
```

**Argumentos:**
- `JOB_ID` - ID del trabajo a mostrar (required)

**Ejemplo:**
```bash
escrow show 1
```

---

## Estados del Trabajo

| Estado | Descripción |
|--------|-------------|
| `Created` | Creado, esperando que freelancer acepte |
| `InProgress` | Freelancer aceptó, trabajando |
| `Submitted` | Freelancer entregó el trabajo |
| `Released` | Completado, fondos liberados |
| `Disputed` | En disputa |
| `Resolved` | Resuelto por árbitro |
| `Cancelled` | Cancelado (refund) |

---

## Flujo Típico

```
Cliente                      Freelancer                    Árbitro
   │                            │                              │
   ├── create ────────────────► │                              │
   │   (depósito funds)         │                              │
   │                            │                              │
   │                            ├── accept ──────────────────► │
   │                            │                              │
   │                            ├── submit ──────────────────► │
   │                            │                              │
   ├── approve ────────────────►│                              │
   │   (libera fondos)          │                              │
   │                            │                              │
   │  (o)                       │                              │
   │                            │                              │
   ├── reject + dispute ───────►│◄──── dispute ───────────────┤
   │                            │                              │
   │                            │◄──── resolve ───────────────┤
   │                            │    (árbitro decide)          │
```

---

## Errores Comunes

| Código | Descripción |
|--------|-------------|
| `1001` | No eres el owner del trabajo |
| `1002` | Trabajo no encontrado |
| `1003` | Estado inválido para esta acción |
| `1004` | Solo el árbitro puede resolver |
| `1005` | Fondos insuficientes |
| `1006` | Fecha límite superada |
