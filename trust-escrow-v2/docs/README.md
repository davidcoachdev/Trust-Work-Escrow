# Documentación - Trust Work Escrow v2

> Documentación completa para el desarrollo de la versión 2 del proyecto.

---

## 📁 Estructura de Documentos

```
docs/
├── README.md                    ← Este archivo (índice)
│
├── planning/                   ← Planificación y requerimientos
│   ├── PRD.md                  ← Product Requirements Document
│   ├── TDD.md                  ← Technical Design Document  
│   ├── SDD.md                  ← Software Design Document
│   ├── requirements.md         ← Requerimientos funcionales
│   └── questions.md            ← Preguntas pendientes por resolver
│
├── architecture/               ← Diseño de arquitectura
│   ├── SYSTEM_DESIGN.md       ← Diagrama de flujo de sistema
│   ├── DATABASE_SCHEMA.md     ← Esquema completo de DB
│   └── API_SPEC.md            ← Especificación de endpoints
│
└── implementation/             ← Plan de implementación
    ├── SPEC_DRIVER.md         ← Especificaciones para IA
    └── IMPLEMENTATION_PLAN.md  ← Plan de desarrollo por fases
```

---

## 📋 Flujo de Trabajo Recomendado

### Paso 1: Leer Documentos Base
1. Lee `planning/PRD.md` - Visión y modelo de negocio
2. Lee `planning/TDD.md` - Diseño técnico completo
3. Lee `planning/SDD.md` - Experiencia de usuario
4. Lee `planning/requirements.md` - Lista de requerimientos

### Paso 2: Responder Preguntas
1. Abre `planning/questions.md`
2. Completa TODAS las secciones
3. Guarda el archivo

### Paso 3: Estudiar Arquitectura
1. Lee `architecture/SYSTEM_DESIGN.md`
2. Lee `architecture/DATABASE_SCHEMA.md`
3. Revisa `architecture/API_SPEC.md`

### Paso 4: Seguir Plan de Implementación
1. Lee `implementation/IMPLEMENTATION_PLAN.md`
2. Lee `implementation/SPEC_DRIVER.md`
3. Sigue las fases en orden

---

## 🎯 Resumen del Proyecto

### Visión
Sistema de escrow descentralizado para freelancers y equipos con:
- **Multi-wallet** por usuario
- **Roles flexibles** (cliente, freelancer, árbitro)
- **Equipos** con jerarquía y split automático
- **Arbitraje** asistido por IA
- **Frontend** web con wallet connect
- **Backend** Rust con Axum

### Modelo de Negocio
| Tipo | Porcentaje |
|------|------------|
| Fee entrada (cliente) | 5% |
| Fee salida (freelancer) | 5% |
| Total platform | 10% |

### Tech Stack
| Componente | Tecnología |
|------------|------------|
| Smart Contract | Anchor 0.32+ / Rust |
| SDK | Rust |
| Backend | Rust + Axum |
| Frontend | Next.js 14+ |
| CLI/TUI | Rust + Clap/Ratatui |
| DB | PostgreSQL + MongoDB |
| Cache | Redis |

---

## 📅 Estado del Proyecto

| Fase | Estado | Descripción |
|------|--------|-------------|
| Planning | ✅ Documentos creados | Listos para revisión |
| Architecture | ✅ Diseño completo | Listo para implementación |
| Implementation | ⏳ Pendiente | En espera de decisiones |

---

## 🔗 Recursos

- [Anchor Documentation](https://book.anchor-lang.com/)
- [Solana Documentation](https://docs.solana.com/)
- [Solana Cookbook](https://solanacookbook.com/)
- [WayLearn Hackathon](https://dorahacks.io/hackathon/solana-waylearn-2026/detail)

---

_Last updated: 2026-03-22_
