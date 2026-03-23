# SPEC DRIVER - AI Arbitration Engine
## Trust Work Escrow v2

---

## 1. Propósito

El AI Arbitration Engine asiste a árbitros humanos en la resolución de disputas dentro del ecosistema COFRE, generando contexto estructurado, resúmenes y recomendaciones de distribución de fondos.

**IMPORTANTE:**
- La IA NUNCA ejecuta acciones financieras
- La IA solo ASISTE al árbitro humano
- La decisión final siempre es del ser humano
- Todas las salidas son auditables y reproducibles

---

## 2. Principios Fundamentales

### 2.1 Lo que la IA DEBE hacer

✅ Analizar contexto del dispute
✅ Generar resúmenes objetivos
✅ Detectar inconsistencias
✅ Identificar patrones de comportamiento
✅ Proponer distribuciones basadas en evidencia
✅ Calcular scores de confianza
✅ Expresar incertidumbre

### 2.2 Lo que la IA NO DEBE hacer

❌ Ejecutar transacciones
❌ Modificar estados en blockchain
❌ Acceder a fondos
❌ Decisiones binarias absolutas
❌ Inventar evidencia
❌ Asumir sin datos

---

## 3. Inputs del Sistema

### 3.1 Datos On-Chain

```json
{
  "job": {
    "id": "job_abc123",
    "title": "Landing Page para SaaS",
    "amount": 5000000000,
    "deposited_amount": 5250000000,
    "released_amount": 0,
    "status": "disputed",
    "deadline": 1714560000,
    "created_at": 1712000000,
    "submitted_at": 1714000000
  },
  "client": {
    "wallet": "XxxXxx...",
    "reputation_score": 4.5,
    "jobs_completed": 12
  },
  "freelancer": {
    "wallet": "YyyYyy...",
    "reputation_score": 4.8,
    "jobs_completed": 45
  },
  "dispute": {
    "opened_by": "freelancer",
    "reason": "Cliente cambió scope sin согласие",
    "created_at": 1714500000
  }
}
```

### 3.2 Datos Off-Chain (Chat E2EE)

```json
{
  "messages": [
    {
      "sender": "client",
      "content_encrypted": "...",
      "timestamp": 1712000100,
      "type": "text"
    },
    {
      "sender": "freelancer", 
      "content_encrypted": "...",
      "timestamp": 1712000200,
      "type": "text"
    }
  ]
}
```

### 3.3 Evidencia Adicional

```json
{
  "attachments": [
    {
      "type": "file",
      "hash": "QmXxx...",
      "name": "wireframes_v1.fig",
      "uploaded_by": "freelancer",
      "timestamp": 1713000000
    }
  ],
  "transactions": [
    {
      "type": "deposit",
      "amount": 5250000000,
      "timestamp": 1712000000
    }
  ]
}
```

---

## 4. Output JSON

### 4.1 Estructura del Output

```json
{
  "report_id": "rpt_xyz789",
  "dispute_id": "dis_abc123",
  "job_id": "job_abc123",
  
  "summary": {
    "title": "Disputa por cambio de scope",
    "narrative": "El freelancer entregó wireframes y mockups согласно lo acordado...",
    "key_points": [
      "Cliente solicitó cambios fuera del scope original",
      "Freelancer intentó negociar pero no hubo respuesta",
      "Trabajo parcial entregado y aprobado informalmente"
    ]
  },
  
  "timeline": [
    {
      "event": "Job creado",
      "timestamp": 1712000000,
      "source": "on_chain",
      "importance": "high"
    },
    {
      "event": "Cliente pide 'pequeño cambio' en scope",
      "timestamp": 1713500000,
      "source": "chat",
      "message_id": "msg_123"
    },
    {
      "event": "Freelancer advierte sobre impacto en tiempo",
      "timestamp": 1713510000,
      "source": "chat",
      "message_id": "msg_124"
    },
    {
      "event": "Cliente insiste con los cambios",
      "timestamp": 1713600000,
      "source": "chat",
      "message_id": "msg_126"
    }
  ],
  
  "analysis": {
    "client_behavior": {
      "score": 0.3,
      "flags": ["scope_creep", "unresponsive"],
      "details": "Cliente realizó 3 solicitudes fuera del scope original"
    },
    "freelancer_behavior": {
      "score": 0.85,
      "flags": ["communicative", "documented"],
      "details": "Freelancer documentó todos los cambios solicitados"
    }
  },
  
  "inconsistencies": [
    {
      "type": "scope_change",
      "severity": "high",
      "description": "Cliente admitió wanting 'mucho más' de lo contratado",
      "evidence": "chat:msg_130"
    },
    {
      "type": "missing_approval",
      "severity": "medium",
      "description": "No hay registro de aprobación formal del milestone 1",
      "evidence": "milestone:1:approved_by"
    }
  ],
  
  "evidence_strength": {
    "freelancer": 0.85,
    "client": 0.25,
    "reasoning": "Chat muestra freelancer intentó negociar cambios mientras cliente mantuvo silencio"
  },
  
  "recommended_split": [
    {
      "recipient": "freelancer",
      "wallet": "YyyYyy...",
      "percentage": 75,
      "amount_lamports": 3937500000,
      "rationale": "75% por trabajo parcial entregado согласно evidencia"
    },
    {
      "recipient": "client",
      "wallet": "XxxXxx...",
      "percentage": 25,
      "amount_lamports": 1312500000,
      "rationale": "25% por scope creep y falta de cooperación"
    }
  ],
  
  "risk_assessment": {
    "overall_risk": 0.35,
    "fraud_indicators": [],
    "dispute_complexity": "medium",
    "recommendation": "Resolver con split 75/25 а favor del freelancer"
  },
  
  "confidence": {
    "score": 0.78,
    "factors": [
      "+ Chat logs completos",
      "+ Múltiples timestamps",
      "- Scope no definido formalmente",
      "- Sin contratos adicionales"
    ]
  },
  
  "flags": [
    "scope_creep",
    "partial_delivery",
    "communication_gap",
    "unilateral_changes"
  ],
  
  "metadata": {
    "model": "gemini-2.0-flash",
    "version": "1.0",
    "processed_at": 1714600000,
    "processing_time_ms": 2340,
    "input_tokens": 1500,
    "output_tokens": 890
  }
}
```

---

## 5. Modos de Operación

### 5.1 Passive Mode

**Uso:** Generación de contexto inicial

```json
{
  "mode": "passive",
  "output": {
    "summary": "...",
    "timeline": [...],
    "confidence": { "score": 0.65 }
  }
}
```

**Características:**
- Resume el caso
- Construye timeline
- No emite recomendaciones
- Útil para que el árbitro entienda el caso rápidamente

### 5.2 Analytical Mode

**Uso:** Análisis profundo de evidencia

```json
{
  "mode": "analytical",
  "output": {
    "analysis": {...},
    "inconsistencies": [...],
    "evidence_strength": {...}
  }
}
```

**Características:**
- Detecta inconsistencias
- Analiza comportamiento de ambas partes
- Evalúa fuerza de evidencia
- No recomienda split

### 5.3 Advisory Mode

**Uso:** Recomendación de resolución

```json
{
  "mode": "advisory",
  "output": {
    "recommended_split": [...],
    "risk_assessment": {...},
    "confidence": {...}
  }
}
```

**Características:**
- Sugiere distribución de fondos
- Calcula riesgo
- Incluye reasoning detallado
- El árbitro decide usar o no la recomendación

---

## 6. Reglas de Negocio

### 6.1 Split Rules

```
REGLA 1: La suma de porcentajes debe ser exactamente 100%
  - freelancer_percentage + client_percentage = 100

REGLA 2: No se pueden proponer splits que favorezcan al que abrió la disputa
  sin evidencia sólida
  - Si opened_by == 'freelancer' Y evidence_strength.freelancer < 0.7
  - Entonces: freelancer_percentage <= 60%

REGLA 3: Siempre considerar el trabajo realizado
  - Si partial_delivery == true
  - Entonces: freelancer_percentage >= 30%

REGLA 4: Nunca proponer 100/0 o 0/100 sin evidencia overwhelming
  - Exceptions: только con confidence.score >= 0.95
```

### 6.2 Confidence Thresholds

```
HIGH_CONFIDENCE: score >= 0.80
  - Múltiples fuentes de evidencia
  - Chat logs claros
  - Documentación

MEDIUM_CONFIDENCE: score >= 0.50 && < 0.80
  - Evidencia parcial
  - Algunas inconsistencias

LOW_CONFIDENCE: score < 0.50
  - Evidencia insuficiente
  - Contradicciones
  - Requiere más input humano
```

---

## 7. Restricciones Hard

### 7.1 Prohibiciones Absolutas

```
❌ NO ejecutar functions del smart contract
❌ NO generar transactions
❌ NO modificar database state
❌ NO acceder a wallet private keys
❌ NO revelar PII de las partes
❌ NO hacer suposiciones sin evidencia
❌ NO proponer splits fuera de 0-100%
```

### 7.2 Validaciones de Input

```
✓ job_id debe existir y estar en status 'disputed'
✓ dispute_id debe existir
✓ El modelo debe recibir SOLO datos relacionados al dispute
✓ Timestamps deben ser válidos y ordenados
✓ Mensajes deben incluir sender identification
```

---

## 8. Flujo de Integración

```
┌─────────────────────────────────────────────────────────────────────────┐
│                     FLOW: AI ARBITRATION                                │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  1. Freelancer abre disputa                                             │
│     POST /api/jobs/:id/dispute                                          │
│                                                                         │
│  2. Sistema marca dispute como 'open'                                   │
│                                                                         │
│  3. Backend consulta datos:                                             │
│     - Job from PostgreSQL                                               │
│     - Messages from MongoDB (E2EE)                                      │
│     - Attachments metadata                                              │
│                                                                         │
│  4. Desencriptar messages (solo para dispute)                           │
│                                                                         │
│  5. Enviar a AI Engine:                                                 │
│     POST /api/ai/analyze                                                │
│     {                                                                   │
│       "dispute_id": "...",                                              │
│       "mode": "advisory"                                                │
│     }                                                                   │
│                                                                         │
│  6. AI Engine procesa:                                                  │
│     - Parse input                                                       │
│     - Build context                                                     │
│     - Generate report                                                   │
│     - Store in MongoDB                                                  │
│                                                                         │
│  7. Return ai_summary al backend                                        │
│                                                                         │
│  8. Backend actualiza:                                                  │
│     - dispute.ai_summary = report                                       │
│     - Emit WebSocket event                                              │
│                                                                         │
│  9. Árbitro recibe notificación                                         │
│                                                                         │
│  10. Árbitro revisa:                                                    │
│      - AI summary                                                       │
│      - Chat logs                                                        │
│      - Evidencia                                                        │
│                                                                         │
│  11. Árbitro decide:                                                    │
│      - Aceptar recomendación de IA                                      │
│      - Modificar split                                                  │
│      - Solicitar más evidencia                                          │
│                                                                         │
│  12. Árbitro ejecuta:                                                   │
│      POST /api/disputes/:id/resolve                                     │
│      {                                                                  │
│        "freelancer_percentage": 75,                                     │
│        "client_percentage": 25,                                         │
│        "reason": "Scope creep documentado"                              │
│      }                                                                  │
│                                                                         │
│  13. Backend ejecuta resolve_dispute en blockchain                      │
│                                                                         │
│  14. Smart contract distribuye fondos                                   │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

**Notas adicionales del flujo:**
- **Stake de disputa:** Ambas partes pagan 2.5% al abrir disputa (total 5%)
- **Destino del stake:** Se le paga al ÁRBITRO por su trabajo
- **Auto-aprobar:** Si el cliente no responde en 7 días después de submitted → auto-aprobado
- **Tiempo del árbitro:** 7 días para resolver
- **Extensión:** Admin puede extender 7 días más si el árbitro lo necesita
- **Penalty:** Si el árbitro no resuelve en 7 días → 5% de multa a tesorería + asignar nuevo árbitro

---

## 9. Arquitectura del AI Engine

```
┌─────────────────────────────────────────────────────────────────────┐
│                       AI ARBITRATION ENGINE                         │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │                      INPUT LAYER                             │   │
│  │                                                              │   │
│  │   ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐         │   │
│  │   │ Job DB  │  │ MongoDB │  │ Files   │  │On-Chain │         │   │
│  │   └────┬────┘  └────┬────┘  └────┬────┘  └────┬────┘         │   │
│  └────────┼────────────┼────────────┼────────────┼──────────────┘   │
│           └────────────┴────────────┴────────────┘                  │
│                          │                                          │
│                          ▼                                          │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │                    PREPROCESSING LAYER                       │   │
│  │                                                              │   │
│  │   • Deserialize JSON                                         │   │
│  │   • Validate schema                                          │   │
│  │   • Enrich with metadata                                     │   │
│  │   • Build context window                                     │   │
│  └──────────────────────────────────────────────────────────────┘   │
│                          │                                          │
│                          ▼                                          │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │                      PROMPT ENGINE                           │   │
│  │                                                              │   │
│  │   System: "Eres un árbitro neutral..."                       │   │
│  │   Context: [Job data, Messages, Evidence]                    │   │
│  │   User: "Analiza este dispute y proporciona..."              │   │
│  │                                                              │   │
│  └──────────────────────────────────────────────────────────────┘   │
│                          │                                          │
│                          ▼                                          │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │                      LLM ENGINE (Gemini)                     │   │
│  │                                                              │   │
│  │   Model: gemini-2.0-flash                                    │   │
│  │   Temperature: 0.3 (reproducible)                            │   │
│  │   Max tokens: 4096                                           │   │
│  └──────────────────────────────────────────────────────────────┘   │
│                          │                                          │
│                          ▼                                          │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │                    OUTPUT VALIDATION                         │   │
│  │                                                              │   │
│  │   • JSON schema validation                                   │   │
│  │   • Business rules verification                              │   │
│  │   • Sanitization                                             │   │
│  │   • Score normalization                                      │   │
│  └──────────────────────────────────────────────────────────────┘   │
│                          │                                          │
│                          ▼                                          │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │                      STORAGE LAYER                           │   │
│  │                                                              │   │
│  │   • Save report to MongoDB                                   │   │
│  │   • Update dispute record                                    │   │
│  │   • Log metrics                                              │   │
│  └──────────────────────────────────────────────────────────────┘   │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

---

## 10. Prompt Maestro

### 10.1 System Prompt

```
You are an impartial AI arbitration assistant for COFRE, a decentralized 
escrow protocol on Solana.

Your role is to assist human arbitrators by:
1. Summarizing dispute context
2. Identifying inconsistencies
3. Analyzing evidence strength
4. Recommending fair fund distributions

CRITICAL RULES:
- NEVER execute financial transactions
- NEVER make binary absolute decisions
- ALWAYS express uncertainty when data is insufficient
- ALWAYS cite evidence for your conclusions
- NEVER invent or assume evidence

Output format: JSON following the COFRE Arbitration Report Schema.
```

### 10.2 User Prompt Template

```
Analyze the following dispute for COFRE escrow protocol:

JOB CONTEXT:
{job_data}

DISPUTE DETAILS:
{dispute_data}

CHAT LOG (E2EE):
{chat_messages}

EVIDENCE:
{evidence}

Generate a complete arbitration report with:
1. Summary and narrative
2. Timeline of events
3. Behavioral analysis
4. Inconsistencies detected
5. Evidence strength assessment
6. Recommended fund distribution (must sum to 100%)
7. Confidence score

Output ONLY valid JSON matching the schema.
```

---

## 11. Métricas y Monitoreo

### 11.1 Métricas de Uso

```
• requests_total: Contador de requests
• requests_success: Requests exitosos
• requests_failed: Requests fallidos
• processing_time_ms: Tiempo de procesamiento
• input_tokens: Tokens de entrada
• output_tokens: Tokens de salida
• confidence_avg: Promedio de confianza
```

### 11.2 Alertas

```
• confidence_below_threshold (< 0.5)
• processing_time_exceeded (> 10s)
• error_rate_above ( > 5%)
```

---

## 12. Versioning

### 12.1 Changelog

```
v1.0 (2026-03-22)
- Initial specification
- Passive, Analytical, Advisory modes
- JSON output schema
- Prompt templates
```

---

_Last updated: 2026-03-22_
