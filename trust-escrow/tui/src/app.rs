use crate::config::{Settings, Theme, WalletConfig};
use crossterm::event::{Event, KeyCode, KeyModifiers};
use escrow_core as solana;
use escrow_core::Signer;
use tracing::{error, info};

// ─── Enums ───────────────────────────────────────────────────────────────────

#[derive(Clone, Debug, PartialEq)]
pub enum Screen {
    WalletSelect,
    RoleSelect,
    MainMenu,
    // Operations
    InitForm,
    CreateJobForm,
    DepositForm,
    AcceptForm,
    SubmitForm,
    ApproveForm,
    RejectForm,
    RaiseDisputeForm,
    ResolveDisputeForm,
    CancelForm,
    ShowForm,
    JobList,
    UpdateJobLookupForm,
    UpdateJobEditForm,
    // Result
    Result,
    // Settings
    SettingsMenu,
    SettingsTheme,
    SettingsNetwork,
    SettingsWallets,
    SettingsNetworkPassword,
    ChangeMainnetPassword,
    AddWalletForm,
    WithdrawTreasuryForm,
    BalancesScreen,
    TxHistoryScreen,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Role {
    Admin,
    Client,
    Freelancer,
    Arbiter,
    Treasury,
}

impl Role {
    pub fn label(&self) -> &str {
        match self {
            Role::Admin => "Admin",
            Role::Client => "Client",
            Role::Freelancer => "Freelancer",
            Role::Arbiter => "Arbiter",
            Role::Treasury => "Treasury",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "client" => Role::Client,
            "freelancer" => Role::Freelancer,
            "arbiter" => Role::Arbiter,
            "treasury" => Role::Treasury,
            _ => Role::Admin,
        }
    }
}

#[derive(Clone, Debug)]
pub enum MessageType {
    Success,
    Error,
    Info,
}

// ─── Form Field ──────────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub struct FormField {
    pub label: String,
    pub value: String,
    pub placeholder: String,
    pub required: bool,
    pub masked: bool,
    /// Opciones para un campo tipo select (Left/Right para ciclar).
    /// Si está vacío, el campo es de texto libre.
    pub options: Vec<String>,
    /// Etiquetas de display para cada opción (si está vacío, se muestra options[i]).
    pub option_labels: Vec<String>,
    pub option_index: usize,
    /// Campo de solo lectura: muestra el valor pero no permite edición.
    pub readonly: bool,
}

impl FormField {
    pub fn new(label: &str, placeholder: &str, required: bool) -> Self {
        Self {
            label: label.into(),
            value: String::new(),
            placeholder: placeholder.into(),
            required,
            masked: false,
            options: Vec::new(),
            option_labels: Vec::new(),
            option_index: 0,
            readonly: false,
        }
    }

    /// Campo select: el valor se elige con ← → entre las opciones dadas.
    pub fn select(label: &str, options: Vec<String>, required: bool) -> Self {
        let value = options.first().cloned().unwrap_or_default();
        Self {
            label: label.into(),
            value,
            placeholder: String::new(),
            required,
            masked: false,
            options,
            option_labels: Vec::new(),
            option_index: 0,
            readonly: false,
        }
    }

    /// Campo select con etiquetas de display distintas de los valores.
    /// `options` = valores reales (e.g. pubkeys), `labels` = texto amigable para mostrar.
    #[allow(dead_code)]
    pub fn select_with_labels(
        label: &str,
        options: Vec<String>,
        labels: Vec<String>,
        required: bool,
    ) -> Self {
        let value = options.first().cloned().unwrap_or_default();
        Self {
            label: label.into(),
            value,
            placeholder: String::new(),
            required,
            masked: false,
            options,
            option_labels: labels,
            option_index: 0,
            readonly: false,
        }
    }

    /// Campo de solo lectura: muestra el valor pero no permite edición.
    pub fn readonly(label: &str, value: &str) -> Self {
        Self {
            label: label.into(),
            value: value.into(),
            placeholder: String::new(),
            required: true,
            masked: false,
            options: Vec::new(),
            option_labels: Vec::new(),
            option_index: 0,
            readonly: true,
        }
    }

    /// Retorna el label de display para la opción actual.
    pub fn current_label(&self) -> &str {
        if !self.option_labels.is_empty() {
            self.option_labels
                .get(self.option_index)
                .map(|s| s.as_str())
                .unwrap_or(&self.value)
        } else {
            &self.value
        }
    }
}

// ─── Menu Item ───────────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub struct MenuItem {
    pub label: String,
    pub screen: Screen,
}

/// Formatea un timestamp Unix como "DD/MM/YYYY HH:MM" para el TUI.
pub fn fmt_date_tui(ts: i64) -> String {
    let secs = ts as u64;
    let s = secs % 60;
    let m = (secs / 60) % 60;
    let h = (secs / 3600) % 24;
    let days = secs / 86400;
    let z = days as i64 + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let mo = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if mo <= 2 { y + 1 } else { y };
    format!("{:02}/{:02}/{} {:02}:{:02}:{:02}", d, mo, y, h, m, s)
}

// ─── App State ───────────────────────────────────────────────────────────────

pub struct App {
    pub screen: Screen,
    pub screen_stack: Vec<Screen>,
    pub role: Role,
    pub theme: Theme,
    pub settings: Settings,
    pub should_quit: bool,

    // List/menu navigation
    pub list_index: usize,
    pub menu_items: Vec<MenuItem>,

    // Form state
    pub form_fields: Vec<FormField>,
    pub form_index: usize,

    // Messages & results
    pub message: Option<(String, MessageType)>,
    pub result_text: String,

    // Wallet pubkey cache (loaded on wallet select)
    pub active_pubkey: String,

    // Update Job state
    pub cached_job_info: Option<escrow_core::JobInfo>,
    pub update_old_job_id: Option<u64>,
    pub update_was_funded: bool,
    pub pending_network_url: Option<String>,
    // Job list (Show Job screen + acción pendiente)
    pub job_list: Vec<escrow_core::JobInfo>,
    /// Si está seteado, al seleccionar un job de la lista se ejecuta esta acción
    /// en vez de mostrar los detalles. Ej: "deposit", "cancel".
    pub job_list_action: Option<String>,
    /// Saldos de wallets: (nombre, pubkey, lamports)
    pub wallet_balances: Vec<(String, String, u64)>,
    /// Historial de transacciones recientes de la wallet activa
    pub tx_history: Vec<escrow_core::TxInfo>,
}

impl App {
    pub fn new() -> Self {
        let settings = Settings::load();
        let theme = Theme::by_name(&settings.theme);
        let role = if let Some(w) = settings.wallets.get(settings.active_wallet) {
            Role::from_str(&w.role)
        } else {
            Role::Admin
        };
        let pubkey = Self::load_pubkey_for(&settings);

        let mut app = Self {
            screen: Screen::RoleSelect,
            screen_stack: Vec::new(),
            role,
            theme,
            settings,
            should_quit: false,
            list_index: 0,
            menu_items: Vec::new(),
            form_fields: Vec::new(),
            form_index: 0,
            message: None,
            result_text: String::new(),
            active_pubkey: pubkey,
            cached_job_info: None,
            update_old_job_id: None,
            update_was_funded: false,
            pending_network_url: None,
            job_list: Vec::new(),
            job_list_action: None,
            wallet_balances: Vec::new(),
            tx_history: Vec::new(),
        };
        app.build_role_menu();
        app
    }

    fn load_pubkey_for(settings: &Settings) -> String {
        if let Some(w) = settings.wallets.get(settings.active_wallet) {
            match solana::load_keypair(&w.path) {
                Ok(kp) => kp.pubkey().to_string(),
                Err(_) => "Error loading keypair".into(),
            }
        } else {
            "No wallet".into()
        }
    }

    pub fn active_wallet(&self) -> Option<&WalletConfig> {
        self.settings.wallets.get(self.settings.active_wallet)
    }

    pub fn active_wallet_name(&self) -> String {
        self.active_wallet()
            .map(|w| w.name.clone())
            .unwrap_or_else(|| "None".into())
    }

    pub fn rpc_url(&self) -> &str {
        &self.settings.rpc_url
    }

    /// Actualiza los paths de las wallets "estándar" (Admin, Client, Freelancer,
    /// Arbiter, Treasury) según la red activa, sin tocar wallets personalizadas.
    fn apply_network_defaults(&mut self) {
        use crate::config::Settings;
        let defaults = Settings::default_for_network(&self.settings.rpc_url);
        let standard_names = ["Admin", "Client", "Freelancer", "Arbiter", "Treasury"];
        for def in &defaults.wallets {
            if standard_names.contains(&def.name.as_str()) {
                if let Some(w) = self
                    .settings
                    .wallets
                    .iter_mut()
                    .find(|w| w.name == def.name)
                {
                    w.path = def.path.clone();
                    w.role = def.role.clone();
                } else {
                    self.settings.wallets.push(def.clone());
                }
            }
        }
        let _ = self.settings.save();
    }

    // ─── Screen Navigation ───────────────────────────────────────────────

    fn push_screen(&mut self, screen: Screen) {
        self.screen_stack.push(self.screen.clone());
        self.screen = screen;
        self.list_index = 0;
        self.form_index = 0;
        self.message = None;
    }

    fn pop_screen(&mut self) {
        if let Some(prev) = self.screen_stack.pop() {
            self.screen = prev;
            self.list_index = 0;
            self.message = None;
            self.rebuild_current_menu();
        }
    }

    fn go_to(&mut self, screen: Screen) {
        self.screen_stack.clear();
        self.screen = screen;
        self.list_index = 0;
        self.form_index = 0;
        self.message = None;
    }

    // ─── Menu Builders ───────────────────────────────────────────────────

    fn build_wallet_list(&mut self) {
        self.menu_items = self
            .settings
            .wallets
            .iter()
            .map(|w| MenuItem {
                label: format!("{} ({})", w.name, w.role),
                screen: Screen::RoleSelect,
            })
            .collect();
        self.menu_items.push(MenuItem {
            label: "➕ Add wallet".into(),
            screen: Screen::AddWalletForm,
        });
    }

    pub fn build_role_menu(&mut self) {
        let roles: [(&str, &str, Role); 5] = [
            ("👑 Admin", "admin", Role::Admin),
            ("💼 Client", "client", Role::Client),
            ("🔧 Freelancer", "freelancer", Role::Freelancer),
            ("⚖️ Arbiter", "arbiter", Role::Arbiter),
            ("💰 Treasury", "treasury", Role::Treasury),
        ];
        self.menu_items = roles
            .iter()
            .map(|(label, role_str, role)| {
                // Mostrar el nombre de la wallet asignada a este rol (o aviso si no hay)
                let wallet_hint = self
                    .settings
                    .wallets
                    .iter()
                    .find(|w| w.role == *role_str)
                    .map(|w| format!("  ({})", w.name))
                    .unwrap_or_else(|| "  ⚠️ sin wallet".into());
                let marker = if *role == self.role { "●" } else { "○" };
                MenuItem {
                    label: format!("{marker}  {label}{wallet_hint}"),
                    screen: Screen::MainMenu,
                }
            })
            .collect();
        // Posicionar cursor en el rol activo
        self.list_index = match self.role {
            Role::Admin => 0,
            Role::Client => 1,
            Role::Freelancer => 2,
            Role::Arbiter => 3,
            Role::Treasury => 4,
        };
    }

    pub fn build_main_menu(&mut self) {
        let mut items = Vec::new();
        match self.role {
            Role::Admin => {
                items.push(MenuItem {
                    label: "🔧 Initialize Config".into(),
                    screen: Screen::InitForm,
                });
                items.push(MenuItem {
                    label: "⏸️ Pause Program".into(),
                    screen: Screen::Result,
                });
                items.push(MenuItem {
                    label: "▶️ Unpause Program".into(),
                    screen: Screen::Result,
                });
            }
            Role::Client => {
                items.push(MenuItem {
                    label: "📝 Create Job".into(),
                    screen: Screen::CreateJobForm,
                });
                items.push(MenuItem {
                    label: "💰 Deposit Funds".into(),
                    screen: Screen::DepositForm,
                });
                items.push(MenuItem {
                    label: "✅ Approve Work".into(),
                    screen: Screen::ApproveForm,
                });
                items.push(MenuItem {
                    label: "❌ Reject Work".into(),
                    screen: Screen::RejectForm,
                });
                items.push(MenuItem {
                    label: "✏️ Update Job".into(),
                    screen: Screen::UpdateJobLookupForm,
                });
                items.push(MenuItem {
                    label: "🚫 Cancel Job".into(),
                    screen: Screen::CancelForm,
                });
            }
            Role::Freelancer => {
                items.push(MenuItem {
                    label: "🤝 Accept Job".into(),
                    screen: Screen::AcceptForm,
                });
                items.push(MenuItem {
                    label: "📦 Submit Work".into(),
                    screen: Screen::SubmitForm,
                });
                items.push(MenuItem {
                    label: "⚠️ Raise Dispute".into(),
                    screen: Screen::RaiseDisputeForm,
                });
            }
            Role::Arbiter => {
                items.push(MenuItem {
                    label: "⚖️ Resolve Dispute".into(),
                    screen: Screen::ResolveDisputeForm,
                });
            }
            Role::Treasury => {
                items.push(MenuItem {
                    label: "💰 Withdraw Funds".into(),
                    screen: Screen::WithdrawTreasuryForm,
                });
            }
        }
        // Common to all roles
        items.push(MenuItem {
            label: "🔍 Show Job".into(),
            screen: Screen::ShowForm,
        });
        items.push(MenuItem {
            label: "💰 Ver Saldos".into(),
            screen: Screen::BalancesScreen,
        });
        items.push(MenuItem {
            label: "🔄 Change Role".into(),
            screen: Screen::RoleSelect,
        });
        items.push(MenuItem {
            label: "⚙️ Settings".into(),
            screen: Screen::SettingsMenu,
        });
        self.menu_items = items;
    }

    pub fn build_settings_menu(&mut self) {
        self.menu_items = vec![
            MenuItem {
                label: "🎨 Theme".into(),
                screen: Screen::SettingsTheme,
            },
            MenuItem {
                label: "🌐 Network (RPC URL)".into(),
                screen: Screen::SettingsNetwork,
            },
            MenuItem {
                label: "👛 Manage Wallets".into(),
                screen: Screen::SettingsWallets,
            },
            MenuItem {
                label: "🔑 Change Mainnet Password".into(),
                screen: Screen::ChangeMainnetPassword,
            },
        ];
    }

    pub fn build_theme_list(&mut self) {
        self.menu_items = Theme::names()
            .iter()
            .map(|n| MenuItem {
                label: format!(
                    "{} {}",
                    if *n == self.settings.theme {
                        "●"
                    } else {
                        "○"
                    },
                    n
                ),
                screen: Screen::SettingsTheme,
            })
            .collect();
    }

    pub fn build_network_list(&mut self) {
        const NETWORKS: [(&str, &str); 3] = [
            ("🏠 Localhost", "http://127.0.0.1:8899"),
            ("🧪 Devnet", "https://api.devnet.solana.com"),
            ("🌐 Mainnet", "https://api.mainnet-beta.solana.com"),
        ];
        self.menu_items = NETWORKS
            .iter()
            .map(|(label, url)| MenuItem {
                label: format!(
                    "{} {}",
                    if *url == self.settings.rpc_url {
                        "●"
                    } else {
                        "○"
                    },
                    label
                ),
                screen: Screen::SettingsNetwork,
            })
            .collect();
    }

    fn rebuild_current_menu(&mut self) {
        match self.screen {
            Screen::WalletSelect => self.build_wallet_list(),
            Screen::RoleSelect => self.build_role_menu(),
            Screen::MainMenu => self.build_main_menu(),
            Screen::SettingsMenu => self.build_settings_menu(),
            Screen::SettingsTheme => self.build_theme_list(),
            Screen::SettingsNetwork => self.build_network_list(),
            Screen::SettingsWallets => self.build_wallet_list(),
            Screen::BalancesScreen => self.fetch_wallet_balances(),
            Screen::TxHistoryScreen => {}
            _ => {}
        }
    }

    // ─── Form Builders ───────────────────────────────────────────────────

    fn setup_form(&mut self, fields: Vec<FormField>) {
        self.form_fields = fields;
        self.form_index = 0;
    }

    fn build_form_for_screen(&mut self, screen: &Screen) {
        match screen {
            Screen::InitForm => {
                self.setup_form(vec![FormField::new(
                    "Treasury Address",
                    "Pubkey of treasury wallet",
                    true,
                )]);
            }
            Screen::CreateJobForm => {
                // Asignar árbitro al azar desde las wallets con role == "arbiter"
                let mut arbiter_pubkeys: Vec<(String, String)> = self
                    .settings
                    .wallets
                    .iter()
                    .filter(|w| w.role == "arbiter")
                    .filter_map(|w| {
                        solana::load_keypair(&w.path)
                            .ok()
                            .map(|kp| (w.name.clone(), kp.pubkey().to_string()))
                    })
                    .collect();
                let arbiter_field = if arbiter_pubkeys.is_empty() {
                    FormField::new("Arbiter Address", "Pubkey of arbiter", true)
                } else {
                    // Selección aleatoria usando timestamp como semilla simple
                    let idx = (solana::now_ts() as usize) % arbiter_pubkeys.len();
                    let (name, pubkey) = arbiter_pubkeys.remove(idx);
                    let pk = pubkey.clone();
                    let short = format!("{}...{}", &pk[..6], &pk[pk.len() - 4..]);
                    let mut f = FormField::readonly("Arbiter (auto-asignado)", &pubkey);
                    f.placeholder = format!("{} ({})", name, short);
                    f
                };
                // Job ID auto-generado desde el timestamp actual
                let auto_job_id = solana::now_ts() as u64;
                let job_id_field =
                    FormField::readonly("Job ID (auto-generado)", &auto_job_id.to_string());
                self.setup_form(vec![
                    FormField::new("Title", "Job title (max 100 chars)", true),
                    FormField::new("Amount (SOL)", "e.g. 2.5", true),
                    FormField::new("Description", "Job description (optional)", false),
                    arbiter_field,
                    job_id_field,
                    FormField::new("Deadline (days)", "Días desde hoy (default: 7)", false),
                ]);
            }
            Screen::DepositForm => {
                self.setup_form(vec![FormField::new("Job ID", "Numeric job ID", true)]);
            }
            Screen::AcceptForm => {
                self.setup_form(vec![
                    FormField::new("Job ID", "Numeric job ID", true),
                    FormField::new("Client Address", "Pubkey of the client", true),
                ]);
            }
            Screen::SubmitForm => {
                self.setup_form(vec![
                    FormField::new("Job ID", "Numeric job ID", true),
                    FormField::new("Client Address", "Pubkey of the client", true),
                    FormField::new("Notas de entrega", "¿Qué entregaste? (opcional)", false),
                ]);
            }
            Screen::ApproveForm => {
                self.setup_form(vec![
                    FormField::new("Job ID", "Numeric job ID", true),
                    FormField::new("Freelancer Address", "Pubkey of freelancer", true),
                ]);
            }
            Screen::RejectForm => {
                self.setup_form(vec![
                    FormField::new("Job ID", "Numeric job ID", true),
                    FormField::new("Reason", "Why are you rejecting?", true),
                ]);
            }
            Screen::RaiseDisputeForm => {
                self.setup_form(vec![
                    FormField::new("Job ID", "Numeric job ID", true),
                    FormField::new("Client Address", "Pubkey of the client", true),
                    FormField::new("Reason", "Why are you disputing?", true),
                ]);
            }
            Screen::ResolveDisputeForm => {
                self.setup_form(vec![
                    FormField::new("Job ID", "Numeric job ID", true),
                    FormField::new("Client Address", "Pubkey of the client", true),
                    FormField::new("Freelancer Address", "Pubkey of freelancer", true),
                    FormField::new("Freelancer %", "0-100 (% to freelancer)", true),
                    FormField::new(
                        "Notas de resolución",
                        "Explica tu decisión (requerido)",
                        true,
                    ),
                ]);
            }
            Screen::CancelForm => {
                self.setup_form(vec![FormField::new("Job ID", "Numeric job ID", true)]);
            }
            Screen::ShowForm => {
                self.setup_form(vec![
                    FormField::new("Job ID", "Numeric job ID", true),
                    FormField::new("Client Address", "Pubkey of the client", true),
                ]);
            }
            Screen::SettingsNetworkPassword => {
                let mut field =
                    FormField::new("Password", "Escribe la contraseña y presiona Enter", true);
                field.masked = true;
                self.setup_form(vec![field]);
            }
            Screen::ChangeMainnetPassword => {
                let mut current = FormField::new("Contraseña actual", "Contraseña actual", true);
                current.masked = true;
                let mut new1 = FormField::new("Nueva contraseña", "Mínimo 4 caracteres", true);
                new1.masked = true;
                let mut new2 = FormField::new(
                    "Confirmar nueva contraseña",
                    "Repite la nueva contraseña",
                    true,
                );
                new2.masked = true;
                self.setup_form(vec![current, new1, new2]);
            }
            Screen::AddWalletForm => {
                // Escanear ~/.config/solana/ para ofrecer paths conocidos
                let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
                let sol_dir = format!("{home}/.config/solana");
                let mut keypair_paths: Vec<String> = std::fs::read_dir(&sol_dir)
                    .ok()
                    .into_iter()
                    .flat_map(|entries| {
                        entries
                            .filter_map(|e| e.ok())
                            .filter(|e| e.path().extension().map(|x| x == "json").unwrap_or(false))
                            .map(|e| e.path().to_string_lossy().to_string())
                    })
                    .collect();
                keypair_paths.sort();

                let path_field = if keypair_paths.is_empty() {
                    FormField::new("Keypair Path", &format!("{sol_dir}/id.json"), true)
                } else {
                    FormField::select("Keypair Path", keypair_paths, true)
                };

                self.setup_form(vec![
                    FormField::new("Wallet Name", "e.g. My Client Wallet", true),
                    path_field,
                    FormField::select(
                        "Role",
                        vec![
                            "admin".into(),
                            "client".into(),
                            "freelancer".into(),
                            "arbiter".into(),
                            "treasury".into(),
                        ],
                        true,
                    ),
                ]);
                // Pre-seleccionar el rol basándose en el primer path de la lista
                self.sync_role_to_path();
            }
            Screen::UpdateJobLookupForm => {
                self.setup_form(vec![FormField::new(
                    "Job ID",
                    "ID numérico del job a editar",
                    true,
                )]);
            }
            Screen::WithdrawTreasuryForm => {
                self.setup_form(vec![
                    FormField::new("Amount (SOL)", "e.g. 1.5", true),
                    FormField::new(
                        "Destination",
                        "Pubkey destino (Enter para usar esta wallet)",
                        false,
                    ),
                ]);
            }
            _ => {}
        }
    }

    // ─── Form Submission ─────────────────────────────────────────────────

    fn get_field(&self, idx: usize) -> &str {
        self.form_fields
            .get(idx)
            .map(|f| f.value.as_str())
            .unwrap_or("")
    }

    fn submit_form(&mut self) {
        // Check required fields
        for f in &self.form_fields {
            if f.required && f.value.trim().is_empty() {
                self.message = Some((format!("'{}' is required", f.label), MessageType::Error));
                return;
            }
        }

        let wallet_path = match self.active_wallet() {
            Some(w) => w.path.clone(),
            None => {
                self.message = Some(("No wallet selected".into(), MessageType::Error));
                return;
            }
        };

        let rpc = solana::make_rpc(self.rpc_url());
        let kp = match solana::load_keypair(&wallet_path) {
            Ok(kp) => kp,
            Err(e) => {
                self.message = Some((format!("Keypair error: {e}"), MessageType::Error));
                return;
            }
        };

        let result = match self.screen {
            Screen::InitForm => solana::op_init(&rpc, &kp, self.get_field(0)),

            Screen::CreateJobForm => {
                let amount: f64 = match self.get_field(1).parse() {
                    Ok(v) => v,
                    Err(_) => {
                        self.message =
                            Some(("Invalid amount (use e.g. 2.5)".into(), MessageType::Error));
                        return;
                    }
                };
                // Job ID viene del campo readonly (auto-generado)
                let job_id: u64 = match self.get_field(4).parse() {
                    Ok(v) => v,
                    Err(_) => {
                        self.message = Some(("Error generando Job ID".into(), MessageType::Error));
                        return;
                    }
                };
                let deadline = if self.get_field(5).is_empty() {
                    None
                } else {
                    match self.get_field(5).parse::<i64>() {
                        Ok(days) => {
                            let now = std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap()
                                .as_secs() as i64;
                            Some(now + days * 86400)
                        }
                        Err(_) => {
                            self.message = Some((
                                "Deadline inválido (usa días e.g. 7)".into(),
                                MessageType::Error,
                            ));
                            return;
                        }
                    }
                };
                solana::op_create_job(
                    &rpc,
                    &kp,
                    self.get_field(0),
                    self.get_field(2),
                    amount,
                    self.get_field(3),
                    job_id,
                    deadline,
                )
            }

            Screen::DepositForm => {
                let job_id: u64 = match self.get_field(0).parse() {
                    Ok(v) => v,
                    Err(_) => {
                        self.message = Some(("Invalid job ID".into(), MessageType::Error));
                        return;
                    }
                };
                solana::op_deposit(&rpc, &kp, job_id)
            }

            Screen::AcceptForm => {
                let job_id: u64 = match self.get_field(0).parse() {
                    Ok(v) => v,
                    Err(_) => {
                        self.message = Some(("Invalid job ID".into(), MessageType::Error));
                        return;
                    }
                };
                solana::op_accept(&rpc, &kp, job_id, self.get_field(1))
            }

            Screen::SubmitForm => {
                let job_id: u64 = match self.get_field(0).parse() {
                    Ok(v) => v,
                    Err(_) => {
                        self.message = Some(("Invalid job ID".into(), MessageType::Error));
                        return;
                    }
                };
                solana::op_submit(&rpc, &kp, job_id, self.get_field(1), self.get_field(2))
            }

            Screen::ApproveForm => {
                let job_id: u64 = match self.get_field(0).parse() {
                    Ok(v) => v,
                    Err(_) => {
                        self.message = Some(("Invalid job ID".into(), MessageType::Error));
                        return;
                    }
                };
                solana::op_approve(&rpc, &kp, job_id, self.get_field(1))
            }

            Screen::RejectForm => {
                let job_id: u64 = match self.get_field(0).parse() {
                    Ok(v) => v,
                    Err(_) => {
                        self.message = Some(("Invalid job ID".into(), MessageType::Error));
                        return;
                    }
                };
                solana::op_reject(&rpc, &kp, job_id, self.get_field(1))
            }

            Screen::RaiseDisputeForm => {
                let job_id: u64 = match self.get_field(0).parse() {
                    Ok(v) => v,
                    Err(_) => {
                        self.message = Some(("Invalid job ID".into(), MessageType::Error));
                        return;
                    }
                };
                solana::op_raise_dispute(&rpc, &kp, job_id, self.get_field(1), self.get_field(2))
            }

            Screen::ResolveDisputeForm => {
                let job_id: u64 = match self.get_field(0).parse() {
                    Ok(v) => v,
                    Err(_) => {
                        self.message = Some(("Invalid job ID".into(), MessageType::Error));
                        return;
                    }
                };
                let pct: u8 = match self.get_field(3).parse() {
                    Ok(v) if v <= 100 => v,
                    _ => {
                        self.message =
                            Some(("Invalid percentage (0-100)".into(), MessageType::Error));
                        return;
                    }
                };
                solana::op_resolve_dispute(
                    &rpc,
                    &kp,
                    job_id,
                    self.get_field(1),
                    self.get_field(2),
                    pct,
                    self.get_field(4),
                )
            }

            Screen::CancelForm => {
                let job_id: u64 = match self.get_field(0).parse() {
                    Ok(v) => v,
                    Err(_) => {
                        self.message = Some(("Invalid job ID".into(), MessageType::Error));
                        return;
                    }
                };
                solana::op_cancel(&rpc, &kp, job_id)
            }

            Screen::ShowForm => {
                let job_id: u64 = match self.get_field(0).parse() {
                    Ok(v) => v,
                    Err(_) => {
                        self.message = Some(("Invalid job ID".into(), MessageType::Error));
                        return;
                    }
                };
                solana::op_show(&rpc, self.get_field(1), job_id).map(|info| info.to_string())
            }

            Screen::ChangeMainnetPassword => {
                let current = self.get_field(0).to_string();
                let new1 = self.get_field(1).to_string();
                let new2 = self.get_field(2).to_string();
                if current != self.settings.mainnet_password {
                    self.message =
                        Some(("Contraseña actual incorrecta".into(), MessageType::Error));
                    return;
                }
                if new1.len() < 4 {
                    self.message = Some((
                        "La nueva contraseña debe tener al menos 4 caracteres".into(),
                        MessageType::Error,
                    ));
                    return;
                }
                if new1 != new2 {
                    self.message =
                        Some(("Las contraseñas no coinciden".into(), MessageType::Error));
                    return;
                }
                self.settings.mainnet_password = new1;
                let _ = self.settings.save();
                self.message = Some((
                    "✅ Contraseña de Mainnet actualizada!".into(),
                    MessageType::Success,
                ));
                self.pop_screen();
                return;
            }

            Screen::SettingsNetworkPassword => {
                if self.get_field(0) != self.settings.mainnet_password {
                    self.message = Some(("Contraseña incorrecta".into(), MessageType::Error));
                    return;
                }
                if let Some(url) = self.pending_network_url.take() {
                    self.settings.rpc_url = url;
                    self.apply_network_defaults();
                }
                self.message = Some(("⚠️  Mainnet activado!".into(), MessageType::Info));
                self.pop_screen();
                self.build_network_list();
                return;
            }

            Screen::AddWalletForm => {
                let name = self.get_field(0).to_string();
                let path = self.get_field(1).to_string();
                let role = self.get_field(2).to_string().to_lowercase();
                // Validate keypair exists
                if solana::load_keypair(&path).is_err() {
                    self.message = Some((
                        format!("Cannot load keypair at: {path}"),
                        MessageType::Error,
                    ));
                    return;
                }
                self.settings
                    .wallets
                    .push(WalletConfig { name, path, role });
                let _ = self.settings.save();
                self.message = Some(("Wallet added!".into(), MessageType::Success));
                self.pop_screen();
                self.rebuild_current_menu();
                return;
            }

            Screen::WithdrawTreasuryForm => {
                let amount: f64 = match self.get_field(0).parse() {
                    Ok(v) => v,
                    Err(_) => {
                        self.message =
                            Some(("Monto inválido (usa e.g. 1.5)".into(), MessageType::Error));
                        return;
                    }
                };
                let dest_raw = self.get_field(1).to_string();
                let dest = if dest_raw.trim().is_empty() {
                    kp.pubkey().to_string()
                } else {
                    dest_raw
                };
                match solana::op_withdraw_treasury(&rpc, &kp, amount, &dest) {
                    Ok(msg) => {
                        self.result_text = msg;
                        self.screen = Screen::Result;
                        self.message = None;
                    }
                    Err(e) => {
                        self.message = Some((format!("Error: {e}"), MessageType::Error));
                    }
                }
                return;
            }

            Screen::UpdateJobLookupForm => {
                let job_id: u64 = match self.get_field(0).parse() {
                    Ok(v) => v,
                    Err(_) => {
                        self.message = Some(("Job ID inválido".into(), MessageType::Error));
                        return;
                    }
                };
                let client_pk = kp.pubkey().to_string();
                let info = match solana::op_show(&rpc, &client_pk, job_id) {
                    Ok(i) => i,
                    Err(e) => {
                        self.result_text = format!("❌ Error:\n{e}");
                        self.screen = Screen::Result;
                        self.message = None;
                        return;
                    }
                };
                if info.status != "Created" && info.status != "Funded" {
                    self.message = Some((
                        format!(
                            "Estado inválido: {} (debe ser Created o Funded)",
                            info.status
                        ),
                        MessageType::Error,
                    ));
                    return;
                }
                let was_funded = info.status == "Funded";
                let title = info.title.clone();
                let amount_str = format!("{:.4}", info.amount as f64 / 1e9);
                let description = info.description.clone();
                self.update_was_funded = was_funded;
                self.update_old_job_id = Some(job_id);
                self.cached_job_info = Some(info);
                let mut fields = vec![
                    FormField::new("Title", "Título del job", true),
                    FormField::new("Amount (SOL)", "e.g. 2.5", true),
                    FormField::new("Description", "Descripción del job (opcional)", false),
                    FormField::new("Deadline (días)", "Días desde ahora (default: 7)", false),
                ];
                fields[0].value = title;
                fields[1].value = amount_str;
                fields[2].value = description;
                self.form_fields = fields;
                self.screen_stack.push(self.screen.clone());
                self.screen = Screen::UpdateJobEditForm;
                self.form_index = 0;
                self.message = None;
                return;
            }

            Screen::UpdateJobEditForm => {
                let amount: f64 = match self.get_field(1).parse() {
                    Ok(v) => v,
                    Err(_) => {
                        self.message =
                            Some(("Monto inválido (usa e.g. 2.5)".into(), MessageType::Error));
                        return;
                    }
                };
                let old_job_id = match self.update_old_job_id {
                    Some(id) => id,
                    None => {
                        self.message = Some(("No hay Job ID en caché".into(), MessageType::Error));
                        return;
                    }
                };
                let deadline = if self.get_field(3).is_empty() {
                    None
                } else {
                    match self.get_field(3).parse::<i64>() {
                        Ok(days) => Some(days),
                        Err(_) => {
                            self.message = Some((
                                "Deadline inválido (usa días e.g. 7)".into(),
                                MessageType::Error,
                            ));
                            return;
                        }
                    }
                };
                let arbiter = self
                    .cached_job_info
                    .as_ref()
                    .map(|i| i.arbiter.clone())
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

            _ => {
                self.message = Some(("Unknown operation".into(), MessageType::Error));
                return;
            }
        };

        match result {
            Ok(text) => {
                info!(screen = ?self.screen, "Operación completada");
                self.result_text = text;
                self.screen = Screen::Result;
                self.message = None;
            }
            Err(e) => {
                error!(screen = ?self.screen, error = %e, "Error en operación Solana");
                self.result_text = format!("❌ Error:\n{e}");
                self.screen = Screen::Result;
                self.message = None;
            }
        }
    }

    // ─── Event Handling ──────────────────────────────────────────────────

    pub fn handle_event(&mut self, event: Event) {
        if let Event::Key(key) = event {
            // Global: Ctrl+C quits
            if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
                self.should_quit = true;
                return;
            }

            match &self.screen {
                Screen::WalletSelect | Screen::SettingsWallets => self.handle_list_event(key.code),
                Screen::RoleSelect => self.handle_role_event(key.code),
                Screen::MainMenu => self.handle_menu_event(key.code),
                Screen::SettingsMenu => self.handle_settings_menu_event(key.code),
                Screen::SettingsTheme => self.handle_theme_event(key.code),
                Screen::SettingsNetwork => self.handle_network_event(key.code),
                Screen::Result => self.handle_result_event(key.code),
                Screen::JobList => self.handle_job_list_event(key.code),
                Screen::BalancesScreen => self.handle_balances_event(key.code),
                Screen::TxHistoryScreen => self.handle_tx_history_event(key.code),
                // All form screens
                _ => self.handle_form_event(key.code),
            }
        }
    }

    fn handle_list_event(&mut self, code: KeyCode) {
        let len = self.menu_items.len();
        match code {
            KeyCode::Up | KeyCode::Char('k') => {
                if self.list_index > 0 {
                    self.list_index -= 1
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if len > 0 && self.list_index < len - 1 {
                    self.list_index += 1
                }
            }
            KeyCode::Enter => {
                if len == 0 {
                    return;
                }
                let idx = self.list_index;

                if self.screen == Screen::WalletSelect || self.screen == Screen::SettingsWallets {
                    // Last item = "Add wallet"
                    if idx == self.settings.wallets.len() {
                        let target = Screen::AddWalletForm;
                        self.build_form_for_screen(&target);
                        self.push_screen(target);
                    } else {
                        self.settings.active_wallet = idx;
                        let _ = self.settings.save();
                        self.active_pubkey = Self::load_pubkey_for(&self.settings);
                        if let Some(w) = self.settings.wallets.get(idx) {
                            self.role = Role::from_str(&w.role);
                        }
                        if self.screen == Screen::SettingsWallets {
                            self.pop_screen();
                        } else {
                            self.build_role_menu();
                            self.push_screen(Screen::RoleSelect);
                        }
                    }
                }
            }
            KeyCode::Char('d') => {
                // Delete wallet (not the last "Add" item, not if only 1 wallet)
                if (self.screen == Screen::WalletSelect || self.screen == Screen::SettingsWallets)
                    && self.list_index < self.settings.wallets.len()
                    && self.settings.wallets.len() > 1
                {
                    self.settings.wallets.remove(self.list_index);
                    if self.settings.active_wallet >= self.settings.wallets.len() {
                        self.settings.active_wallet = 0;
                    }
                    let _ = self.settings.save();
                    self.active_pubkey = Self::load_pubkey_for(&self.settings);
                    self.build_wallet_list();
                    if self.list_index > 0 {
                        self.list_index -= 1;
                    }
                }
            }
            KeyCode::Esc | KeyCode::Char('q') => {
                if self.screen == Screen::SettingsWallets {
                    self.pop_screen();
                } else if !self.screen_stack.is_empty() {
                    // Hay historial → retroceder
                    self.pop_screen();
                } else {
                    // WalletSelect sin historial = salida
                    self.should_quit = true;
                }
            }
            _ => {}
        }
    }

    fn handle_role_event(&mut self, code: KeyCode) {
        let len = self.menu_items.len();
        match code {
            KeyCode::Up | KeyCode::Char('k') => {
                if self.list_index > 0 {
                    self.list_index -= 1
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if len > 0 && self.list_index < len - 1 {
                    self.list_index += 1
                }
            }
            KeyCode::Enter => {
                let selected_role = match self.list_index {
                    0 => Role::Admin,
                    1 => Role::Client,
                    2 => Role::Freelancer,
                    3 => Role::Arbiter,
                    4 => Role::Treasury,
                    _ => Role::Admin,
                };
                let role_str = selected_role.label().to_lowercase();
                // Buscar la primera wallet que tenga este rol
                let wallet_idx = self
                    .settings
                    .wallets
                    .iter()
                    .position(|w| w.role == role_str);
                match wallet_idx {
                    Some(idx) => {
                        self.role = selected_role;
                        self.settings.active_wallet = idx;
                        let _ = self.settings.save();
                        self.active_pubkey = Self::load_pubkey_for(&self.settings);
                        self.message = None;
                        self.build_main_menu();
                        self.go_to(Screen::MainMenu);
                    }
                    None => {
                        self.message = Some((
                            format!(
                                "No hay wallet para '{}'. Ve a ⚙️ Settings → Manage Wallets.",
                                selected_role.label()
                            ),
                            MessageType::Error,
                        ));
                    }
                }
            }
            KeyCode::Esc => {
                if self.screen_stack.is_empty() {
                    self.should_quit = true;
                } else {
                    self.pop_screen();
                }
            }
            _ => {}
        }
    }

    fn handle_menu_event(&mut self, code: KeyCode) {
        let len = self.menu_items.len();
        match code {
            KeyCode::Up | KeyCode::Char('k') => {
                if self.list_index > 0 {
                    self.list_index -= 1
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if len > 0 && self.list_index < len - 1 {
                    self.list_index += 1
                }
            }
            KeyCode::Enter => {
                if len == 0 {
                    return;
                }
                // Pause/Unpause are direct actions
                let label = self.menu_items[self.list_index].label.clone();
                if label.contains("Pause Program") {
                    self.execute_direct_action("pause");
                    return;
                }
                if label.contains("Unpause Program") {
                    self.execute_direct_action("unpause");
                    return;
                }
                let target = self.menu_items[self.list_index].screen.clone();
                match target {
                    Screen::RoleSelect => {
                        self.build_role_menu();
                        self.push_screen(Screen::RoleSelect);
                    }
                    Screen::WalletSelect => {
                        self.build_wallet_list();
                        self.go_to(Screen::WalletSelect);
                    }
                    Screen::ShowForm => {
                        // Navegar a ShowForm lanza la búsqueda de jobs del cliente
                        self.fetch_and_show_jobs();
                    }
                    Screen::DepositForm => {
                        // Mostrar lista de jobs en Created para escoger cuál depositar
                        self.fetch_jobs_for_action("deposit", &["Created"]);
                    }
                    Screen::CancelForm => {
                        // Mostrar lista de jobs en Created/Funded para cancelar
                        self.fetch_jobs_for_action("cancel", &["Created", "Funded"]);
                    }
                    Screen::ApproveForm => {
                        self.fetch_jobs_for_action("approve", &["Submitted"]);
                    }
                    Screen::RejectForm => {
                        self.fetch_jobs_for_action("reject", &["Submitted"]);
                    }
                    Screen::UpdateJobLookupForm => {
                        self.fetch_jobs_for_action("update", &["Created", "Funded"]);
                    }
                    Screen::AcceptForm => {
                        // Freelancer: ver todos los jobs en Funded de cualquier cliente
                        self.fetch_jobs_for_action("accept", &["Funded"]);
                    }
                    Screen::SubmitForm => {
                        self.fetch_jobs_for_action("submit", &["InProgress"]);
                    }
                    Screen::RaiseDisputeForm => {
                        self.fetch_jobs_for_action("raise_dispute", &["Submitted", "InProgress"]);
                    }
                    Screen::ResolveDisputeForm => {
                        self.fetch_jobs_for_action("resolve", &["Disputed"]);
                    }
                    Screen::BalancesScreen => {
                        self.fetch_wallet_balances();
                    }
                    Screen::SettingsMenu => {
                        self.build_settings_menu();
                        self.push_screen(Screen::SettingsMenu);
                    }
                    screen => {
                        self.build_form_for_screen(&screen);
                        self.push_screen(screen);
                    }
                }
            }
            KeyCode::Esc => {
                // ESC en MainMenu: retroceder a RoleSelect
                self.build_role_menu();
                self.go_to(Screen::RoleSelect);
            }
            KeyCode::Char('q') => {
                self.should_quit = true;
            }
            _ => {}
        }
    }

    fn handle_settings_menu_event(&mut self, code: KeyCode) {
        let len = self.menu_items.len();
        match code {
            KeyCode::Up | KeyCode::Char('k') => {
                if self.list_index > 0 {
                    self.list_index -= 1
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if len > 0 && self.list_index < len - 1 {
                    self.list_index += 1
                }
            }
            KeyCode::Enter => {
                if len == 0 {
                    return;
                }
                let target = self.menu_items[self.list_index].screen.clone();
                match target {
                    Screen::SettingsTheme => {
                        self.build_theme_list();
                        self.push_screen(Screen::SettingsTheme);
                    }
                    Screen::SettingsNetwork => {
                        self.build_network_list();
                        self.push_screen(Screen::SettingsNetwork);
                    }
                    Screen::SettingsWallets => {
                        self.build_wallet_list();
                        self.push_screen(Screen::SettingsWallets);
                    }
                    Screen::ChangeMainnetPassword => {
                        self.build_form_for_screen(&Screen::ChangeMainnetPassword);
                        self.push_screen(Screen::ChangeMainnetPassword);
                    }
                    _ => {}
                }
            }
            KeyCode::Esc => self.pop_screen(),
            _ => {}
        }
    }

    fn handle_theme_event(&mut self, code: KeyCode) {
        let names = Theme::names();
        let len = names.len();
        match code {
            KeyCode::Up | KeyCode::Char('k') => {
                if self.list_index > 0 {
                    self.list_index -= 1
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if len > 0 && self.list_index < len - 1 {
                    self.list_index += 1
                }
            }
            KeyCode::Enter => {
                if self.list_index < len {
                    let name = names[self.list_index];
                    self.settings.theme = name.to_string();
                    self.theme = Theme::by_name(name);
                    let _ = self.settings.save();
                    self.build_theme_list();
                    self.message =
                        Some((format!("Theme changed to '{name}'"), MessageType::Success));
                }
            }
            KeyCode::Esc => self.pop_screen(),
            _ => {}
        }
    }

    fn handle_network_event(&mut self, code: KeyCode) {
        const NETWORKS: [(&str, &str); 3] = [
            ("🏠 Localhost", "http://127.0.0.1:8899"),
            ("🧪 Devnet", "https://api.devnet.solana.com"),
            ("🌐 Mainnet", "https://api.mainnet-beta.solana.com"),
        ];
        let len = NETWORKS.len();
        match code {
            KeyCode::Up | KeyCode::Char('k') => {
                if self.list_index > 0 {
                    self.list_index -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if len > 0 && self.list_index < len - 1 {
                    self.list_index += 1;
                }
            }
            KeyCode::Enter => {
                if self.list_index < len {
                    let (_, url) = NETWORKS[self.list_index];
                    if url.contains("mainnet") {
                        self.pending_network_url = Some(url.to_string());
                        self.build_form_for_screen(&Screen::SettingsNetworkPassword);
                        self.push_screen(Screen::SettingsNetworkPassword);
                    } else {
                        self.settings.rpc_url = url.to_string();
                        self.apply_network_defaults();
                        self.build_network_list();
                        self.message = Some(("Red actualizada!".into(), MessageType::Success));
                    }
                }
            }
            KeyCode::Esc => self.pop_screen(),
            _ => {}
        }
    }

    fn handle_result_event(&mut self, code: KeyCode) {
        match code {
            KeyCode::Enter | KeyCode::Esc | KeyCode::Char('q') => {
                self.build_main_menu();
                self.go_to(Screen::MainMenu);
            }
            _ => {}
        }
    }

    /// Carga el saldo SOL de la wallet del rol activo y navega a BalancesScreen.
    pub fn fetch_wallet_balances(&mut self) {
        let rpc = solana::make_rpc(self.rpc_url());
        let pubkey = self.active_pubkey.clone();
        let role_name = self.role.label().to_string();
        let lamports = solana::op_get_balance(&rpc, &pubkey).unwrap_or(0);
        self.wallet_balances = vec![(role_name, pubkey, lamports)];
        if self.screen != Screen::BalancesScreen {
            self.push_screen(Screen::BalancesScreen);
        }
    }

    fn handle_balances_event(&mut self, code: KeyCode) {
        match code {
            KeyCode::Char('r') => {
                self.fetch_wallet_balances();
            }
            KeyCode::Char('h') => {
                self.fetch_tx_history();
            }
            KeyCode::Char('f') => {
                // Fondear wallet (airdrop) — solo devnet / localhost
                let url = self.rpc_url().to_string();
                if url.contains("mainnet") {
                    self.message = Some((
                        "Airdrop no disponible en mainnet".into(),
                        MessageType::Error,
                    ));
                    return;
                }
                let pubkey = self.active_pubkey.clone();
                let rpc = solana::make_rpc(&url);
                match solana::op_airdrop(&rpc, &pubkey, 1.0) {
                    Ok(msg) => {
                        self.message = Some((msg, MessageType::Success));
                        self.fetch_wallet_balances();
                    }
                    Err(e) => {
                        self.message = Some((format!("Error en airdrop: {e}"), MessageType::Error));
                    }
                }
            }
            KeyCode::Esc | KeyCode::Char('q') => {
                self.wallet_balances.clear();
                self.pop_screen();
            }
            _ => {}
        }
    }

    fn fetch_tx_history(&mut self) {
        let rpc = solana::make_rpc(self.rpc_url());
        let pubkey = self.active_pubkey.clone();
        match solana::op_get_recent_txs(&rpc, &pubkey, 10) {
            Ok(txs) => {
                self.tx_history = txs;
                self.push_screen(Screen::TxHistoryScreen);
            }
            Err(e) => {
                self.message = Some((format!("Error cargando historial: {e}"), MessageType::Error));
            }
        }
    }

    fn handle_tx_history_event(&mut self, code: KeyCode) {
        match code {
            KeyCode::Char('r') => {
                self.tx_history.clear();
                self.pop_screen();
                self.fetch_tx_history();
            }
            KeyCode::Esc | KeyCode::Char('q') => {
                self.tx_history.clear();
                self.pop_screen();
            }
            _ => {}
        }
    }

    /// Busca todos los jobs del cliente activo y navega a JobList.
    fn fetch_and_show_jobs(&mut self) {
        let rpc = solana::make_rpc(self.rpc_url());
        info!(pubkey = %self.active_pubkey, "Cargando lista de jobs");
        match solana::op_list_jobs(&rpc, &self.active_pubkey) {
            Ok(jobs) => {
                info!(count = jobs.len(), "Jobs cargados en TUI");
                self.job_list = jobs;
                self.job_list_action = None;
                self.screen_stack.push(self.screen.clone());
                self.screen = Screen::JobList;
                self.list_index = 0;
                self.message = None;
            }
            Err(e) => {
                error!(pubkey = %self.active_pubkey, error = %e, "Error buscando jobs");
                self.message = Some((format!("Error buscando jobs: {e}"), MessageType::Error));
            }
        }
    }

    /// Busca jobs filtrados por rol y estado, y navega a JobList con la acción seteada.
    fn fetch_jobs_for_action(&mut self, action: &str, statuses: &[&str]) {
        let rpc = solana::make_rpc(self.rpc_url());
        let pubkey = self.active_pubkey.clone();
        info!(pubkey = %pubkey, action = action, "Cargando jobs para acción");

        let result = match action {
            // Freelancer busca todos los jobs en estado Funded para aceptar
            "accept" => solana::op_list_all_jobs(&rpc),
            // Freelancer busca sus propios jobs asignados
            "submit" | "raise_dispute" => solana::op_list_jobs_as_freelancer(&rpc, &pubkey),
            // Árbitro busca los jobs donde es árbitro
            "resolve" => solana::op_list_jobs_as_arbiter(&rpc, &pubkey),
            // Cliente: deposit, cancel, approve, reject, update
            _ => solana::op_list_jobs(&rpc, &pubkey),
        };

        match result {
            Ok(mut jobs) => {
                // Filtrar por estados relevantes para la acción
                if !statuses.is_empty() {
                    jobs.retain(|j| statuses.contains(&j.status.as_str()));
                }
                info!(
                    count = jobs.len(),
                    action = action,
                    "Jobs filtrados para acción"
                );
                self.job_list = jobs;
                self.job_list_action = Some(action.to_string());
                self.screen_stack.push(self.screen.clone());
                self.screen = Screen::JobList;
                self.list_index = 0;
                self.message = None;
            }
            Err(e) => {
                error!(pubkey = %pubkey, action = action, error = %e, "Error buscando jobs para acción");
                self.message = Some((format!("Error buscando jobs: {e}"), MessageType::Error));
            }
        }
    }

    /// Ejecuta una acción directa (sin formulario) sobre el job seleccionado.
    /// Usado para deposit y cancel donde no hay campos adicionales.
    fn execute_job_action_direct(&mut self, action: &str, job: &escrow_core::JobInfo) {
        if job.job_id == 0 {
            // No pudimos recuperar el job_id — fallback al formulario
            self.job_list_action = None;
            let target = match action {
                "deposit" => Screen::DepositForm,
                "cancel" => Screen::CancelForm,
                _ => return,
            };
            self.build_form_for_screen(&target);
            self.push_screen(target);
            return;
        }

        let wallet_path = match self.active_wallet() {
            Some(w) => w.path.clone(),
            None => {
                self.message = Some(("No wallet selected".into(), MessageType::Error));
                return;
            }
        };
        let rpc = solana::make_rpc(self.rpc_url());
        let kp = match solana::load_keypair(&wallet_path) {
            Ok(kp) => kp,
            Err(e) => {
                self.message = Some((format!("Keypair error: {e}"), MessageType::Error));
                return;
            }
        };

        let result = match action {
            "deposit" => solana::op_deposit(&rpc, &kp, job.job_id),
            "cancel" => solana::op_cancel(&rpc, &kp, job.job_id),
            _ => return,
        };

        self.job_list_action = None;
        match result {
            Ok(msg) => {
                info!(
                    action = action,
                    job_id = job.job_id,
                    "Acción directa exitosa"
                );
                self.result_text = msg;
                self.push_screen(Screen::Result);
            }
            Err(e) => {
                error!(action = action, job_id = job.job_id, error = %e, "Error en acción directa");
                self.message = Some((format!("Error: {e}"), MessageType::Error));
            }
        }
    }

    fn handle_job_list_event(&mut self, code: KeyCode) {
        let len = self.job_list.len();
        match code {
            KeyCode::Up | KeyCode::Char('k') => {
                if self.list_index > 0 {
                    self.list_index -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if len > 0 && self.list_index < len - 1 {
                    self.list_index += 1;
                }
            }
            KeyCode::Enter => {
                if let Some(job) = self.job_list.get(self.list_index).cloned() {
                    match self.job_list_action.clone().as_deref() {
                        None => {
                            // Modo visualización: mostrar detalles
                            self.result_text = job.to_string();
                            self.push_screen(Screen::Result);
                        }
                        Some("deposit") => {
                            self.execute_job_action_direct("deposit", &job);
                        }
                        Some("cancel") => {
                            self.execute_job_action_direct("cancel", &job);
                        }
                        Some("approve") => {
                            // Prellenar ApproveForm: Job ID (readonly) + Freelancer (readonly)
                            let fl = job.freelancer.clone().unwrap_or_default();
                            self.setup_form(vec![
                                FormField::readonly("Job ID", &job.job_id.to_string()),
                                FormField::readonly("Freelancer Address", &fl),
                            ]);
                            self.form_index = 0;
                            self.job_list_action = None;
                            self.push_screen(Screen::ApproveForm);
                        }
                        Some("reject") => {
                            // Prellenar RejectForm: Job ID (readonly) + Reason (editable)
                            self.setup_form(vec![
                                FormField::readonly("Job ID", &job.job_id.to_string()),
                                FormField::new("Reason", "¿Por qué rechazas el trabajo?", true),
                            ]);
                            self.form_index = self
                                .form_fields
                                .iter()
                                .position(|f| !f.readonly)
                                .unwrap_or(0);
                            self.job_list_action = None;
                            self.push_screen(Screen::RejectForm);
                        }
                        Some("update") => {
                            // Prellenar UpdateJobLookupForm: Job ID (readonly)
                            self.setup_form(vec![FormField::readonly(
                                "Job ID",
                                &job.job_id.to_string(),
                            )]);
                            self.form_index = 0;
                            self.job_list_action = None;
                            self.push_screen(Screen::UpdateJobLookupForm);
                        }
                        Some("accept") => {
                            // Prellenar AcceptForm: Job ID (readonly) + Client (readonly)
                            self.setup_form(vec![
                                FormField::readonly("Job ID", &job.job_id.to_string()),
                                FormField::readonly("Client Address", &job.client),
                            ]);
                            self.form_index = 0;
                            self.job_list_action = None;
                            self.push_screen(Screen::AcceptForm);
                        }
                        Some("submit") => {
                            // Prellenar SubmitForm: Job ID (readonly) + Client (readonly) + Notas (editable)
                            self.setup_form(vec![
                                FormField::readonly("Job ID", &job.job_id.to_string()),
                                FormField::readonly("Client Address", &job.client),
                                FormField::new(
                                    "Notas de entrega",
                                    "¿Qué entregaste? (opcional)",
                                    false,
                                ),
                            ]);
                            self.form_index = 2;
                            self.job_list_action = None;
                            self.push_screen(Screen::SubmitForm);
                        }
                        Some("raise_dispute") => {
                            // Prellenar RaiseDisputeForm: Job ID + Client (readonly) + Reason (editable)
                            self.setup_form(vec![
                                FormField::readonly("Job ID", &job.job_id.to_string()),
                                FormField::readonly("Client Address", &job.client),
                                FormField::new("Reason", "¿Por qué disputas el trabajo?", true),
                            ]);
                            self.form_index = self
                                .form_fields
                                .iter()
                                .position(|f| !f.readonly)
                                .unwrap_or(0);
                            self.job_list_action = None;
                            self.push_screen(Screen::RaiseDisputeForm);
                        }
                        Some("resolve") => {
                            // Prellenar ResolveDisputeForm: Job ID + Client + Freelancer (readonly) + % (editable) + Notas (editable)
                            let fl = job.freelancer.clone().unwrap_or_default();
                            self.setup_form(vec![
                                FormField::readonly("Job ID", &job.job_id.to_string()),
                                FormField::readonly("Client Address", &job.client),
                                FormField::readonly("Freelancer Address", &fl),
                                FormField::new(
                                    "Freelancer %",
                                    "0-100 (% para el freelancer)",
                                    true,
                                ),
                                FormField::new("Notas de resolución", "Explica tu decisión", true),
                            ]);
                            self.form_index = self
                                .form_fields
                                .iter()
                                .position(|f| !f.readonly)
                                .unwrap_or(0);
                            self.job_list_action = None;
                            self.push_screen(Screen::ResolveDisputeForm);
                        }
                        _ => {}
                    }
                }
            }
            KeyCode::Char('r') => {
                // Refrescar la lista según el contexto actual
                if let Some(action) = self.job_list_action.clone() {
                    let statuses: &[&str] = match action.as_str() {
                        "deposit" => &["Created"],
                        "cancel" => &["Created", "Funded"],
                        "approve" => &["Submitted"],
                        "reject" => &["Submitted"],
                        "update" => &["Created", "Funded"],
                        "accept" => &["Funded"],
                        "submit" => &["InProgress"],
                        "raise_dispute" => &["Submitted", "InProgress"],
                        "resolve" => &["Disputed"],
                        _ => &[],
                    };
                    self.fetch_jobs_for_action(&action, statuses);
                } else {
                    self.fetch_and_show_jobs();
                }
            }
            KeyCode::Esc | KeyCode::Char('q') => {
                self.job_list_action = None;
                self.pop_screen();
            }
            _ => {}
        }
    }

    /// client.json / devnet-client.json -> "client", id.json -> "admin", etc.
    fn role_from_keypair_path(path: &str) -> &'static str {
        let stem = std::path::Path::new(path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("");
        // Quitar prefijo de red si existe (devnet-, mainnet-)
        let stem = stem
            .strip_prefix("devnet-")
            .or_else(|| stem.strip_prefix("mainnet-"))
            .unwrap_or(stem);
        match stem {
            "client" => "client",
            "freelancer" => "freelancer",
            "arbiter" => "arbiter",
            "treasury" => "treasury",
            _ => "admin", // id.json y cualquier otro
        }
    }

    /// Cuando el campo de keypair path cambia (en AddWalletForm),
    /// sincroniza automáticamente el campo Role.
    fn sync_role_to_path(&mut self) {
        if self.screen != Screen::AddWalletForm {
            return;
        }
        // field 1 = Keypair Path, field 2 = Role
        let path_val = self
            .form_fields
            .get(1)
            .map(|f| f.value.clone())
            .unwrap_or_default();
        let inferred = Self::role_from_keypair_path(&path_val);
        if let Some(role_field) = self.form_fields.get_mut(2) {
            if let Some(idx) = role_field.options.iter().position(|o| o == inferred) {
                role_field.option_index = idx;
                role_field.value = inferred.to_string();
            }
        }
    }

    fn handle_form_event(&mut self, code: KeyCode) {
        match code {
            KeyCode::Tab => {
                // Saltar campos readonly
                let mut next = self.form_index;
                loop {
                    if next < self.form_fields.len() - 1 {
                        next += 1;
                    } else {
                        break;
                    }
                    if !self.form_fields[next].readonly {
                        break;
                    }
                }
                self.form_index = next;
            }
            KeyCode::BackTab => {
                // Saltar campos readonly
                let mut prev = self.form_index;
                loop {
                    if prev > 0 {
                        prev -= 1;
                    } else {
                        break;
                    }
                    if !self.form_fields[prev].readonly {
                        break;
                    }
                }
                self.form_index = prev;
            }
            KeyCode::Up => {
                let mut prev = self.form_index;
                loop {
                    if prev > 0 {
                        prev -= 1;
                    } else {
                        break;
                    }
                    if !self.form_fields[prev].readonly {
                        break;
                    }
                }
                self.form_index = prev;
            }
            KeyCode::Down => {
                let mut next = self.form_index;
                loop {
                    if next < self.form_fields.len() - 1 {
                        next += 1;
                    } else {
                        break;
                    }
                    if !self.form_fields[next].readonly {
                        break;
                    }
                }
                self.form_index = next;
            }
            KeyCode::Left => {
                if let Some(field) = self.form_fields.get_mut(self.form_index) {
                    if !field.readonly && !field.options.is_empty() {
                        if field.option_index == 0 {
                            field.option_index = field.options.len() - 1;
                        } else {
                            field.option_index -= 1;
                        }
                        field.value = field.options[field.option_index].clone();
                        self.message = None;
                    }
                }
                self.sync_role_to_path();
            }
            KeyCode::Right => {
                if let Some(field) = self.form_fields.get_mut(self.form_index) {
                    if !field.readonly && !field.options.is_empty() {
                        field.option_index = (field.option_index + 1) % field.options.len();
                        field.value = field.options[field.option_index].clone();
                        self.message = None;
                    }
                }
                self.sync_role_to_path();
            }
            KeyCode::Char(c) => {
                if let Some(field) = self.form_fields.get_mut(self.form_index) {
                    if !field.readonly && field.options.is_empty() {
                        field.value.push(c);
                        self.message = None;
                    }
                }
            }
            KeyCode::Backspace => {
                if let Some(field) = self.form_fields.get_mut(self.form_index) {
                    if !field.readonly && field.options.is_empty() {
                        field.value.pop();
                        self.message = None;
                    }
                }
            }
            KeyCode::Enter => {
                self.submit_form();
            }
            KeyCode::Esc => {
                self.pop_screen();
            }
            _ => {}
        }
    }

    // ─── Direct Actions (Pause/Unpause) ──────────────────────────────────

    fn execute_direct_action(&mut self, action: &str) {
        let wallet_path = match self.active_wallet() {
            Some(w) => w.path.clone(),
            None => {
                self.message = Some(("No wallet selected".into(), MessageType::Error));
                return;
            }
        };
        let rpc = solana::make_rpc(self.rpc_url());
        let kp = match solana::load_keypair(&wallet_path) {
            Ok(kp) => kp,
            Err(e) => {
                self.message = Some((format!("Keypair error: {e}"), MessageType::Error));
                return;
            }
        };
        let result = match action {
            "pause" => solana::op_pause(&rpc, &kp),
            "unpause" => solana::op_unpause(&rpc, &kp),
            _ => return,
        };
        match result {
            Ok(text) => {
                info!(action, "Acción admin completada");
                self.result_text = text;
                self.screen = Screen::Result;
                self.message = None;
            }
            Err(e) => {
                error!(action, error = %e, "Error en acción admin");
                self.result_text = format!("❌ Error:\n{e}");
                self.screen = Screen::Result;
                self.message = None;
            }
        }
    }
}
