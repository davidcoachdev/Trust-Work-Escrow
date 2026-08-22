# Product Requirements Document (PRD)
## Trust Work Escrow v2

---

## 1. Visión del Producto

Trust Work Escrow v2 es un protocolo de escrow descentralizado en Solana que permite pagos seguros entre clientes y freelancers (individuales o equipos), con resolución de disputas híbrida (IA + humano).

**El contrato inteligente es solo un cajero seguro e inmutable. Toda la lógica compleja vive off-chain.**

---

## 2. Modelo de Negocio

### 2.1 Comisiones (Fees)

| Tipo | Porcentaje | Descripción |
|------|------------|-------------|
| **Fee de Entrada** | 5% | Cobrado al cliente al publicar el job |
| **Fee de Salida** | 5% | Retenido al freelancer al cobrar |
| **Total Platform** | 10% | Por cada transacción exitosa |

### 2.2 Stake de Disputa

| Aspecto | Detalle |
|---------|---------|
| **Porcentaje** | 2.5% del valor total del job POR CADA PARTE |
| **Pagado por** | Cliente (2.5%) + Freelancer (2.5%) = 5% total |
| **Destino** | Se le paga al ÁRBITRO por su trabajo |
| **Recuperable** | NO - no se devuelve a ninguna parte |

### 2.3 Recuperación de Renta

- Al cerrar cada job (PDA), los lamports de "rent exemption" vuelven a la tesorería
- Esto hace que el sistema sea autosustentable en costos de red

---

## 3. Roles del Sistema

### 3.1 Roles de Usuario (On-Chain)

| Rol | Descripción | Permisos |
|-----|-------------|----------|
| **Client** | Publica y fondea trabajos | Crear jobs, depositar, aprobar/rechazar, cancelar |
| **Freelancer** | Ejecuta trabajos | Aceptar jobs, entregar trabajo, abrir disputas |
| **Arbiter** | Resuelve disputas | Ver disputas, ejecutar resolución |
| **Admin** | Gestiona configuración | Pausar programa, actualizar config, gestionar árbitros |
| **Treasurer** | Gestiona fondos | Retiros de tesorería, auditoría de fondos |

### 3.2 Roles de Equipo (Off-Chain)

| Rol | Descripción | Permisos |
|-----|-------------|----------|
| **Owner/Lead** | Líder del equipo | Firma autorizada para cobros |
| **Project Manager** | Gestor de proyecto | Gestión de hitos, aprobación de entregables |
| **Contributors** | Miembros del equipo | Reciben pagos según porcentaje |
 
### 3.3 Regla de Compatibilidad

- Un usuario puede tener múltiples roles simultáneamente
- Un usuario puede ser client, freelancer y arbiter pero no en el mismo contrato
- Múltiples wallets por usuario (max 10)

---

## 4. Estructura de Equipos

### 4.1 Concepto

Soporte para equipos/agencias con jerarquía interna, donde el pago se reparte automáticamente según porcentajes definidos.

### 4.2 Características

- Equipos con N miembros (10-20 máximo)
- Roles internos: Owner, PM, Contributors
- Departamentos: Frontend, Backend, QA, Design, etc.
- Reparto porcentual automático (suma = 100%)
- **Validación:** La suma de porcentajes debe ser exactamente 100%
---

## 5. Funcionalidades Core

### 5.1 Sistema de Jobs

| Funcionalidad | Descripción | Estado |
|---------------|-------------|--------|
| Crear job | Título, descripción, monto, deadline | ⏳ |
| Publicar job | Requiere fondeo previo (105%) | ⏳ |
| Aplicar a job | Freelancer envía solicitud | ⏳ |
| Revisar aplicación | Cliente revisa perfil y acepta/rechaza | ⏳ |
| Aceptar job | Freelancer o equipo acepta | ⏳ |
| Entregar trabajo | Freelancer marca como entregado | ⏳ |
| Aprobar trabajo | Cliente aprueba → pago automático | ⏳ |
| Auto-aprobar | Si no hay respuesta en 7 días → automático | ⏳ |
| Rechazar trabajo | Cliente rechaza → disputa | ⏳ |
| Cancelar job | Devolver fondos (sin freelancer) | ⏳ |

### 5.2 Sistema de Disputas

| Funcionalidad | Descripción | Estado |
|---------------|-------------|--------|
| Abrir disputa | Cliente o Freelancer inicia conflicto (stake requerido) | ⏳ |
| Pagar stake | Ambas partes pagan 2.5% al abrir disputa (total 5%) | ⏳ |
| Asignar árbitro | Sistema asigna automáticamente por sorteo | ⏳ |
| Extender tiempo | Admin puede extender 7 días más si el árbitro lo necesita | ⏳ |
| IA Summary | Resumen automático del caso | 🔲 |
| Resolución | Árbitro decide split | ⏳ |
| Penalty Árbitro | Si no resuelve en 7 días: 5% de multa a tesorería + nuevo árbitro | ⏳ |

### 5.3 Sistema de Hitos (Milestones) - v2 Post-Hackathon

> **Nota:** Los hitos se implementarán en v2 (post-hackathon). Incluirán:
> - Crear hitos (cliente define las fases)
> - Aprobar hitos (pago parcial por hito)
> - Validar hitos (verificación on-chain)

---

## 6. Estados del Job

```
┌──────────┐     ┌──────────┐     ┌──────────┐     ┌───────────┐
│  DRAFT   │────►│ CREATED  │────►│  FUNDED  │────►│IN_PROGRESS│
└──────────┘     └──────────┘     └──────────┘     └─────┬─────┘
                                                         │
                                                         ▼
┌──────────┐     ┌──────────┐     ┌──────────┐     ┌──────────┐
│ CANCELLED│◄────│ REJECTED │◄────│ DISPUTED │◄────│ SUBMITTED│
└──────────┘     └──────────┘     └────┬─────┘     └──────────┘
                                       │
                                       ▼
                                  ┌──────────┐
                                  │ RESOLVED │
                                   └──────────┘
```

| Estado | Descripción |
|--------|-------------|
| **DRAFT** | Borrador, no publicado |
| **CREATED** | Job creado, esperando fondeo |
| **FUNDED** | Fondos depositados (105%) |
| **APPLICATIONS_OPEN** | Freelancers pueden aplicar |
| **APPLICATION_REVIEW** | Cliente revisando aplicaciones |
| **IN_PROGRESS** | Freelancer trabajando |
| **SUBMITTED** | Trabajo entregado |
| **AUTO_APPROVED** | Auto-aprobado tras 7 días sin respuesta |
| **APPROVED** | Cliente aprobó → pago |
| **DISPUTED** | En disputa (stake de 5% requerido) |
| **RESOLVED** | Resuelto por árbitro (stake distribuido) |
| **CANCELLED** | Cancelado por cliente |

**Notas del flujo de disputas:**
- Al abrir disputa: ambas partes pagan 2.5% del monto del job (total 5%)
- El stake (5%) se le paga al ÁRBITRO por su trabajo
- El árbitro tiene 7 días para resolver
- El admin puede extender 7 días más si el árbitro lo necesita
- Si el árbitro no resuelve en el tiempo asignado: 5% de multa a tesorería + asignar nuevo árbitro

---

## 7. Seguridad

### 7.1 Validaciones On-Chain

- No self-hiring (cliente != freelancer)
- No double release (fondos solo se mueven una vez)
- Suma de payouts == balance disponible
- No wallets duplicadas
- Programa pausable por admin
- **Árbitro no puede ser:** cliente ni freelancer del job

### 7.2 Cifrado y Privacidad

- Chat E2EE (off-chain)
- Hash de archivos para integridad
- Acceso a datos de disputa solo para partes involucradas

### 7.3 Tesorería

- Wallet controlada por multisig (2-de-3)
- Threshold dinámico por monto:
  - Bajo: 1 firma
  - Medio: 2 firmas
  - Alto: 3 firmas

---

## 8. Modelo de Monetización

### 8.1 Ingresos

- 5% fee de entrada (cliente)
- 5% fee de salida (freelancer)
- Recuperación de rent de PDAs

### 8.2 Costos

- Infraestructura de hosting
- APIs de terceros (Helius, etc.)
- Costos de desarrollo

---

## 9. Stack Tecnológico

| Componente | Tecnología | Estado |
|------------|------------|--------|
| Smart Contract | Anchor 0.32+ / Rust | 🔲 |
| SDK | Rust | 🔲 |
| Backend | Rust (Axum) | 🔲 |
| Frontend | Next.js 14+ | 🔲 |
| CLI | Rust + Clap | 🔲 |
| TUI | Rust + Ratatui | 🔲 |
| DB Relacional | PostgreSQL | 🔲 |
| DB NoSQL | MongoDB | 🔲 |
| Eventos | Helius Webhooks | 🔲 |
| Notificaciones | Sistema propio (WebSockets) | 🔲 |

---

## 10. Requisitos del Hackathon

| Requisito | Estado |
|-----------|--------|
| Proyecto en Solana | ⏳ |
| Frontend conectado | ⏳ |
| CLI/TUI funcional | ⏳ |
| Video demo (3 min) | 🔲 |
| Código funcional | 🔲 |

**Prioridad:** Primero lograr CLI funcional → luego frontend para el demo.

---

## 11. Multi-Wallet

- Máximo **5 wallets** por usuario
- Verificación por **sign-message** (recomendado)
- Wallet primaria vs secundarias

---

## 12. Roadmap

### Fase 1: Foundation
- Diseño de smart contract
- Definición de arquitectura
- Schema de base de datos

### Fase 2: Core Implementation
- Smart contract funcional
- SDK en Rust
- CLI y TUI básicos

### Fase 3: Backend & Frontend
- API backend
- Frontend web
- Wallet connect

### Fase 4: Advanced Features
- Chat E2EE
- Sistema de IA
- Notificaciones

---

_Feature Flags para versiones futuras (no en v1):_
- USDC integration
- Milestones
- Sub-teams
- Multi-idioma
- Email notifications
- Squads Protocol multisig

_Last updated: 2026-03-22_
