# 🧱 Milestone 2 — Business Foundation

**Proyecto:** Trust Work Escrow  
**Fecha:** 3 de Julio, 2026  
**Programa:** WayLearn Solana Labs Incubation

---

## 1. Propuesta de Valor — Value Proposition Canvas

### Perfil del Cliente

#### 🟢 Gains (Beneficios esperados)
| # | Gain | Evidencia |
|---|------|-----------|
| 1 | Pagos rápidos y sin demoras | Freelancers reportan esperas de 14-30 días en plataformas tradicionales |
| 2 | Comisiones justas y transparentes | Upwork cobra 20% en primeros $500, Fiverr 20% |
| 3 | Confianza sin intermediarios | Transacciones visibles y auditables en explorer |
| 4 | Protección contra impago | Fondos bloqueados en escrow hasta aprobación |
| 5 | Resolución justa de disputas | Pool de árbitros neutral en vez de algoritmo opaco |

#### 🔴 Pains (Frustraciones)
| # | Pain | Impacto |
|---|------|---------|
| 1 | Comisiones de hasta 20% por transacción | Alto — freelancers pierden $1 de cada $5 |
| 2 | Pagos lentos (14-30 días) | Alto — afecta cash flow de freelancers |
| 3 | Decisiones de disputa opacas | Medio — sin transparencia ni apelación real |
| 4 | Barreras geográficas y bancarias | Medio — freelancers sin acceso a Stripe/PayPal |
| 5 | Riesgo de chargebacks injustos | Alto — clientes pueden reversar pagos |

#### 🛠️ Customer Jobs (Tareas del usuario)
| # | Job | Tipo | Usuario |
|---|-----|------|---------|
| 1 | Publicar trabajo y contratar freelancer | Funcional | Cliente |
| 2 | Encontrar trabajo y cobrar por entregas | Funcional | Freelancer |
| 3 | Asegurar que el pago se libere solo si el trabajo está bien | Funcional | Cliente |
| 4 | Asegurar que el pago se recibe al entregar | Funcional | Freelancer |
| 5 | Resolver disputas de forma justa | Social/Emocional | Ambos |
| 6 | Mantener reputación on-chain verificable | Social | Ambos |

---

### Mapa de Valor

#### 🎁 Gain Creators
| # | Creador de ganancia | Resuelve Gain # |
|---|-------------------|-----------------|
| 1 | Liberación automática al aprobar entrega | 1 (pagos rápidos) |
| 2 | Comisión fija baja (~2-3%) vs 20% de plataformas tradicionales | 2 (comisiones justas) |
| 3 | Historial de transacciones en Solscan/SolanaFM | 3 (transparencia) |
| 4 | Fondos bloqueados en PDA, liberación multi-firma | 4 (protección) |
| 5 | Pool rotativo de árbitros de la comunidad | 5 (disputas justas) |

#### 💊 Pain Relievers
| # | Aliviador de dolor | Resuelve Pain # |
|---|-------------------|-----------------|
| 1 | 2-3% de fee vs 20% de competidores tradicionales | 1 (comisiones) |
| 2 | Pago en minutos post-aprobación en vez de 14-30 días | 2 (pagos lentos) |
| 3 | Todo el proceso de disputa on-chain y auditable | 3 (opacidad) |
| 4 | Solo necesitás wallet Solana, sin banco ni Stripe | 4 (barreras geográficas) |
| 5 | Contrato determina liberación, no hay chargeback posible | 5 (chargebacks) |

#### 🧩 Productos y Servicios
- Smart contract de escrow con lifecycle de trabajo
- Pool de árbitros descentralizado
- CLI para operaciones power-user
- (Futuro) Web app + mobile app
- (Futuro) Reputación on-chain

---

### Fit (Ajuste)

```
Freelancers crypto-native  ──tienen──>  Comisiones altas, pagos lentos, disputas opacas
       │                                       │
       │                                   Resuelve
       └────── Trust Work Escrow ──────────────┘
                    │
    ┌───────────────┴────────────────┐
    │ - 2-3% fee vs 20%             │
    │ - Pago inmediato al aprobar    │
    │ - Disputas on-chain auditables │
    │ - Sin barreras geográficas     │
    └────────────────────────────────┘
```

---

## 2. Modelo de Negocio — Business Model Canvas

| Bloque | Descripción |
|--------|-------------|
| **💎 Propuesta de Valor** | Escrow descentralizado en Solana para freelancers. Depósito en contrato inteligente, liberación automática al aprobar, disputas resueltas por pool de árbitros. Comisiones 2-3% vs 20% de Upwork/Fiverr. |
| **👥 Segmentos de Cliente** | **Primario:** Freelancers crypto-native (devs, diseñadores, creators que ya usan Solana). **Secundario:** Clientes tech-savvy (founders, CTOs que contratan freelancers). **Terciario:** DAOs y protocolos que necesitan coordinar trabajo con contractors. |
| **📢 Canales** | **Digital:** X/Twitter, Discord, comunidades Solana (Mad Lads, Superteam), LinkedIn. **Directo:** Boca a boca en comunidades crypto. **Futuro:** Onboarding asistido desde wallet (Phantom Browser). |
| **🤝 Relación con Clientes** | Self-serve vía web app. Comunidad Discord para soporte y dudas. Árbitros de la comunidad como early adopters y evangelistas. |
| **💰 Fuentes de Ingreso** | **Fee por transacción:** 2-3% sobre cada pago liberado en escrow. **Premium:** (Futuro) Suscripción para equipos con features avanzadas (múltiples wallets, reportes). **Token:** (Largo plazo) Protocol token con gobernanza y descuentos en fees. |
| **🔑 Recursos Clave** | Smart contract (activo principal), pool de árbitros, código del SDK/CLI, comunidad de usuarios, marca + confianza. |
| **⚙️ Actividades Clave** | Desarrollo y auditoría del contrato, gestión del pool de árbitros, moderación de comunidad, marketing en ecosistema Solana, integración con wallets. |
| **🤲 Socios Clave** | **Ecosistema:** Superteam, Solana Foundation, Mad Lads. **Wallets:** Phantom, Solflare. **Infra:** Helius, Triton (RPC). **Auditoría:** Neodyme, OtterSec (a futuro). |
| **📊 Estructura de Costos** | **Fijos:** Costos de desarrollo (equipo), servidores backend livianos, RPC nodes. **Variables:** Auditorías de seguridad, grants para árbitros, marketing. **Blockchain:** Costos de deploy (mínimos en Solana). |

---

## 3. Competidores y Alternativas

### Competencia Directa (escrow on-chain)

| Competidor | Propuesta | Fee | Diferenciador de TrustWork |
|------------|-----------|-----|---------------------------|
| **Scrow** | Escrow en Solana freelancer-client | ~2% | Sin pool de árbitros activo, menos tracción |
| **Talent Protocol** | Reputación on-chain + pool de talento | Subscription | No tiene escrow ni disputas |
| **Paycrest** | Puente fiat-crypto con escrow | Variable | Enfocado en pagos, no en freelance |

### Competencia Indirecta (tradicional)

| Competidor | Fee | Dolor que no resuelven |
|------------|-----|------------------------|
| **Upwork** | 20% (primeros $500), después 5-10% | Comisiones altas, disputas opacas, pagos lentos |
| **Fiverr** | 20% sobre cada transacción | Mismo problema + sin personalización |
| **Freelancer.com** | 10% + fees de retiro | Pagos lentos, disputas injustas |

### Alternativas (cómo resuelven hoy)

| Alternativa | Cómo funciona hoy | Por qué no es ideal |
|-------------|------------------|---------------------|
| Contrato legal tradicional | Acuerdo firmado, factura, transferencia bancaria | Lento, caro, sin ejecución automática |
| Crypto directo (USDT/USDC) | Cliente envía crypto directo al freelancer | Sin protección: si no paga, no hay recurso |
| Multi-sig wallet manual | Cliente + freelancer comparten wallet 2/2 | Frágil, si una parte no firma, fondos quedan trabados |

### Matriz Competitiva

| Criterio | TrustWork | Upwork | Scrow | Crypto directo |
|----------|-----------|--------|-------|----------------|
| Fee | 🟢 2-3% | 🔴 20% | 🟢 ~2% | 🟢 0% |
| Velocidad pago | 🟢 Minutos | 🔴 14-30 días | 🟢 Minutos | 🟢 Minutos |
| Protección escrow | 🟢 Sí | 🟡 Sí (pero centralizado) | 🟢 Sí | 🔴 No |
| Arbitraje | 🟢 Pool descentralizado | 🟡 Soporte centralizado | 🔴 No tiene | 🔴 No |
| Sin KYC/barreras | 🟢 Solo wallet | 🔴 KYC + banco | 🟢 Solo wallet | 🟢 Solo wallet |
| Transparencia | 🟢 100% on-chain | 🔴 Opaco | 🟢 On-chain | 🔴 Sin registro |
| Comunidad | 🟢 Ecosistema Solana | 🟡 Masivo pero tóxico | 🟡 Pequeña | N/A |

---

## 4. Primeras Hipótesis de Mercado

### Hipótesis 1: Dolor económico
> **Creemos que** freelancers crypto-native que ganan >$2K/mes en plataformas tradicionales están perdiendo $300-400/mes solo en comisiones.
> **Sabremos que es cierto cuando** al menos 5 freelancers confirmen en entrevistas que pagarían 2-3% por transacción si el pago fuera inmediato y sin riesgo de impago.
> **Métrica:** % de entrevistados que califican "comisiones altas" como su dolor #1.

### Hipótesis 2: Disputas
> **Creemos que** freelancers con más de 2 años de experiencia han tenido al menos 1 disputa no resuelta favorablemente en plataformas tradicionales.
> **Sabremos que es cierto cuando** más del 50% de los entrevistados reporten una experiencia negativa en resolución de disputas.
> **Métrica:** Tasa de experiencias negativas en disputas reportadas.

### Hipótesis 3: Disposición a usar escrow on-chain
> **Creemos que** freelancers que ya tienen wallet Solana instalada (Phantom/Solflare) están dispuestos a probar una plataforma freelance on-chain si el proceso es tan simple como conectar wallet y publicar.
> **Sabremos que es cierto cuando** al menos 3 freelancers completen el flujo de prueba en devnet sin asistencia.
> **Métrica:** % de usuarios de prueba que completan el flujo completo.

### Hipótesis 4: Segmento beachhead
> **Creemos que** el segmento más accesible inicialmente son freelancers latinoamericanos del ecosistema Solana que ya usan crypto como herramienta financiera primaria.
> **Sabremos que es cierto cuando** identifiquemos 20+ freelancers en este perfil en comunidades Solana LATAM.
> **Métrica:** Cantidad de freelancers reachables en canales del ecosistema.

### Hipótesis 5: Clientes
> **Creemos que** founders y CTOs de startups web3 prefieren contratar freelancers con reputación on-chain verificable antes que plataformas tradicionales.
> **Sabremos que es cierto cuando** al menos 2 clientes potenciales expresen interés en fondeary un trabajo de prueba.
> **Métrica:** Clientes dispuestos a participar en beta.

---

## 5. Señal de Validación Inicial

### ✅ Lo que ya validamos

| Señal | Detalle | Peso |
|-------|---------|------|
| **Hackathon completado** | Participamos y completamos el WayLearn Solana Hackathon (Marzo 2026) con producto funcional | 🟢 Alta |
| **Smart contract deployado** | 31 instrucciones en devnet, todas funcionando. Program ID: `28QTH6qfG2iKDVXNY8nUTKDjx8yrBrQpnvXCPyJsrwuA` | 🟢 Alta |
| **31 tests pasando** | Suite de tests TypeScript para todas las instrucciones del contrato | 🟢 Alta |
| **CLI funcional** | Comandos para todas las operaciones contra devnet | 🟡 Media |
| **TUI funcional** | Interfaz de terminal con temas y roles (mock data) | 🟡 Media |

### 🔄 Lo que estamos validando

| Señal | Acción | Estado |
|-------|--------|--------|
| **Interés del segmento** | Entrevistas con freelancers crypto-native LATAM | ⏳ Pendiente |
| **Disposición a pagar** | Validación de hipótesis de precio (2-3% fee) | ⏳ Pendiente |
| **Interés de clientes** | Outreach a founders/CTOs de proyectos Solana | ⏳ Pendiente |
| **Atractivo del pool de árbitros** | Evaluar si árbitros potenciales se quieren sumar | ⏳ Pendiente |

### 📋 Plan de validación inmediato (Julio)

| Semana | Acción | Meta |
|--------|--------|------|
| 29 Jun - 5 Jul | Encuesta rápida en comunidades Solana LATAM | 20+ respuestas |
| 6-12 Jul | Entrevistas 1:1 con freelancers | 5-8 entrevistas |
| 13-19 Jul | Entrevistas con potenciales clientes | 3-5 entrevistas |
| 20-26 Jul | Prueba de flujo en devnet con usuarios | 3 usuarios completan flujo |
| 27-31 Jul | **Reporte de validación (Milestone 4)** | Síntesis de aprendizajes |

---

## Criterio de Aceptación ✅

| Criterio | Cumplimiento |
|----------|-------------|
| 🎯 **Oportunidad de negocio identificada** | Freelancers crypto-native pierden $300-400/mes en comisiones. Escrow on-chain con pool de árbitros resuelve el problema a 2-3% de fee. |
| 📡 **Señal concreta para validar** | Smart contract funcional en devnet. Hipótesis definidas. Plan de entrevistas y encuestas para Julio. |

---

*Próximo entregable: 🏗️ Milestone 3 — Arquitectura técnica del MVP (Viernes 10 de Julio)*
