# 🔍 Milestone 4 — Validación Inicial con Usuarios

**Proyecto:** Trust Work Escrow  
**Fecha:** 31 de Julio, 2026  
**Programa:** WayLearn Solana Labs Incubation

---

## Índice

1. [Hipótesis Principal](#1-hipótesis-principal)
2. [Perfiles de Usuario](#2-perfiles-de-usuario)
3. [Plan de Validación](#3-plan-de-validación)
4. [Entrevistas en Profundidad](#4-entrevistas-en-profundidad)
5. [Encuesta de Validación](#5-encuesta-de-validación)
6. [Testing de Prototipo/MVP](#6-testing-de-prototipomvp)
7. [KPIs y Métricas](#7-kpis-y-métricas)
8. [Registro de Sesiones](#8-registro-de-sesiones)
9. [Reporte de Resultados](#9-reporte-de-resultados)
10. [Cronograma](#10-cronograma)

---

## 1. Hipótesis Principal

### Hipótesis General

> Creemos que **freelancers tech con experiencia en crypto** tienen el problema de **comisiones abusivas (hasta 20%) y demoras en pagos (14-30 días)** en plataformas como Upwork y Fiverr, y que nuestro MVP les ayudará a **recibir pagos más rápidos, con comisiones justas y transparencia on-chain** mediante un sistema de escrow descentralizado en Solana.

### Sub-hipótesis a Validar

| # | Hipótesis | Métrica |
|---|-----------|---------|
| H1 | Los freelancers tech están dispuestos a migrar de plataformas tradicionales si las comisiones bajan del 10% | % de interesados en probar > 60% |
| H2 | La transparencia on-chain (ver fondos bloqueados en explorer) genera más confianza que el sistema de reputación tradicional | Calificación de confianza > 4/5 |
| H3 | Los clientes valoran no tener que pagar tarifas de plataforma (20%) aunque tengan que gestionar el pago ellos mismos | % clientes que completarían setup > 50% |
| H4 | El sistema de disputas con árbitros es percibido como más justo que el algoritmo opaco de Upwork/Fiverr | Calificación de justicia > 3.5/5 |
| H5 | Los freelancers en LATAM tienen más dolor con pagos internacionales que los de USA/Europa | Dolor reportado (escala 1-5) en LATAM > 4 |

---

## 2. Perfiles de Usuario

### Persona 1 — Freelancer Tech + Crypto Native (Primario)

| Atributo | Descripción |
|----------|-------------|
| **Nombre** | Alex (ella/él) |
| **Edad** | 25-35 años |
| **Ocupación** | Desarrollador/a blockchain, diseñador/a Web3, o ingeniero/a smart contracts |
| **Ubicación** | LATAM (México, Colombia, Argentina, Brasil) o Europa del Este |
| **Experiencia crypto** | Alta — usa Solana/Ethereum, tiene wallet, entiende DeFi |
| **Plataformas actuales** | Upwork, Fiverr, LaborX (con frustración) |
| **Ingreso mensual** | $2,000 - $8,000 USD |
| **Dolor principal** | "Upwork me cobra 20% en los primeros $500 y las transferencias tardan 2 semanas en llegar" |
| **Motivación** | Encontrar alternativa que no le quite un tercio de sus ingresos |
| **Canal de contacto** | Discord técnico, Telegram de Solana, r/solana, DevPost |

### Persona 2 — Cliente Tech-Savvy (Secundario)

| Atributo | Descripción |
|----------|-------------|
| **Nombre** | Sam (elle) |
| **Edad** | 30-45 años |
| **Ocupación** | Founder de startup tech, CTO, o Project Manager en Web3 |
| **Ubicación** | Global (USA, Europa, LATAM) |
| **Experiencia crypto** | Media-alta — usa wallets, entiende transacciones on-chain |
| **Plataformas actuales** | Upwork, Toptal, contratación directa |
| **Presupuesto mensual** | $5,000 - $50,000 USD en freelancers |
| **Dolor principal** | "No tengo visibilidad de dónde está mi dinero ni garantía de que el freelancer va a entregar" |
| **Motivación** | Transparencia total y pagos garantizados sin intermediarios |
| **Canal de contacto** | Twitter/X, LinkedIn, newsletters tech, Founder communities |

### Persona 3 — Árbitro (Terciario)

| Atributo | Descripción |
|----------|-------------|
| **Nombre** | Cris |
| **Edad** | 35-50 años |
| **Ocupación** | Abogado/a tech, Project Manager senior, o profesional con experiencia en resolución de conflictos |
| **Experiencia crypto** | Baja-media — dispuesto a aprender |
| **Motivación** | Participar en un sistema descentralizado de justicia, aportar su expertise |
| **Canal de contacto** | LinkedIn, comunidades legales tech, DAOs |

---

## 3. Plan de Validación

### Enfoque General

| Método | Cantidad | Objetivo | Semana |
|--------|----------|----------|--------|
| Entrevistas 1:1 | 5-8 | Validación cualitativa profunda de hipótesis | 14-18 Jul |
| Encuesta online | 20-50 respuestas | Validación cuantitativa de demanda | 21-25 Jul |
| Testing de prototipo/MVP | 3-5 usuarios | Validación de usabilidad | 28-30 Jul |

### Paso a Paso

```
Semana 1 (14-18 Jul) → Entrevistas 1:1 con freelancers y clientes
Semana 2 (21-25 Jul) → Encuesta online + análisis de resultados
Semana 3 (28-30 Jul) → Testing de prototipo + preparación del entregable
31 de Julio        → 📦 ENTREGA DEL MILESTONE 4
```

---

## 4. Entrevistas en Profundidad

### Objetivo
Validar hipótesis cualitativamente: entender dolores reales, hábitos de trabajo, y disposición a probar Trust Work Escrow.

### Guía de Preguntas — Freelancers

**Intro (romper hielo)**
1. Contame un poco sobre qué hacés y cómo conseguís tus clientes actualmente.
2. Hace cuánto que trabajás como freelancer?

**Problema (dolores)**
3. ¿Qué es lo que más te frustra de las plataformas que usás hoy para cobrar?
4. ¿Alguna vez tuviste un problema con un pago o una disputa? ¿Cómo se resolvió?
5. ¿Cuánto tiempo suele pasar entre que entregás un trabajo y recibís el pago?

**Solución (interés)**
6. Te muestro una idea rápida — un sistema donde el pago se bloquea en un contrato inteligente y se libera automáticamente cuando aprobás la entrega. Sin plataforma de por medio. ¿Qué te parece?
7. ¿Usarías un sistema así si las comisiones fueran <5% en vez del 20% de Upwork?
8. ¿Qué tendría que tener para que te cambiaras mañana?

**Cierre**
9. ¿Hay algo más que te gustaría agregar?
10. ¿Podría contactarte de nuevo cuando tengamos una versión para probar?

### Guía de Preguntas — Clientes

**Intro**
1. ¿Cómo encontrás y contratás freelancers hoy?
2. ¿Cuántos freelancers contratás por mes aprox?

**Problema**
3. ¿Qué es lo que menos te gusta del proceso actual de pago a freelancers?
4. ¿Alguna vez tuviste una mala experiencia con un freelancer y perdiste plata?
5. ¿Cómo manejás la confianza con alguien que contratás por primera vez?

**Solución**
6. ¿Qué opinás de un sistema donde el pago se bloquea al inicio y se libera solo cuando aprobás el trabajo?
7. ¿Te daria más confianza poder ver los fondos en el explorador de Solana?
8. ¿Cambiarías a una plataforma sin comisiones si tuvieras que usar wallet crypto?

### Registro de Entrevistas

> Completar durante las sesiones y volcar aquí los resultados.

| # | Fecha | Persona | Perfil | Canal | Hipótesis validadas | Insights clave | Acciones |
|---|-------|---------|--------|-------|---------------------|----------------|----------|
| 1 | | | | | | | |
| 2 | | | | | | | |
| 3 | | | | | | | |
| 4 | | | | | | | |
| 5 | | | | | | | |
| 6 | | | | | | | |
| 7 | | | | | | | |
| 8 | | | | | | | |

---

## 5. Encuesta de Validación

### Objetivo
Validar cuantitativamente las hipótesis: medir disposición a migrar, dolores más comunes, y features más valoradas.

### Estructura

**Sección 1 — Filtro (2 preguntas)**
- ¿Sos freelancer o contratás freelancers?
- ¿Usás plataformas como Upwork, Fiverr, o similares?

**Sección 2 — Dolores (3 preguntas)**
- ¿Cuál es tu mayor frustración con las plataformas actuales? (opción múltiple)
  - [ ] Comisiones altas
  - [ ] Demoras en pagos
  - [ ] Disputas injustas
  - [ ] Falta de transparencia
  - [ ] Barreras geográficas
- En escala 1-5, ¿qué tan urgente es para vos encontrar una alternativa?
- ¿Cuánto pagás en comisiones por mes aprox?

**Sección 3 — Interés (3 preguntas)**
- ¿Qué tan interesado estarías en probar un sistema de escrow en Solana? (1-5)
- ¿Qué feature es más importante para vos? (ranking)
  - [ ] Comisiones bajas (<5%)
  - [ ] Pagos inmediatos
  - [ ] Transparencia on-chain
  - [ ] Disputas justas con árbitros
- Si las comisiones fueran 2-5%, ¿migrarías en tu próximo proyecto?

**Sección 4 — Perfil (2 preguntas)**
- ¿En qué país estás basado?
- ¿Tenés experiencia con crypto/wallets?

**Duración estimada:** 3-4 minutos

### Resultados de Encuesta

> Completar cuando la encuesta esté respondida.

| Pregunta | Resultado | Interpretación |
|----------|-----------|----------------|
| | | |


---

## 6. Testing de Prototipo/MVP

### Objetivo
Validar que el usuario entiende el flujo sin explicación y completa la tarea principal.

### Escenario de Prueba

> "Acabás de conseguir un trabajo de desarrollo por 5 SOL. El cliente te dijo que use Trust Work Escrow. Llegás a la terminal y querés ver el trabajo, aceptarlo y entregarlo."

### Tareas a Evaluar

| # | Tarea | Usuarios | Completado | Tiempo | Fricciones |
|---|-------|----------|------------|--------|------------|
| 1 | Iniciar sesión con wallet | | | | |
| 2 | Navegar a trabajos disponibles | | | | |
| 3 | Ver detalle de un trabajo | | | | |
| 4 | Aplicar a un trabajo | | | | |
| 5 | Ver trabajos activos | | | | |
| 6 | Entregar trabajo | | | | |
| 7 | Ver estado del pago | | | | |

### Registro de Testing

> Completar durante las sesiones de testing.

| Usuario | Perfil | Tarea | ¿Completó? | Fricciones | Comentarios | Interés |
|---------|--------|-------|------------|------------|-------------|---------|
| | | | | | | |
| | | | | | | |
| | | | | | | |
| | | | | | | |

---

## 7. KPIs y Métricas

### Metas para el Milestone 4

| KPI | Meta | Resultado | ¿Cumplida? |
|-----|------|-----------|------------|
| Entrevistas 1:1 realizadas | ≥5 | | |
| Respuestas de encuesta | ≥20 | | |
| Usuarios que probaron MVP/CLI | ≥3 | | |
| % que completa tarea sin ayuda | ≥50% | | |
| Calificación de interés (1-5) | ≥4 | | |
| Insights accionables | ≥5 | | |
| Cambios identificados al producto | ≥3 | | |

---

## 8. Registro de Sesiones

### Template por Sesión

**Sesión #:**
**Fecha:**
**Usuario:** (alias)
**Perfil:** Freelancer / Cliente / Árbitro
**Canal:** Videollamada / Presencial / Formulario
**Duración:**

#### Notas
- 
- 
- 

#### Insights
- 
- 

#### Quotes textuales
> 

#### Acciones
- [ ] 

---

## 9. Reporte de Resultados

> Esta sección se completa al cierre del milestone (31 Jul).

### Resumen Ejecutivo

**Hipótesis validadas:**
- ...
- ...

**Hipótesis descartadas:**
- ...
- ...

**Principales aprendizajes:**
- ...
- ...

### Cambios al Producto

| # | Cambio | Prioridad | Origen (feedback) |
|---|--------|-----------|-------------------|
| 1 | | 🔴/🟡/🟢 | |
| 2 | | 🔴/🟡/🟢 | |
| 3 | | 🔴/🟡/🟢 | |

### Lo que No Cambia (validado)
- ...
- ...

### Próximos Pasos
- [ ] Refinar features según feedback
- [ ] Publicar MVP funcional (Milestone 5, 21 Ago)
- [ ] Preparar ship para testing en Semana 8

---

## 10. Cronograma

| Fecha | Actividad | Responsable |
|-------|-----------|-------------|
| 10-13 Jul | Preparar guías y reclutar participantes | ✅ Listo |
| 14-18 Jul | Realizar 5-8 entrevistas 1:1 | 🔲 |
| 14-18 Jul | Identificar patrones tempranos | 🔲 |
| 21-25 Jul | Lanzar encuesta online + difundir en comunidades | 🔲 |
| 21-25 Jul | Analizar resultados de encuesta | 🔲 |
| 28-30 Jul | Testing de prototipo con 3-5 usuarios | 🔲 |
| 28-30 Jul | Compilar reporte final | 🔲 |
| **31 Jul** | **📦 ENTREGA MILESTONE 4** | 🔲 |

---

## Apéndice A: Canales para Reclutar Participantes

| Perfil | Canal | Estrategia |
|--------|-------|------------|
| Freelancer crypto | r/solana, r/cryptocurrency, Solana Discord | Post breve + link a encuesta |
| Freelancer tech | r/freelance, r/digitalnomad, grupos de WhatsApp | Post pidiendo opinión |
| Cliente tech | LinkedIn, Twitter/X, newsletters | Mensaje directo a founders |
| Árbitro potencial | LinkedIn legal tech, DAOs | Outreach directo |

## Apéndice B: Referencias del Programa

| Recurso | Link |
|---------|------|
| Guía User Research | `incubacion/referencias/10-user-research-guia.md` |
| Guía Validación de Idea | `incubacion/referencias/09-validacion-idea-guia.md` |
| Guía Design Thinking | `incubacion/referencias/08-design-thinking-guia.md` |
| Guía Testing de MVP | `incubacion/referencias/13-testing-mvp-guia.md` |
| Requisitos de Milestones | `incubacion/referencias/programa-milestones.md` |
