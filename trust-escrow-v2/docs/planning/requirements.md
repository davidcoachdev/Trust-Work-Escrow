# Requerimientos Funcionales
## Trust Work Escrow v2

---

## 1. Autenticación y Usuarios

### 1.1 Autenticación Wallet
- [ ] RF001: Sistema debe permitir autenticación con wallet Solana (Phantom, Solflare, Backpack)
- [ ] RF002: Sistema debe verificar posesión de wallet mediante sign-message
- [ ] RF003: Sistema debe generar JWT tras verificación exitosa
- [ ] RF004: Sistema debe permitir múltiples wallets por usuario

### 1.2 Perfil de Usuario
- [ ] RF010: Sistema debe almacenar username único
- [ ] RF011: Sistema debe almacenar bio opcional (max 500 caracteres)
- [ ] RF012: Sistema debe almacenar avatar URL
- [ ] RF013: Sistema debe permitir edición de perfil
- [ ] RF014: Sistema debe mostrar reputación del usuario

---

## 2. Sistema de Jobs

### 2.1 Creación de Jobs
- [ ] RF100: Sistema debe permitir crear job con título (max 100 caracteres)
- [ ] RF101: Sistema debe permitir descripción (max 1000 caracteres)
- [ ] RF102: Sistema debe permitir monto en lamports
- [ ] RF103: Sistema debe permitir deadline (timestamp)
- [ ] RF104: Sistema debe permitir categoría de trabajo
- [ ] RF105: Sistema debe guardar job como DRAFT inicialmente

### 2.2 Publicación de Jobs
- [ ] RF110: Sistema debe requerir fondeo antes de publicar (105%)
- [ ] RF111: Sistema debe calcular fee automáticamente (5%)
- [ ] RF112: Sistema debe mostrar desglose antes de confirmar
- [ ] RF113: Sistema debe notificar al cliente tras publicar
- [ ] RF114: Sistema debe hacer job visible en marketplace

### 2.3 Aceptación de Jobs
- [ ] RF120: Freelancer debe poder aceptar job publicado
- [ ] RF121: Sistema debe validar que no sea el propio job del freelancer
- [ ] RF122: Sistema debe permitir aceptar con equipo
- [ ] RF123: Sistema debe notificar al cliente tras aceptación

### 2.4 Entrega de Trabajo
- [ ] RF130: Freelancer debe poder marcar trabajo como entregado
- [ ] RF131: Freelancer debe poder incluir descripción de entrega
- [ ] RF132: Sistema debe notificar al cliente

### 2.5 Aprobación y Pago
- [ ] RF140: Cliente debe poder aprobar trabajo
- [ ] RF141: Sistema debe transferir fondos automáticamente tras aprobación
- [ ] RF142: Sistema debe descontar fee de plataforma (5%)
- [ ] RF143: Sistema debe registrar transacción en ledger

### 2.6 Rechazo y Cancelación
- [ ] RF150: Cliente debe poder rechazar trabajo
- [ ] RF151: Sistema debe permitir cancelar job sin freelancer
- [ ] RF152: Sistema debe devolver fondos al cliente (menos fee si aplica)

---

## 3. Sistema de Equipos

### 3.1 Gestión de Equipos
- [ ] RF200: Sistema debe permitir crear equipo
- [ ] RF201: Sistema debe permitir agregar miembros
- [ ] RF202: Sistema debe permitir definir rol de miembro
- [ ] RF203: Sistema debe permitir definir porcentaje de pago
- [ ] RF204: Sistema debe validar que porcentajes sumen 100%

### 3.2 Roles de Equipo
- [ ] RF210: Sistema debe soportar roles: Owner, Lead, PM, Developer, Designer, QA
- [ ] RF211: Sistema debe permitir múltiples departamentos
- [ ] RF212: Sistema debe permitir cambiar rol de miembro

---

## 4. Sistema de Disputas

### 4.1 Apertura de Disputa
- [ ] RF300: Sistema debe permitir abrir disputa tras rechazo
- [ ] RF301: Sistema debe requerir razón de disputa
- [ ] RF302: Sistema debe cobrar stake de disputa (5%)
- [ ] RF303: Sistema debe asignar árbitro automáticamente

### 4.2 Resolución de Disputa
- [ ] RF310: Árbitro debe poder ver detalles del caso
- [ ] RF311: Árbitro debe poder solicitar evidencia
- [ ] RF312: Árbitro debe poder proponer distribución
- [ ] RF313: Sistema debe ejecutar distribución tras resolución

### 4.3 AI Arbitration
- [ ] RF320: Sistema debe generar resumen automático con IA
- [ ] RF321: Sistema debe detectar inconsistencias
- [ ] RF322: Sistema debe sugerir distribución basada en evidencia
- [ ] RF323: IA nunca debe ejecutar acciones financieras

---

## 5. Sistema de Hitos (Milestones)

### 5.1 Creación de Hitos
- [ ] RF400: Sistema debe permitir crear múltiples hitos por job
- [ ] RF401: Sistema debe validar que hitos no excedan monto total
- [ ] RF402: Sistema debe permitir definir deadline por hito

### 5.2 Aprobación de Hitos
- [ ] RF410: Cliente debe poder aprobar hito individual
- [ ] RF411: Sistema debe transferir monto del hito al aprobar
- [ ] RF412: Sistema debe marcar hito como completado

---

## 6. Notificaciones

### 6.1 Tipos de Notificación
- [ ] RF500: Sistema debe notificar: Job publicado
- [ ] RF501: Sistema debe notificar: Job aceptado
- [ ] RF502: Sistema debe notificar: Trabajo entregado
- [ ] RF503: Sistema debe notificar: Pago recibido
- [ ] RF504: Sistema debe notificar: Disputa abierta
- [ ] RF505: Sistema debe notificar: Disputa resuelta

### 6.2 Canales
- [ ] RF510: Sistema debe mostrar notificaciones in-app
- [ ] RF511: Sistema debe poder enviar emails (opcional)
- [ ] RF512: Sistema debe soportar WebSocket para real-time

---

## 7. Chat (Opcional)

### 7.1 Funcionalidades
- [ ] RF600: Sistema debe permitir mensajería entre cliente y freelancer
- [ ] RF601: Mensajes deben ser E2EE
- [ ] RF602: Sistema debe guardar historial
- [ ] RF603: Sistema debe permitir adjuntar archivos

---

## 8. Seguridad

### 8.1 Validaciones
- [ ] RF700: Sistema debe validar wallet antes de cada acción
- [ ] RF701: Sistema debe verificar estado de job antes de acciones
- [ ] RF702: Sistema debe prevenir double-spending
- [ ] RF703: Sistema debe prevenir self-hiring

### 8.2 Rate Limiting
- [ ] RF710: Sistema debe limitar requests por wallet
- [ ] RF711: Sistema debe bloquear después de N intentos fallidos

---

## 9. Requerimientos No Funcionales

### 9.1 Performance
- [ ] RNF001: Sistema debe responder en menos de 500ms (API)
- [ ] RNF002: Sistema debe manejar 1000 concurrent users

### 9.2 Disponibilidad
- [ ] RNF010: Sistema debe estar disponible 99.9% del tiempo
- [ ] RNF011: Sistema debe tener plan de recuperación ante desastres

### 9.3 Escalabilidad
- [ ] RNF020: Sistema debe poder escalar horizontalmente
- [ ] RNF021: Base de datos debe soportar millones de registros

---

## 10. Compliance

- [ ] RC001: Sistema debe cumplir regulaciones de AML
- [ ] RC002: Sistema debe permitir auditoría de transacciones
- [ ] RC003: Sistema debe almacenar logs por período legal

---

_Last updated: 2026-03-22_
