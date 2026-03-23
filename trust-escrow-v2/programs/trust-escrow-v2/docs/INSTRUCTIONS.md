# 📚 Instrucciones del Smart Contract - Trust Work Escrow v2

Este documento explica cada instrucción del contrato inteligente para que cualquier persona pueda entender cómo funciona.

---

## 🏗️ Accounts (Cuentas)

### Config Account
```rust
pub struct Config {
    pub admin: Pubkey,              // Wallet del admin
    pub treasury_wallet: Pubkey,     // Wallet donde se colectan las fees
    pub treasurer: Pubkey,           // Wallet que puede retirar del treasury
    pub entry_fee_bps: u16,         // Fee de entrada en basis points (500 = 5%)
    pub exit_fee_bps: u16,          // Fee de salida en basis points
    pub dispute_stake_bps: u16,      // Stake de disputa (250 = 2.5% por parte)
    pub max_job_duration_days: u32, // Días máximos para un job
    pub auto_approve_days: u8,      // Días para auto-aprobar (7)
    pub paused: bool,                // Si el programa está pausado
    pub bump: u8,                   // Bump del PDA
}
```

### User Account
```rust
pub struct User {
    pub owner: Pubkey,              // Wallet dueña de la cuenta
    pub username: String,           // Nombre de usuario (max 32 chars)
    pub bio: String,                // Biografía (max 256 chars)
    pub skills: String,              // Habilidades separadas por coma (max 128)
    pub reputation: u8,             // Score de reputación (0-100)
    pub jobs_completed: u32,        // Jobs completados
    pub disputes_won: u32,          // Disputas ganadas
    pub disputes_lost: u32,          // Disputas perdidas
    pub is_arbiter: bool,          // Si es árbitro
    pub wallet_count: u8,           // Cantidad de wallets vinculadas
    pub wallets: Vec<u8>,           // Datos de wallets (max 5)
    pub active_wallet_index: u8,    // Índice de wallet activa
    pub bump: u8,                   // Bump del PDA
    pub created_at: i64,             // Timestamp de creación
    pub updated_at: i64,             // Timestamp de última actualización
}
```

---

## 🔧 Instrucciones de Configuración

### 1. `initialize_config`

**¿Qué hace?**
Inicializa la configuración global del programa. Solo puede ejecutarse una vez.

**¿Quién puede llamarlo?**
Solo el admin (la wallet que llama la instrucción).

**¿Por qué existe?**
El contrato necesita parámetros globales como:
- ¿Cuánto cobra de fee?
- ¿Cuál es la wallet de tesorería?
- ¿Cuánto dura un job máximo?

**Parámetros:**
```rust
pub struct InitializeConfigParams {
    pub treasury_wallet: Pubkey,        // Wallet para collectar fees
    pub entry_fee_bps: u16,             // Fee al publicar (default: 500 = 5%)
    pub exit_fee_bps: u16,              // Fee al cobrar (default: 500 = 5%)
    pub dispute_stake_bps: u16,         // Stake de disputa (default: 250 = 2.5%)
    pub max_job_duration_days: u32,    // Días máximos (default: 90)
    pub auto_approve_days: u8,         // Días para auto-aprobar (default: 7)
}
```

**Ejemplo:**
```javascript
await program.methods.initializeConfig(
    new anchor.web3.PublicKey("TreasuryWallet..."),
    500,  // 5% fee de entrada
    500,  // 5% fee de salida
    250,  // 2.5% stake de disputa por parte
    90,   // 90 días max
    7     // 7 días para auto-aprobar
).accounts({...}).rpc();
```

**Validaciones:**
- Los fees no pueden superar 1000 bps (10%)
- Solo puede ejecutarse una vez

---

### 2. `update_config`

**¿Qué hace?**
Actualiza los parámetros de configuración del programa.

**¿Quién puede llamarlo?**
Solo el admin.

**¿Por qué existe?**
Permite cambiar los fees y parámetros sin necesidad de redesplegar el contrato.

**Parámetros:**
```rust
pub struct UpdateConfigParams {
    pub treasury_wallet: Option<Pubkey>,  // Nueva wallet de treasury
    pub entry_fee_bps: Option<u16>,        // Nuevo fee de entrada
    pub exit_fee_bps: Option<u16>,         // Nuevo fee de salida
    pub dispute_stake_bps: Option<u16>,      // Nuevo stake de disputa
}
```

**Ejemplo:**
```javascript
await program.methods.updateConfig(
    null,           // No cambiar treasury
    null,           // No cambiar fee entrada
    600,            // Cambiar fee salida a 6%
    null            // No cambiar stake
).accounts({...}).rpc();
```

**Validaciones:**
- Solo el admin puede actualizar
- El programa no debe estar pausado
- Los nuevos fees no pueden superar 1000 bps

---

### 3. `pause`

**¿Qué hace?**
Pausa el programa, preventiendo que se ejecuten nuevas instrucciones (excepto `unpause`).

**¿Quién puede llamarlo?**
Solo el admin.

**¿Por qué existe?**
Para casos de emergencia:
- обнаружен bug crítico
- Necesidad de actualizar el programa
- Mantenimiento

**Ejemplo:**
```javascript
await program.methods.pause().accounts({...}).rpc();
```

**Validaciones:**
- Solo el admin puede pausar
- El programa no debe estar ya pausado

---

### 4. `unpause`

**¿Qué hace?**
Reactiva el programa permitiendo que se ejecuten instrucciones nuevamente.

**¿Quién puede llamarlo?**
Solo el admin.

**Ejemplo:**
```javascript
await program.methods.unpause().accounts({...}).rpc();
```

**Validaciones:**
- Solo el admin puede despausar
- El programa debe estar pausado

---

## 👤 Instrucciones de Usuario

### 5. `create_user`

**¿Qué hace?**
Crea un perfil de usuario en la blockchain.

**¿Quién puede llamarlo?**
Cualquier persona con una wallet.

**¿Por qué existe?**
Para crear identidades on-chain que pueden:
- Postularse a jobs
- Crear jobs como cliente
- Ser árbitros

**PDA Derivation:**
```
user_pda = PDA["user", wallet_owner]
```

**Parámetros:**
```rust
pub struct CreateUserParams {
    pub username: String,  // Nombre único (max 32 chars)
    pub bio: String,       // Descripción (max 256 chars)
    pub skills: String,    // Habilidades separadas por coma (max 128)
}
```

**Ejemplo:**
```javascript
await program.methods.createUser(
    "john_dev",                    // Username
    "Full-stack developer",       // Bio
    "rust,react,python,sql"       // Skills
).accounts({...}).rpc();
```

**Validaciones:**
- Username max 32 caracteres
- Bio max 256 caracteres
- Skills max 128 caracteres
- Solo una cuenta de usuario por wallet

---

### 6. `update_user`

**¿Qué hace?**
Actualiza la información del perfil de usuario.

**¿Quién puede llamarlo?**
Solo el owner del perfil.

**¿Por qué existe?**
Permite cambiar información sin perder el historial.

**Parámetros:**
```rust
pub struct UpdateUserParams {
    pub username: Option<String>,   // Nuevo username
    pub bio: Option<String>,        // Nueva bio
    pub skills: Option<String>,     // Nuevas skills
}
```

**Ejemplo:**
```javascript
await program.methods.updateUser(
    "john_updated",      // Nuevo username
    null,               // No cambiar bio
    "rust,anchor,js"    // Nuevas skills
).accounts({...}).rpc();
```

**Validaciones:**
- Solo el owner puede actualizar
- Campos opcionales (null = no cambiar)

---

## 💼 Instrucciones de Wallet (Multi-wallet)

### 7. `add_wallet`

**¿Qué hace?**
Agrega una wallet secundaria a la cuenta de usuario.

**¿Quién puede llamarlo?**
Solo el owner del perfil de usuario.

**¿Por qué existe?**
Permite que un usuario tenga múltiples wallets (hasta 5):
- Wallet principal (creada con el usuario)
- Wallets secundarias para diferentes propósitos
- Mejor seguridad al separar wallets

**PDA Derivation:**
```
user_pda = PDA["user", primary_wallet]
```

**Ejemplo:**
```javascript
await program.methods.addWallet()
    .accounts({
        user: userPda,
        newWallet: secondaryWallet,
        owner: primaryWallet
    }).rpc();
```

**Validaciones:**
- Máximo 5 wallets por usuario
- La wallet no debe estar ya agregada
- Solo el owner puede agregar wallets

---

### 8. `set_active_wallet`

**¿Qué hace?**
Cambia la wallet activa del usuario.

**¿Quién puede llamarlo?**
Solo el owner del perfil.

**¿Por qué existe?**
Permite cambiar entre wallets sin perder el perfil:
- Trabajar con diferentes wallets
- Cambiar wallet predeterminada para transacciones

**Parámetros:**
```rust
pub struct SetActiveWalletParams {
    pub wallet_index: u8,  // Índice de la wallet a activar (0-4)
}
```

**Ejemplo:**
```javascript
await program.methods.setActiveWallet({ walletIndex: 2 })
    .accounts({...}).rpc();
```

**Validaciones:**
- El índice debe ser menor a `wallet_count`
- Solo el owner puede cambiar la wallet activa

---

### 9. `remove_wallet`

**¿Qué hace?**
Elimina una wallet secundaria de la cuenta.

**¿Quién puede llamarlo?**
Solo el owner del perfil.

**¿Por qué existe?**
Permite:
- Remover wallets comprometidas
- Liberar espacio (máximo 5)
- Cambiar a una wallet diferente

**Parámetros:**
```rust
pub struct RemoveWalletParams {
    pub wallet_index: u8,  // Índice de la wallet a remover
}
```

**Ejemplo:**
```javascript
await program.methods.removeWallet({ walletIndex: 2 })
    .accounts({...}).rpc();
```

**Validaciones:**
- No se puede remover la wallet primaria (índice 0)
- El índice debe ser válido
- Solo el owner puede remover wallets

---

## 🔐 Modelo de Seguridad

### Validaciones Comunes

1. **Admin Checks**: Solo el admin puede pausar/despausar y actualizar config
2. **Owner Checks**: Solo el dueño puede modificar su perfil
3. **PDA Validation**: Las cuentas se derivan usando PDAs para prevent spoofing
4. **Pausable**: El programa puede pausarse en emergencias

### Errors Personalizados

```rust
pub enum EscrowError {
    #[msg("Unauthorized: Only admin can perform this action")]
    UnauthorizedAdmin,
    
    #[msg("Program is paused")]
    ProgramPaused,
    
    #[msg("Invalid fee percentage")]
    InvalidFeePercentage,
    
    #[msg("Maximum wallets reached (5)")]
    MaxWalletsReached,
    
    #[msg("Cannot remove primary wallet")]
    CannotRemovePrimaryWallet,
    
    #[msg("Invalid wallet index")]
    InvalidWalletIndex,
    
    #[msg("Wallet already added")]
    WalletAlreadyAdded,
}
```

---

## 📊 Estructura de Datos

### ¿Por qué usar PDAs?

Los Program Derived Addresses (PDAs) permiten:
1. **Determinismo**: La misma wallet siempre genera el mismo PDA
2. **Seguridad**: Solo el programa puede firmar para el PDA
3. **Sin keys privadas**: No hay seed phrase que perder

### Estructura de Wallets en User

```
wallets_data: Vec<u8>
├── Wallet 0 (34 bytes)
│   ├── pubkey (32 bytes)
│   └── is_primary (1 byte) + padding (1 byte)
├── Wallet 1 (34 bytes)
│   └── ...
└── ...
```

---

## 🚀 Próximas Instrucciones

- `create_job`: Crear un nuevo trabajo
- `publish_job`: Publicar y fondear un trabajo
- `apply_to_job`: Postularse a un trabajo
- `accept_application`: Aceptar un postulante
- `submit_work`: Entregar trabajo
- `approve_work`: Aprobar trabajo (desbloquea pago)
- `raise_dispute`: Abrir disputa

---

_Last updated: 2026-03-23_
