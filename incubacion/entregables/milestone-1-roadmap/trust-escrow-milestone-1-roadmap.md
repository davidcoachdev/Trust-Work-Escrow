# 🧭 Milestone 1 — Roadmap Inicial del Producto

**Proyecto:** Trust Work Escrow  
**Fecha:** 26 de Junio, 2026  
**Programa:** Incubación Solana — WayLearn

---

## Problema

Las plataformas freelance tradicionales (Upwork, Fiverr) cobran hasta **20% por transacción**, tienen pagos lentos y decisiones de disputa opacas. Freelancers talentosos pierden ingresos, clientes no tienen garantías, y ambos sufren intermediarios innecesarios.

Trust Work Escrow reemplaza esa confianza centralizada por un contrato inteligente en Solana.

---

## Usuario Objetivo

**Primario — Freelancer tech + crypto native:** Desarrolladores, diseñadores y profesionales digitales que ya usan Solana, entienden DeFi y buscan una alternativa a las plataformas tradicionales. Su dolor principal: comisiones abusivas y demoras en pagos.

**Secundario — Cliente tech-savvy:** Founders, CTOs y project managers que contratan freelancers. Valoran la transparencia on-chain y quieren asegurar pagos sin intermediarios.

---

## Funcionalidades del MVP

### Must-have (esencial para el flujo core)

| # | Funcionalidad | Descripción |
|---|---------------|-------------|
| 1 | Login con wallet | Conexión con Phantom / Solflare |
| 2 | Publicar trabajo | Título, descripción, monto en SOL, deadline |
| 3 | Escrow automático | Fondos bloqueados en PDA hasta aprobación |
| 4 | Aplicar a trabajos | Freelancer postula a trabajos publicados |
| 5 | Entrega y aprobación | Freelancer entrega → cliente aprueba → pago liberado |

### Should-have (completan la experiencia)

| # | Funcionalidad | Descripción |
|---|---------------|-------------|
| 6 | Perfil de usuario | Nombre, bio, historial de trabajos |
| 7 | Dashboard | Lista de trabajos activos y estado de pagos |

### Nice-to-have (diferenciadores)

| # | Funcionalidad | Descripción |
|---|---------------|-------------|
| 8 | Sistema de disputas | Apertura de conflicto con árbitro del pool |
| 9 | Pool de árbitros | Asignación aleatoria para resolución justa |

---

## Flujo Principal del Producto

```
1. Cliente se conecta con su wallet
2. Cliente publica un trabajo y deposita fondos en escrow
3. Freelancer encuentra el trabajo y aplica
4. Cliente acepta al freelancer
5. Freelancer entrega el trabajo
6. Cliente aprueba → pago liberado automáticamente
7. (Si hay conflicto) Se abre disputa → árbitro resuelve
```

---

## Integración con Solana

Solana no es un agregado — es el núcleo del producto:

| Componente | On-chain (Solana) | Off-chain (Backend) |
|-------------|-------------------|---------------------|
| Pagos | ✅ Escrow en PDA — inmutables | — |
| Estados del trabajo | ✅ Lifecycle on-chain, público y auditable | — |
| Disputas | ✅ Resolución y ejecución final on-chain | — |
| Pool de árbitros | ✅ Registro público en el contrato | — |
| Perfiles | — | ✅ Datos detallados (bio, avatar, portafolio) |
| Marketplace | — | ✅ Búsqueda, filtros, indexación |
| Chat / comunicación | — | ✅ Mensajería entre partes |

**¿Por qué Solana?** Velocidad (~400ms de finalidad), costos casi cero (fracciones de centavo por tx), escalabilidad real (miles de tps), y ecosistema maduro (Phantom, Solflare — wallets que los usuarios ya tienen instaladas).

---

## Roadmap del Programa

```
Sem 1-2  │ Validación del problema + Roadmap     ← Estamos acá
Sem 3    │ Business Foundation
Sem 4    │ Arquitectura técnica del MVP
Sem 5-8  │ Desarrollo: migración → landing + app web + app móvil → flujo completo → disputas
Sem 9    │ Live review + iteración con feedback
Sem 10   │ MVP funcional completo
Sem 11   │ Pitch deck + preparación Demo Day
Sem 12   │ 🚀 Demo Day — 31 de Agosto
```

---

## ¿Qué Tendremos Construido al Final de la Incubación?

1. **Smart contract modular** en Anchor 1.0.2 con tests pasando
2. **Landing page** en Next.js con presentación del producto, benefits y CTA
3. **App web tipo SaaS** en Next.js con wallet connect, dashboard, y gestión completa de trabajos
4. **App móvil** para iOS y Android con las funcionalidades core de la plataforma
5. **Flujo completo funcional:** registro → publicar → escrow → aplicar → entregar → aprobar → pago
6. **Sistema de disputas** con pool de árbitros
7. **CLI actualizado** como herramienta power-user
8. **Tests automatizados** de integración
9. **Documentación** para desarrolladores y usuarios

---

## Criterio de Aceptación ✅

**Explicamos con claridad:**
- **Qué construimos:** Un protocolo de escrow descentralizado en Solana para freelancers
- **Para quién:** Freelancers crypto-native y clientes tech-savvy que buscan una alternativa justa, rápida y transparente a plataformas como Upwork
- **Por qué Solana:** Velocidad, costo casi cero, escalabilidad real y un ecosistema de wallets que los usuarios ya tienen instaladas

---

*Próximo entregable: 🧱 Milestone 2 — Business Foundation (Viernes 3 de Julio)*
