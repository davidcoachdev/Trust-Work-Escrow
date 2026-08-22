# Reparto de Datos — Trust-Work-Escrow v3

**Principio rector:** on-chain (contrato) solo lo que el programa necesita para *hacer cumplir fondos y trust*: pubkeys, amounts, enums de status, deadlines/timestamps funcionales, contadores, porcentajes y **hashes**. Todo lo descriptivo, legible por humano o voluminoso → PostgreSQL (relacional) o MongoDB (NoSQL).

---

## 1. 🔵 CONTRATOS — On-chain (Solana PDAs) · lo crítico

Lo que se queda en el contrato. Si el programa no lo usa para validar lógica de fondos/trust, NO va acá.

| PDA | Campos on-chain (se quedan) | Rol |
|---|---|---|
| `Config` | `authority`, `advisor`, `treasury`, `arbitration_treasury`, `fee_bps`, `paused`, `bump` | Config crítica del protocolo |
| `ArbitrationEscrow` | `client_bond`, `freelancer_bond` | Bonds de arbitraje (fondos) |
| `Job` | `client`, `freelancer`, `amount`, `fee_amount`, `status`, `paused`, `paused_at`, `deadline`, `submitted_at`, `milestones_total`, `milestones_approved`, `milestones_amount_total`, `applicants`, `bump` | Estado de fondos + flujo |
| `ArbiterPool` | `authority`, `arbiters[]`, `bump` | Gobernanza de árbitros |
| `Dispute` | `job`, `raised_by`, `arbiter`, `status`, `evidence_count`, `evidence_cleanup_cursor`, `deadline`, `client_payout_percent`, `freelancer_payout_percent`, `bump` | Resolución de disputa |
| `Milestone` | `job`, `amount`, `status`, `index`, `bump` | Hitos de fondos |
| `Application` | `job`, `index`, `applicant`, `proposal_hash`, `status`, `bump` | Postulación (lógica) |
| `Evidence` | `dispute`, `index`, `author`, `content_hash`, `bump` | **Solo el hash** del contenido; el contenido va a Mongo |
| `SupportTicket` | `job`, `opened_by`, `status`, `bump` | Estado del ticket de soporte |

---

## 2. 🟢 POSTGRESQL — relacional / estructurado

Datos con esquema estable, relaciones y consultas por índice.

| Entidad | Campos principales | De dónde viene |
|---|---|---|
| `users` | `wallet_principal`, `username`, `bio`, `avatar_url`, `reputation_score`, `jobs_completed`, `disputes_won/lost` | Perfiles / reputación |
| `user_wallets` | `user_id`, `wallet_address`, `provider`, `is_verified` | Multi-wallet |
| `teams` | `pda_address`, `owner_id`, `name`, `description` | Agencias/equipos |
| `team_members` | `team_id`, `user_id`, `role`, `department`, `payout_percentage` | Miembros |
| `jobs_metadata` | `pda_address`, `client_id`, `freelancer_id`, `title`, `description`, `created_at`, `updated_at` | **Sale del `Job` on-chain** |
| `milestones_metadata` | `job_pda`, `index`, `title`, `description`, `deadline`, `submitted_at`, `approved_at`, `created_at` | **Sale del `Milestone` on-chain** |
| `disputes_metadata` | `dispute_pda`, `reason`, `created_at`, `resolved_at`, `resolution` | **Sale del `Dispute` on-chain** |
| `support_tickets_metadata` | `ticket_pda`, `reason`, `created_at`, `resolved_at`, `resolution` | **Sale del `SupportTicket` on-chain** |
| `applications` | `application_pda`, `proposal`, `applied_at` | **Sale del `Application` on-chain** (texto libre) |
| `payments` | `signature`, `job_pda`, `amount`, `type`, `created_at` | Transacciones |
| `notifications` | `user_id`, `type`, `payload`, `read` | Avisos |
| `reviews` | `from`, `to`, `rating`, `comment` | Reviews/ratings |

---

## 3. 🟠 MONGODB — NoSQL / no estructurado

Datos voluminosos, libres o de serie temporal.

| Colección | Documento (ej.) | De dónde viene |
|---|---|---|
| `chat_messages` | `{ job_pda, from, to, ciphertext, created_at }` | Chat E2EE cliente ↔ freelancer |
| `dispute_evidence` | `{ dispute_pda, index, author, content, submitted_at }` | **Sale del `Evidence` on-chain** (hasta ~20KB) |
| `api_logs` | `{ method, path, auth_wallet, status, ts }` | Logs de API |
| `audit_logs` | `{ actor, action, pda, ts }` | Auditoría |
| `events` | `{ type, payload, ts }` | Eventos / streams |
| `files` | `{ ref, owner_pda, url, mime }` | Adjuntos (ref a object storage) |

---

## 4. 🔁 Mapa de lo que SALE del contrato → DB

Lo que hoy está on-chain en v3 y debe moverse:

| Campo on-chain actual | Destino | Por qué |
|---|---|---|
| `Job.title`, `Job.description` | PostgreSQL `jobs_metadata` | Texto libre, no validado, paga rent |
| `Job.created_at`, `Job.updated_at` | PostgreSQL `jobs_metadata` | Metadata temporal |
| `Milestone.title`, `Milestone.description` | PostgreSQL `milestones_metadata` | Descriptivo |
| `Milestone.deadline`, `Milestone.submitted_at`, `Milestone.approved_at`, `Milestone.created_at` | PostgreSQL `milestones_metadata` | Metadata temporal |
| `Dispute.reason`, `Dispute.created_at`, `Dispute.resolved_at`, `Dispute.resolution` | PostgreSQL `disputes_metadata` | Narrativo / metadata |
| `SupportTicket.reason`, `SupportTicket.created_at`, `SupportTicket.resolved_at`, `SupportTicket.resolution` | PostgreSQL `support_tickets_metadata` | Narrativo / metadata |
| `Application.proposal`, `Application.applied_at` | PostgreSQL `applications` | Texto libre / metadata |
| `Evidence.content`, `Evidence.submitted_at` | MongoDB `dispute_evidence` + `content_hash` on-chain | Voluminoso (~20KB); el programa solo cuenta evidencias |

---

## 5. Pendiente de modelar (capa off-chain que v3 no tiene hoy)

- Perfiles/reputación, teams, notifications, reviews → PostgreSQL.
- Chat E2EE, audit/api logs, files → MongoDB.
- Object storage para adjuntos (S3/IPFS) + referencia en Mongo/Postgres.
