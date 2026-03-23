# Software Design Document (SDD)
## Trust Work Escrow v2

---

## 1. Diseño de Interfaz (UI/UX)

### 1.1 Sistema de Diseño

#### Paleta de Colores por Rol

| Rol | Color Primario | Color Secundario | Uso |
|-----|----------------|------------------|-----|
| **Client** | `#3B82F6` (Azul) | `#EFF6FF` | Publicar, fondear |
| **Freelancer** | `#10B981` (Verde) | `#ECFDF5` | Trabajar, cobrar |
| **Arbiter** | `#8B5CF6` (Púrpura) | `#F5F3FF` | Resolver disputas |
| **Admin** | `#F59E0B` (Dorado) | `#FFFBEB` | Configuración |
| **Support** | `#6B7280` (Gris) | `#F9FAFB` | Tickets |

#### Tipografía
- **Font Family**: Inter (primary), system-ui (fallback)
- **Headings**: Bold, sizes 24px/20px/16px
- **Body**: Regular, size 14px
- **Code**: JetBrains Mono, size 13px

#### Espaciado
- Base unit: 4px
- Spacing scale: 4, 8, 12, 16, 24, 32, 48, 64px
- Container max-width: 1280px
- Card padding: 24px

### 1.2 Temas

| Tema | Background | Text | Border |
|------|------------|------|--------|
| **Light** | `#FFFFFF` | `#111827` | `#E5E7EB` |
| **Dark** | `#0F172A` | `#F9FAFB` | `#1E293B` |
| **Hacker** | `#000000` | `#00FF00` | `#003300` |
| **Ocean** | `#0C4A6E` | `#E0F2FE` | `#0369A1` |

---

## 2. Flujos de Usuario

### 2.1 Flujo: Publicar un Trabajo (Cliente)

```
┌─────────────────────────────────────────────────────────────────┐
│                    FLUJO: PUBLICAR TRABAJO                      │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│   ┌───────────┐      ┌──────────┐      ┌──────────┐             │
│   │ Dashboard │─────►│  Crear   │─────►│  Hitos   │             │
│   │   Jobs    │      │   Job    │      │(Opcional)│             │
│   └───────────┘      └────┬─────┘      └────┬─────┘             │
│                           │                 │                   │
│                           ▼                 ▼                   │
│                    ┌──────────┐         ┌──────────┐            │
│                    │  Resumen │◄────────│ Agregar  │            │
│                    │  Final   │         │ Detalles │            │
│                    └────┬─────┘         └──────────┘            │
│                         │                                       │
│                         ▼                                       │
│   ┌──────────────────────────────────────────────────────┐      │
│   │                   WALLET CONNECT                     │      │
│   │            Phantom / Solflare / Backpack             │      │
│   └─────────────────────────┬────────────────────────────┘      │
│                             │                                   │
│                             ▼                                   │
│                    ┌──────────────────┐                         │
│                    │  Confirmar Fee   │                         │
│                    │   5% = $25        │                        │
│                    │   Total: $525     │                        │
│                    └────────┬─────────┘                         │
│                             │                                   │
│                             ▼                                   │
│                    ┌──────────────────┐                         │
│                    │  Firmar TX       │                         │
│                    │  (Phantom)       │                         │
│                    └────────┬─────────┘                         │
│                             │                                   │
│                             ▼                                   │
│                    ┌──────────────────┐                         │
│                    │   Transacción    │                         │
│                    │   Confirmando    │                         │
│                    │    3/3 blocks    │                         │
│                    └────────┬─────────┘                         │
│                             │                                   │
│                             ▼                                   │
│                    ┌──────────────────┐                         │
│                    │ Job Publicado    │                         │
│                    │ visible en el    │                         │
│                    │ marketplace      │                         │
│                    └──────────────────┘                         │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### 2.2 Flujo: Postular a un Trabajo (Freelancer)

```
┌─────────────────────────────────────────────────────────────────┐
│                  FLUJO: POSTULAR A TRABAJO                      │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│   ┌──────────┐      ┌──────────┐      ┌──────────┐              │
│   │ Mercado  │─────►│   Job    │─────►│ Perfil   │              │
│   │  Jobs    │      │  Detail  │      │ Cliente  │              │
│   └──────────┘      └────┬─────┘      └──────────┘              │
│                          │                                      │
│                          ▼                                      │
│                    ┌──────────┐                                 │
│                    │ Ver      │                                 │
│                    │ Métricas │                                 │
│                    │ Cliente  │                                 │
│                    └────┬─────┘                                 │
│                         │                                       │
│                         ▼                                       │
│   ┌───────────┐      ┌──────────┐                               │
│   │ Retirar   │◄─────│ Propuesta│                               │
│   │Postulación│      │  Enviada │                               │
│   └───────────┘      └──────────┘                               │
│                                                                 │
│   * Esperar aceptación del cliente *                            │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### 2.3 Flujo: Resolver Disputa (Arbiter)

```
┌─────────────────────────────────────────────────────────────────┐
│                    FLUJO: RESOLVER DISPUTA                      │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│   ┌──────────┐      ┌──────────┐      ┌──────────┐              │
│   │ Panel    │─────►│ Disputa  │─────►│ Revisión │              │
│   │ Árbitros │      │  Nueva   │      │   IA     │              │
│   └──────────┘      └────┬─────┘      └────┬─────┘              │
│                          │                 │                    │
│                          ▼                 ▼                    │
│                    ┌──────────┐      ┌──────────┐               │
│                    │ Timeline │      │ Resumen │                │
│                    │  Chat    │      │  Auto    │               │
│                    └────┬─────┘      └────┬─────┘               │
│                         │                 │                     │
│                         ▼                 ▼                     │
│                    ┌──────────────────────────────────┐         │
│                    │         EVIDENCIA                │         │
│                    │  • Chat (E2EE)                   │         │
│                    │  • Archivos subidos              │         │
│                    │  • Transacciones on-chain        │         │
│                    └─────────────────┬────────────────┘         │
│                                      │                          │
│                                      ▼                          │
│                    ┌──────────────────────────────────┐         │
│                    │        DECISIÓN                  │         │
│                    │                                  │         │
│                    │  Freelancer: [____]%             │         │
│                    │  Cliente:    [____]%             │         │
│                    │  Total:      100%                │         │
│                    │                                  │         │
│                    │  [Firmar y Resolver]             │         │
│                    └──────────────────────────────────┘         │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## 3. Componentes UI

### 3.1 Botones

```tsx
// Variants
type ButtonVariant = 'primary' | 'secondary' | 'danger' | 'ghost';

// Estados
type ButtonState = 'default' | 'hover' | 'active' | 'disabled' | 'loading';

// Props
interface ButtonProps {
  variant: ButtonVariant;
  state: ButtonState;
  icon?: ReactNode;
  children: ReactNode;
  onClick?: () => void;
}

// Ejemplo de uso
<Button variant="primary" state="loading">
  Publicando...
</Button>
```

### 3.2 Cards

```tsx
// JobCard
interface JobCardProps {
  title: string;
  amount: number;
  client: UserPreview;
  status: JobStatus;
  deadline: Date;
  category: string;
  onClick?: () => void;
}

// Estados: default, hover, selected
```

### 3.3 Forms

```tsx
// JobForm
interface JobFormData {
  title: string;
  description: string;
  amount: number;
  deadline: Date;
  category: string;
  milestones?: MilestoneInput[];
}
```

### 3.4 Modals

```tsx
// WalletConnectModal
// ConfirmationModal
// DisputeModal
// MilestoneModal
```

---

## 4. Páginas

### 4.1 Landing Page

```
/                           → Landing pública
├── /#features             → Características
├── /#how-it-works         → Cómo funciona
├── /#pricing              → Comisiones
└── /#faq                  → Preguntas frecuentes
```

### 4.2 Dashboard

```
/dashboard                  → Dashboard principal
├── /dashboard/jobs         → Mis jobs
│   ├── /dashboard/jobs/new → Crear job
│   └── /dashboard/jobs/:id → Detalle job
├── /dashboard/applications → Postulaciones
├── /dashboard/teams        → Mis equipos
├── /dashboard/disputes     → Disputas (arbiter)
├── /dashboard/notifications → Notificaciones
└── /dashboard/settings     → Configuración
```

### 4.3 Marketplace

```
/jobs                       → Marketplace de jobs
├── /jobs/:id               → Detalle job público
└── /jobs/:id/apply         → Postular
```

### 4.4 Admin

```
/admin                      → Panel admin
├── /admin/users            → Gestión usuarios
├── /admin/jobs             → Gestión jobs
├── /admin/arbiter-pool     → Pool de árbitros
├── /admin/config           → Configuración
└── /admin/treasury         → Estado de treasury
```

---

## 5. Estados de UI

### 5.1 Loading States

```
┌─────────────────────────────────────┐
│           Skeleton Loader           │
│                                     │
│  ┌─────────────────────────────┐    │
│  │ ███████████████████████████ │    │
│  └─────────────────────────────┘    │
│  ┌───────────┐  ┌───────────┐       │
│  │ █████████ │  │ █████████ │       │
│  └───────────┘  └───────────┘       │
│                                     │
└─────────────────────────────────────┘
```

### 5.2 Empty States

```
┌─────────────────────────────────────┐
│              Sin Jobs               │
│                                     │
│              (icono)                │
│                                     │
│    No tienes trabajos publicados    │
│                                     │
│    [Crear primer trabajo]           │
│                                     │
└─────────────────────────────────────┘
```

### 5.3 Error States

```
┌─────────────────────────────────────┐
│           Error State               │
│                                     │
│            ❌ (icono)               │
│                                     │
│   No se pudo cargar los jobs        │
│                                     │
│   [Reintentar]  [Ver ayuda]         │
│                                     │
└─────────────────────────────────────┘
```

---

## 6. Responsive Design

### 6.1 Breakpoints

| Breakpoint | Width | Devices |
|------------|-------|---------|
| `sm` | 640px | Mobile landscape |
| `md` | 768px | Tablet |
| `lg` | 1024px | Desktop |
| `xl` | 1280px | Large desktop |

### 6.2 Grid System

```tsx
// Layout grid
<div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
  <JobCard />
  <JobCard />
  <JobCard />
  <JobCard />
</div>
```

---

## 7. Navegación

### 7.1 Desktop Navbar

```
┌────────────────────────────────────────────────────────────────────┐
│ 🛡️ COFRE     Jobs    Dashboard    [Buscar...]    [🔔] [👤 Wallet] │
└────────────────────────────────────────────────────────────────────┘
```

### 7.2 Mobile Navbar

```
┌─────────────────────────────────────┐
│ 🛡️ COFRE        [🔔] [👤]         │
├─────────────────────────────────────┤
│ Jobs  │ Dashboard │ Notificaciones │
└─────────────────────────────────────┘
```

### 7.3 Sidebar (Dashboard)

```
┌─────────────────────────────────────┐
│                                     │
│  📊 Dashboard                       │
│  💼 Mis Jobs ───────────────►       │
│     ├── + Crear Job                 │
│     └── Ver Todos                   │
│  📨 Postulaciones                   │
│  👥 Equipos                        │
│  ⚖️ Disputas                       │
│  🔔 Notificaciones                  │
│  ⚙️ Configuración                  │
│                                     │
│  ──────────────────────────        │
│  🌙 Tema: Dark                     │
│                                     │
└─────────────────────────────────────┘
```

---

## 8. Animaciones y Transiciones

### 8.1 Micro-interactions

| Elemento | Animación | Duración |
|----------|-----------|----------|
| Button hover | scale(1.02) | 150ms |
| Card hover | shadow-lg | 200ms |
| Modal open | fade + slide-up | 300ms |
| Toast | slide-in-right | 250ms |
| Page transition | fade | 200ms |

### 8.2 Feedback Visual

```tsx
// Toast notifications
type ToastType = 'success' | 'error' | 'warning' | 'info';

// Ejemplo
toast.success('Trabajo publicado exitosamente');
toast.error('Error al conectar wallet');
toast.warning('Sesión expirada');
```

---

## 9. Accesibilidad

- ARIA labels en todos los interactive elements
- Keyboard navigation (Tab, Enter, Escape)
- Focus visible states
- Color contrast ratios AA compliant
- Screen reader support

---

_Last updated: 2026-03-22_
