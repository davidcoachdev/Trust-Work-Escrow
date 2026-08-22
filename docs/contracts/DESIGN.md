# Diseño del Smart Contract v2

## Resumen de Decisiones

### 1. Usuarios y Wallets

```mermaid
erDiagram
    USER {
        Pubkey wallet_principal "Primary wallet"
        vector_Pubkey wallets_asociadas "Secondary wallets"
        Pubkey active_wallet "Currently active"
        string username "Display name"
        string bio "Optional bio"
        i64 created_at "Creation timestamp"
    }
```

**Semántica**: Una cuenta de usuario puede tener múltiples wallets associadas (como multiboot). Solo una activa por sesión, pero puede cambiar.

---

### 2. Roles (Libertad total)

```mermaid
flowchart LR
    subgraph "Roles del Sistema"
        Client["Client<br/>Publica trabajos"]
        Freelancer["Freelancer<br/>Acepta y completa"]
        Arbiter["Arbiter<br/>Resuelve disputas"]
    end
    
    User((Usuario)) --> Client
    User --> Freelancer
    User --> Arbiter
    
    User -->|"Todos<br/>los roles"| All[Un usuario puede<br/>tener todos]
    
    style All fill:#e1f5fe,stroke:#01579b
```

**Cómo se determina el rol**:
- Client → Es el `client` en algún Job PDA
- Freelancer → Es el `freelancer` en algún Job PDA
- Arbiter → Está registrado en el pool de árbitros

---

### 3. Árbitros

```mermaid
flowchart TB
    subgraph "Arbiter Pool"
        AP["ArbiterPool PDA"] 
        Auth["authority<br/>(Admin)"]
        ArbList["arbiters[]<br/>(lista)"]
        
        AP --> Auth
        AP --> ArbList
    end
    
    subgraph "Job"
        J["Job PDA"]
        Client["client"]
        Freelancer["freelancer"]
        Arbiter["arbiter (Option)"]
        Status["status"]
        
        J --> Client
        J --> Freelancer
        J --> Arbiter
        J --> Status
    end
    
    AP -.->|"En disputa<br/>asigna"| J
    
    style AP fill:#e1f5fe,stroke:#01579b
    style J fill:#fff3e0,stroke:#e65100
```

**Flujo**:
1. Al crear job → `arbiter` = None (en blanco)
2. Si hay disputa → Se asigna automáticamente un árbitro del pool
3. O el cliente puede elegir uno específico

---

### 4. Gobernanza (Admin + Tesorero)

| Rol | Tipo | Justificación |
|-----|------|---------------|
| **Admin** | Multisig 2-of-3 | Inicializar, pausar, upgrades |
| **Tesorero** | Multisig 2-of-3 | Retirar fees acumulados |

**Por qué NO single signer**:
- Seguridad: Si perdés una key, el proyecto muere
- Confianza: Para un DeFi, múltiples firmas es estándar

---

### 5. Estructura de Cuentas On-Chain

```mermaid
graph TB
    subgraph "ESCROW PROGRAM"
        direction TB
        
        subgraph "Global Accounts"
            Config["Config<br/>admin, treasury,<br/>fee_percent, paused"]
            ArbiterPool["ArbiterPool<br/>authority, arbiters[]"]
        end
        
        subgraph "User Accounts (PDA)"
            User["User PDA<br/>wallet_principal,<br/>wallets_asociadas,<br/>active_wallet,<br/>username, bio"]
        end
        
        subgraph "Job Accounts (PDA)"
            Job["Job PDA<br/>client, freelancer,<br/>arbiter, title,<br/>description, amount,<br/>deadline, status"]
        end
        
        Config -.->|"Configura"| ArbiterPool
        User -.->|"Crea"| Job
    end
    
    style Config fill:#e8f5e8,stroke:#2e7d32
    style ArbiterPool fill:#e1f5fe,stroke:#01579b
    style User fill:#fff3e0,stroke:#e65100
    style Job fill:#f3e5f5,stroke:#6a1b9a
```

---

## Instrucciones del Programa

| Instruccion | Descripcion | Quien puede |
|-------------|-------------|-------------|
| `initialize_config` | Crear config global | Admin (multisig) |
| `create_user` | Crear perfil de usuario | Cualquiera |
| `add_wallet` | Agregar wallet secundaria | Usuario owner |
| `set_active_wallet` | Cambiar wallet activa | Usuario owner |
| `register_arbiters` | Agregar árbitros al pool | Admin |
| `create_job` | Publicar trabajo | Client |
| `accept_job` | Aceptar trabajo | Freelancer |
| `submit_work` | Entregar trabajo | Freelancer |
| `approve_work` | Aprobar trabajo | Client |
| `reject_work` | Rechazar trabajo | Client |
| `raise_dispute` | Iniciar disputa | Client o Freelancer |
| `resolve_dispute` | Resolver disputa | Arbiter |
| `cancel_job` | Cancelar trabajo | Client |
| `pause` | Pausar programa | Admin |
| `unpause` | Reanudar programa | Admin |
| `withdraw_treasury` | Retirar fees | Tesorero (multisig) |

---

## Notas Técnicas

- **Account discriminator**: 8 bytes para discriminación
- **Space calculation**: Usar `INIT_SPACE` de Anchor
- **PDA derivation**: Semillas = [b"seed", ...args]
- **CPI**: System Program para transferencias SOL

---

## Próximos Pasos

1. SDD → Especificación detallada
2. Implementar smart contract en Anchor
3. Tests de integración
4. Desplegar a devnet