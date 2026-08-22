# Plan: Doble Comisión + Update Job Atómico

## TL;DR

Implementar (1) que el freelancer también pague 5% de comisión al cobrar, y (2) que el cliente pueda "editar" un job haciendo cancel+create+deposit en una sola transacción atómica de Solana (multi-instruction tx).

---

## FASE 1 — Doble comisión (5% cliente + 5% freelancer)

**Pasos** _(secuencial — cada uno depende del anterior)_

### Paso 1 — `programs/trust-escrow/src/lib.rs`

Modificar 2 instrucciones:

**`approve_work`** (líneas ~148-175):

```rust
// Antes:
let payment_amount = job.amount;
let fee_amount = job.fee_amount;
freelancer += payment_amount;
treasury  += fee_amount;

// Después:
let fee_amount = job.fee_amount;
let payment_amount = job.amount - fee_amount;   // freelancer paga su 5%
freelancer += payment_amount;
treasury   += fee_amount * 2;                   // 10% total al treasury
// ✅ cuadra: (amount - fee) + (fee*2) = amount + fee = lo que hay en el PDA
```

**`resolve_dispute`** (líneas ~237-270):

```rust
// Antes:
let freelancer_amount = (job.amount as u128 * freelancer_percent as u128 / 100) as u64;

// Después:
let fee_amount = job.fee_amount;
let net_amount = job.amount - fee_amount;       // base a repartir (sin fees)
let freelancer_amount = (net_amount as u128 * freelancer_percent as u128 / 100) as u64;
// client_amount = net_amount * (100 - freelancer_percent) / 100 → vía close = client
// treasury recibe fee_amount * 2 siempre (incluso en disputa)
```

Actualizar los `msg!()` logs en ambas instrucciones.

### Paso 2 — `escrow-core/src/lib.rs`

Actualizar `impl Display for JobInfo` para mostrar pago neto:

```rust
// Añadir después de la línea de Amount:
writeln!(f, "Neto freelancer: {} SOL ({} lamports, descontando 5%)",
    (self.amount - self.fee_amount) as f64 / 1e9,
    self.amount - self.fee_amount
)?;
```

### Paso 3 — `tests/trust-escrow.ts`

Actualizar assertions de balance:

**Test `approve_work`** (línea ~363):

```typescript
// Antes:
const expectedPayment = JOB_AMOUNT.toNumber();
const expectedFee = (JOB_AMOUNT.toNumber() * FEE_PERCENT) / 100;
expect(freelancerAfter - freelancerBefore).to.equal(expectedPayment);
expect(treasuryAfter - treasuryBefore).to.equal(expectedFee);

// Después:
const feeAmount = (JOB_AMOUNT.toNumber() * FEE_PERCENT) / 100;
const expectedPayment = JOB_AMOUNT.toNumber() - feeAmount; // freelancer recibe menos 5%
const expectedFee = feeAmount * 2; // treasury recibe 10%
expect(freelancerAfter - freelancerBefore).to.equal(expectedPayment);
expect(treasuryAfter - treasuryBefore).to.equal(expectedFee);
```

**Test `resolve_dispute` arbiter resolves (70% freelancer)** (línea ~525):

```typescript
// Antes:
const expectedFreelancer = Math.floor((JOB_AMOUNT.toNumber() * 70) / 100);
const expectedFee = (JOB_AMOUNT.toNumber() * FEE_PERCENT) / 100;

// Después:
const feeAmount = (JOB_AMOUNT.toNumber() * FEE_PERCENT) / 100;
const netAmount = JOB_AMOUNT.toNumber() - feeAmount;
const expectedFreelancer = Math.floor((netAmount * 70) / 100);
const expectedFee = feeAmount * 2;
```

Revisar si el `raise_dispute flow` tiene assertions de balance y actualizar igual.

### Paso 4 — Verificar Fase 1

```bash
cd trust-escrow && anchor build && anchor test
```

Deben pasar los 23 tests.

---

## FASE 2 — Update Job atómico (cancel + recrear en 1 tx)

### Paso 5 — `escrow-core/src/lib.rs`

Añadir helper `send_many` para múltiples instrucciones en 1 transacción atómica:

```rust
fn send_many(rpc: &RpcClient, payer: &Keypair, ixs: Vec<Instruction>) -> Result<String> {
    let bh = rpc.get_latest_blockhash()?;
    let tx = Transaction::new_signed_with_payer(&ixs, Some(&payer.pubkey()), &[payer], bh);
    let sig = rpc.send_and_confirm_transaction(&tx)?;
    Ok(sig.to_string())
}
```

Refactorizar `op_cancel`, `op_create_job`, `op_deposit` para extraer helpers privados que construyen la instrucción sin enviarla:

```rust
fn build_cancel_ix(pid: &Pubkey, payer: &Pubkey, job_id: u64) -> Instruction { ... }
fn build_create_ix(pid: &Pubkey, payer: &Pubkey, arbiter: &Pubkey, job_id: u64,
    title: &str, desc: &str, amount_lamports: u64, deadline: i64) -> Instruction { ... }
fn build_deposit_ix(pid: &Pubkey, payer: &Pubkey, job_id: u64) -> Instruction { ... }
```

Añadir función pública `op_update_job`:

```rust
pub fn op_update_job(
    rpc: &RpcClient,
    payer: &Keypair,
    old_job_id: u64,
    title: &str,
    description: &str,
    amount: f64,
    arbiter: &str,         // pubkey del árbitro original (no cambia)
    deadline: Option<i64>,
    was_funded: bool,      // si tenía fondos depositados → re-depositar
) -> Result<String> {
    let pid = program_id()?;
    let new_job_id = now_ts() as u64;   // nuevo ID = timestamp actual
    let amount_lamports = (amount * 1e9) as u64;
    let arbiter_pk = Pubkey::from_str(arbiter)?;
    let dl = deadline.unwrap_or(now_ts() + 7 * 86400);

    let cancel_ix = build_cancel_ix(&pid, &payer.pubkey(), old_job_id);
    let create_ix = build_create_ix(&pid, &payer.pubkey(), &arbiter_pk,
        new_job_id, title, description, amount_lamports, dl);

    let ixs = if was_funded {
        let deposit_ix = build_deposit_ix(&pid, &payer.pubkey(), new_job_id);
        vec![cancel_ix, create_ix, deposit_ix]
    } else {
        vec![cancel_ix, create_ix]
    };

    let sig = send_many(rpc, payer, ixs)?;
    Ok(format!(
        "✅ Job actualizado (transacción atómica).\n   ID anterior: {old_job_id} (cancelado)\n   Nuevo ID:    {new_job_id}\n   Tx: {sig}"
    ))
}
```

### Paso 6 — `tui/src/app.rs`

**Añadir a enum `Screen`:**

```rust
UpdateJobLookupForm,   // paso 1: el cliente ingresa el Job ID a modificar
UpdateJobEditForm,     // paso 2: formulario pre-llenado con datos actuales
```

**Añadir a struct `App`:**

```rust
pub cached_job_info: Option<escrow_core::JobInfo>,
pub update_old_job_id: Option<u64>,
pub update_was_funded: bool,
```

**En `build_main_menu` (Role::Client)**, añadir antes de "Cancel Job":

```rust
items.push(MenuItem {
    label: "✏️  Update Job".into(),
    screen: Screen::UpdateJobLookupForm,
});
```

**En `build_form_for_screen`:**

```rust
Screen::UpdateJobLookupForm => {
    self.setup_form(vec![
        FormField::new("Job ID", "ID del job a modificar", true),
    ]);
}
Screen::UpdateJobEditForm => {
    // Los valores se pre-llenan desde cached_job_info en submit_form del Lookup
    self.setup_form(vec![
        FormField::new("Title", "Título del trabajo", true),
        FormField::new("Amount (SOL)", "e.g. 2.5", true),
        FormField::new("Description", "Descripción (opcional)", false),
        FormField::new("Deadline (días)", "Días desde hoy (default: 7)", false),
    ]);
}
```

**En `submit_form`, caso `UpdateJobLookupForm`:**

```rust
Screen::UpdateJobLookupForm => {
    let job_id: u64 = match self.get_field(0).parse() {
        Ok(v) => v,
        Err(_) => {
            self.message = Some(("ID de job inválido".into(), MessageType::Error));
            return;
        }
    };
    let client_pubkey = self.active_pubkey.clone();
    let rpc = solana::make_rpc(self.rpc_url());

    match solana::op_show(&rpc, &client_pubkey, job_id) {
        Err(e) => {
            self.message = Some((format!("Job no encontrado: {e}"), MessageType::Error));
        }
        Ok(info) => {
            if info.status != "Created" && info.status != "Funded" {
                self.message = Some((
                    format!("No se puede modificar: el job está en estado '{}'", info.status),
                    MessageType::Error,
                ));
                return;
            }
            self.update_was_funded = info.status == "Funded";
            self.update_old_job_id = Some(job_id);
            // Pre-llenar el form de edición
            let target = Screen::UpdateJobEditForm;
            self.build_form_for_screen(&target);
            // Poblar con datos actuales
            self.form_fields[0].value = info.title.clone();
            self.form_fields[1].value = format!("{:.9}", info.amount as f64 / 1e9);
            self.form_fields[2].value = info.description.clone();
            // deadline: calcular días restantes desde ahora
            let days_left = ((info.deadline - solana::now_ts()) / 86400).max(1);
            self.form_fields[3].value = days_left.to_string();
            self.cached_job_info = Some(info);
            self.push_screen(target);
        }
    }
    return; // no ir a Result
}
```

**En `submit_form`, caso `UpdateJobEditForm`:**

```rust
Screen::UpdateJobEditForm => {
    let old_job_id = match self.update_old_job_id {
        Some(id) => id,
        None => {
            self.message = Some(("Error interno: job ID perdido".into(), MessageType::Error));
            return;
        }
    };
    let amount: f64 = match self.get_field(1).parse() {
        Ok(v) => v,
        Err(_) => {
            self.message = Some(("Monto inválido".into(), MessageType::Error));
            return;
        }
    };
    let deadline = if self.get_field(3).is_empty() {
        None
    } else {
        match self.get_field(3).parse::<i64>() {
            Ok(days) => Some(solana::now_ts() + days * 86400),
            Err(_) => {
                self.message = Some(("Deadline inválido".into(), MessageType::Error));
                return;
            }
        }
    };
    let arbiter = self.cached_job_info
        .as_ref()
        .map(|j| j.arbiter.clone())
        .unwrap_or_default();

    solana::op_update_job(
        &rpc,
        &kp,
        old_job_id,
        self.get_field(0),
        self.get_field(2),
        amount,
        &arbiter,
        deadline,
        self.update_was_funded,
    )
}
```

**En `rebuild_current_menu`** añadir los 2 nuevos casos de pantalla de formulario (no necesitan reconstruir menú, ya están cubiertos por el `_ => {}`).

### Paso 7 — `tui/src/ui.rs`

Añadir casos en el `match` del título del form:

```rust
Screen::UpdateJobLookupForm => "Update Job — Buscar",
Screen::UpdateJobEditForm   => "Update Job — Editar",
```

En el render del `UpdateJobEditForm`, añadir nota informativa antes del hint de Enter:

```rust
if app.screen == Screen::UpdateJobEditForm {
    lines.push(Line::from(Span::styled(
        "  ⚠️  El árbitro y el nuevo ID se asignan automáticamente",
        Style::default().fg(t.warning),
    )));
    lines.push(Line::from(""));
}
```

### Paso 8 — `escrow-core/src/lib.rs` — exponer `now_ts`

Cambiar `fn now_ts()` de privada a pública para que la TUI pueda calcular días restantes:

```rust
pub fn now_ts() -> i64 { ... }
```

### Paso 9 — Verificar Fase 2

```bash
cd trust-escrow/tui && cargo build
# Sin warnings, compila limpio
```

Prueba manual: crear job → depositar → update → verificar el nuevo job on-chain con show.

---

## Archivos modificados (resumen)

| Archivo                            | Fase | Cambio                                                                      |
| ---------------------------------- | ---- | --------------------------------------------------------------------------- |
| `programs/trust-escrow/src/lib.rs` | 1    | `approve_work`, `resolve_dispute`                                           |
| `escrow-core/src/lib.rs`           | 1+2  | `Display`, `now_ts` público, `send_many`, helpers privados, `op_update_job` |
| `tests/trust-escrow.ts`            | 1    | Assertions de balance en `approve_work` y `resolve_dispute` tests           |
| `tui/src/app.rs`                   | 2    | 2 screens, 3 campos App, menú client, form builders, submit_form            |
| `tui/src/ui.rs`                    | 2    | Títulos + nota informativa                                                  |

## Decisiones de diseño

- **Árbitro NO cambia** en update (seguridad: ya estaba pactado en el PDA original)
- **Job ID nuevo = timestamp** en el momento del submit (unicidad garantizada por cliente)
- **Atomicidad real**: cancel+create+deposit van en 1 sola transacción → si una falla, todo se revierte
- **Si was_funded**: 3 instrucciones atómicas; **si was_created**: 2 instrucciones atómicas
- **Si status ≥ InProgress**: error inmediato — no se puede modificar, el contrato es vinculante
- **Fase 1 primero**: validar que los 23 tests siguen pasando antes de Fase 2
