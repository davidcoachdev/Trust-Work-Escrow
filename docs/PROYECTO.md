# Trust Work Escrow - Documentación del Proyecto

---

## 🧠 Ideación

### ¿Qué problema quieres resolver?

**Dificultad para realizar pagos seguros en relaciones freelancer-cliente.**

En trabajos remotos, tanto freelancers como clientes enfrentan riesgos:
- El cliente paga pero no recibe el trabajo acordado
- El freelancer entrega el trabajo pero no recibe pago
- No existe un mecanismo neutral que proteja a ambas partes

### ¿Quién lo vive?

- **Freelancers** que trabajan para clientes nuevos/extranjeros sin historial de confianza
- **Clientes** que contratan talento remoto y quieren asegurar que pagan solo por trabajo completado
- **Árbitros/mediadores** profesionales que resuelven disputas

### ¿Cómo lo resolverías?

Una plataforma **escrow basada en blockchain de Solana** donde:
1. El **cliente deposita** los fondos en un vault seguro (PDA)
2. El **freelancer entrega** el trabajo y lo marca como completado
3. **Ambas partes deben aprobar** para liberar el pago al freelancer
4. Si hay **desacuerdo**, cualquier parte puede abrir una **disputa**
5. Un **tercero (árbitro)** revisa evidencia y decide el resultado

### ¿Qué tiene de especial tu propuesta?

| Traditional Escrow | Trust Work Escrow |
|---|---|
| Centralizado (banco/tercero) | Descentralizado (código = confianza) |
| Comisiones altas (3-10%) | Comisiones mínimas (≤1%) |
| Lento (días/semanas) | Instantáneo (minutos en Solana) |
| Requiere identidad KYC | Pseudonimo (solo wallet) |
| Unilateral | Bilateral + arbitraje |

### ¿Qué tecnologías de Solana vas a integrar?

- **Anchor Framework** (Rust) - Smart contracts
- **SPL Tokens** - Pagos en USDC o SOL
- **PDA** (Program Derived Addresses) - Vaults seguros
- **CLI con Clap** - Interfaz de terminal en Rust

---

## 🧱 Definición del Proyecto

### Minimum Viable Product (MVP)

**Flujo básico (5 pasos):**

```
1. CREATE_JOB
   Cliente crea trabajo → deposita fondos en vault PDA
   
2. ACCEPT_JOB
   Freelancer acepta y comienza el trabajo
   
3. SUBMIT_WORK
   Freelancer marca trabajo como completado
   
4. RELEASE / DISPUTE
   - Cliente aprueba → fondos → freelancer
   - Cliente rechaza → abre disputa
   - Freelancer abre disputa si no hay acuerdo
   
5. RESOLVE_DISPUTE
   Árbitro revisa → decide distribución de fondos
```

### Arquitectura Inicial

```
┌─────────────────────────────────────────────────────────────────┐
│                        CLI (Rust + Clap)                         │
│   ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐       │
│   │ create   │  │ accept   │  │ submit   │  │ dispute  │       │
│   │ list     │  │ approve  │  │ resolve  │  │ cancel   │       │
│   └────┬─────┘  └────┬─────┘  └────┬─────┘  └────┬─────┘       │
└────────┼─────────────┼─────────────┼─────────────┼──────────────┘
         │             │             │             │
         └─────────────┴─────────────┴─────────────┘
                              │
                    ┌─────────▼─────────┐
                    │   RPC Connection  │
                    │   (Solana Devnet) │
                    └─────────┬─────────┘
                              │
         ┌────────────────────┼────────────────────┐
         │                    │                    │
┌────────▼────────┐  ┌────────▼────────┐  ┌────────▼────────┐
│  Smart Contract│  │   Backend API   │  │  Token Program │
│  (Anchor/Rust) │  │   (Optional)   │  │  (SPL Tokens)  │
│                │  │                 │  │                │
│ - Escrow PDA   │  │ - Metadata     │  │ - USDC/SOL     │
│ - State Accts  │  │ - Off-chain IPFS│  │ - Vault PDA    │
└────────────────┘  └─────────────────┘  └─────────────────┘
```

### Estructura de Carpetas Propuesta

```
trust-work-escrow/
├── programs/
│   └── escrow/
│       ├── src/
│       │   ├── lib.rs           # Entry point + instructions
│       │   ├── state.rs         # Account structs
│       │   ├── errors.rs        # Custom errors
│       │   └── constants.rs     # Seeds, fees
│       └── Cargo.toml
├── cli/                         # CLI con Clap
│   ├── src/
│   │   ├── main.rs
│   │   ├── commands.rs
│   │   └── constants.rs
│   └── Cargo.toml
├── docs/                        # Documentación
├── tests/
│   └── escrow.ts                # Integration tests
├── README.md
├── Anchor.toml
└── package.json
```

### Cuentas del Smart Contract

```rust
// 1. Global Config (Singleton)
#[account]
pub struct Config {
    pub authority: Pubkey,      // Admin
    pub fee_percent: u16,        // Ej: 100 = 1%
    pub treasury: Pubkey,        // Wallet para fees
    pub bump: u8,
}

// 2. Job/Escrow
#[account]
pub struct Job {
    pub client: Pubkey,          // Cliente
    pub freelancer: Pubkey,       // Freelancer (opcional al crear)
    pub arbiter: Pubkey,         // Árbitro designado
    pub amount: u64,            // Monto total
    pub fee: u64,               // Comision
    pub status: JobStatus,       // enum
    pub title: String,
    pub description: String,
    pub deadline: i64,
    pub created_at: i64,
    pub bump: u8,
}

// 3. JobStatus enum
pub enum JobStatus {
    Created,       // Esperando accept
    InProgress,    // Aceptado, trabajando
    Submitted,     // Trabajo entregado
    Released,      // Completado + pagado
    Disputed,      // En disputa
    Resolved,      // Resuelto por arbitro
    Cancelled,     // Cancelado (refund)
}
```

### Instrucciones del Programa

| Instruction | Descripción | Acciones |
|---|---|---|
| `create_job` | Cliente crea trabajo | +Job account, +Vault PDA, deposita fondos |
| `accept_job` | Freelancer acepta | Asigna freelancer, status → InProgress |
| `submit_work` | Freelancer entrega | status → Submitted |
| `approve_work` | Cliente aprueba | status → Released, libera fondos |
| `reject_work` | Cliente rechaza | Abre disputa, status → Disputed |
| `raise_dispute` | Freelancer abre disputa | status → Disputed |
| `resolve_dispute` | Árbiter decide | Distribuye fondos, status → Resolved |
| `cancel_job` | Cancela (solo si no started) | Refund al cliente |
| `refund_timeout` | Auto-refund si expire | Refund por inactividad |

---

## 🧩 ¿Qué te falta investigar o aprender?

### Prioridad Alta (Necesario para MVP)

1. **Anchor + Token Extensions**
   - Cómo usar `token-2022` con extensions
   - PDA vaults con `TokenExtensionDelegate`

2. **Sistema de Dispute Resolution**
   - Cómo estructurar datos de evidencia (IPFS?)
   - Lógica de distribución de fondos (50/50, 100/0, etc.)

3. **Gasless Transactions**
   - Investigar Bria u otros relayers
   - Cómo evitar que usuarios paguen gas

### Prioridad Media (Mejora el producto)

4. **Off-chain Metadata**
   - IPFS para descripciones de trabajo
   - URLs de evidencia en disputas

5. **Notificaciones**
   - Webhooks o push notifications para eventos

6. **Reputación**
   - Sistema de ratings en Solana (no on-chain, costoso)

### Prioridad Baja (Future)

7. **Milestones/Phased Payments**
   - Múltiples vaults para un job
   - Release parcial por milestone

8. **Stablecoins**
   - USDC en Solana (Portal Bridge)

---

## 📋 Checklist de Inicio

- [ ] Inicializar proyecto Anchor: `anchor init trust-escrow`
- [ ] Configurar Token Program en Anchor.toml
- [ ] Diseñar account structs (Job, Config)
- [ ] Implementar instrucción `create_job`
- [ ] Implementar instrucción `accept_job`
- [ ] Implementar `submit_work` + `approve_work`
- [ ] Implementar `raise_dispute` + `resolve_dispute`
- [ ] Escribir tests de integración
- [ ] Setup CLI con Clap
- [ ] Deploy a devnet

---

## 📚 Referencias

- [Biblioteca-Solana](https://github.com/WayLearnLatam/Biblioteca-Solana) - Ejemplo base del bootcamp
- [La-Poderosa-Biblioteca-En-Solana](https://github.com/DvdRivas/La-Poderosa-Biblioteca-En-Solana) - Cliente de referencia
- [AION SDK - Escrow with Arbiter](https://github.com/AION721963/aion-sdk)
- [Detork - Freelance Marketplace](https://github.com/mz0x0100/DetorkSmartContracts)
- [Anchor Escrow 2026 - Tutorial](https://github.com/solanakite/anchor-escrow-2026)
- [Anchor Documentation](https://www.anchor-lang.com/)

---

## 🎯 Certificación WayLearn

Este proyecto cumple con los requisitos de la certificación:

- ✅ Proyecto libre
- ✅ Desarrollado en Rust
- ✅ CRUD + PDA implementado
- ✅ Documentación clara

---

*Documento generado para Trust Work Escrow - Proyecto de Pagos Freelancer con Disputas y Arbitraje en Solana*
