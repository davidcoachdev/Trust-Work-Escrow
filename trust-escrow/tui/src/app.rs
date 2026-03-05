use crate::config::{Settings, Theme, WalletConfig};
use escrow_core as solana;
use escrow_core::Signer;
use crossterm::event::{Event, KeyCode, KeyModifiers};

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
    UpdateJobLookupForm,
    UpdateJobEditForm,
    // Result
    Result,
    // Settings
    SettingsMenu,
    SettingsTheme,
    SettingsNetwork,
    SettingsWallets,
    AddWalletForm,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Role {
    Admin,
    Client,
    Freelancer,
    Arbiter,
}

impl Role {
    pub fn label(&self) -> &str {
        match self {
            Role::Admin => "Admin",
            Role::Client => "Client",
            Role::Freelancer => "Freelancer",
            Role::Arbiter => "Arbiter",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "client" => Role::Client,
            "freelancer" => Role::Freelancer,
            "arbiter" => Role::Arbiter,
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
}

impl FormField {
    pub fn new(label: &str, placeholder: &str, required: bool) -> Self {
        Self {
            label: label.into(),
            value: String::new(),
            placeholder: placeholder.into(),
            required,
        }
    }
}

// ─── Menu Item ───────────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub struct MenuItem {
    pub label: String,
    pub screen: Screen,
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
            screen: Screen::WalletSelect,
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
        };
        app.build_wallet_list();
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

    fn rpc_url(&self) -> &str {
        &self.settings.rpc_url
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
        self.menu_items = vec![
            MenuItem {
                label: "👑 Admin".into(),
                screen: Screen::MainMenu,
            },
            MenuItem {
                label: "💼 Client".into(),
                screen: Screen::MainMenu,
            },
            MenuItem {
                label: "🔧 Freelancer".into(),
                screen: Screen::MainMenu,
            },
            MenuItem {
                label: "⚖️  Arbiter".into(),
                screen: Screen::MainMenu,
            },
        ];
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
                    label: "⏸️  Pause Program".into(),
                    screen: Screen::Result,
                });
                items.push(MenuItem {
                    label: "▶️  Unpause Program".into(),
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
                    label: "✏️  Update Job".into(),
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
                    label: "⚠️  Raise Dispute".into(),
                    screen: Screen::RaiseDisputeForm,
                });
            }
            Role::Arbiter => {
                items.push(MenuItem {
                    label: "⚖️  Resolve Dispute".into(),
                    screen: Screen::ResolveDisputeForm,
                });
            }
        }
        // Common to all roles
        items.push(MenuItem {
            label: "🔍 Show Job".into(),
            screen: Screen::ShowForm,
        });
        items.push(MenuItem {
            label: "🔄 Change Role".into(),
            screen: Screen::RoleSelect,
        });
        items.push(MenuItem {
            label: "👛 Change Wallet".into(),
            screen: Screen::WalletSelect,
        });
        items.push(MenuItem {
            label: "⚙️  Settings".into(),
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

    fn rebuild_current_menu(&mut self) {
        match self.screen {
            Screen::WalletSelect => self.build_wallet_list(),
            Screen::RoleSelect => self.build_role_menu(),
            Screen::MainMenu => self.build_main_menu(),
            Screen::SettingsMenu => self.build_settings_menu(),
            Screen::SettingsTheme => self.build_theme_list(),
            Screen::SettingsWallets => self.build_wallet_list(),
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
                self.setup_form(vec![
                    FormField::new("Title", "Job title (max 100 chars)", true),
                    FormField::new("Amount (SOL)", "e.g. 2.5", true),
                    FormField::new("Description", "Job description (optional)", false),
                    FormField::new("Arbiter Address", "Pubkey of arbiter", true),
                    FormField::new("Job ID", "Unique numeric ID e.g. 1", true),
                    FormField::new("Deadline (days)", "Days from now (default: 7)", false),
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
            Screen::SettingsNetwork => {
                let mut field = FormField::new("RPC URL", "http://127.0.0.1:8899", true);
                field.value = self.settings.rpc_url.clone();
                self.setup_form(vec![field]);
            }
            Screen::AddWalletForm => {
                self.setup_form(vec![
                    FormField::new("Wallet Name", "e.g. My Client Wallet", true),
                    FormField::new("Keypair Path", "e.g. ~/.config/solana/id.json", true),
                    FormField::new("Role", "admin / client / freelancer / arbiter", true),
                ]);
            }
            Screen::UpdateJobLookupForm => {
                self.setup_form(vec![FormField::new(
                    "Job ID",
                    "ID numérico del job a editar",
                    true,
                )]);
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
                self.message = Some((
                    format!("'{}' is required", f.label),
                    MessageType::Error,
                ));
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
                let job_id: u64 = match self.get_field(4).parse() {
                    Ok(v) => v,
                    Err(_) => {
                        self.message =
                            Some(("Invalid job ID (use a number)".into(), MessageType::Error));
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
                                "Invalid deadline (use days e.g. 7)".into(),
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
                        self.message =
                            Some(("Invalid job ID".into(), MessageType::Error));
                        return;
                    }
                };
                solana::op_deposit(&rpc, &kp, job_id)
            }

            Screen::AcceptForm => {
                let job_id: u64 = match self.get_field(0).parse() {
                    Ok(v) => v,
                    Err(_) => {
                        self.message =
                            Some(("Invalid job ID".into(), MessageType::Error));
                        return;
                    }
                };
                solana::op_accept(&rpc, &kp, job_id, self.get_field(1))
            }

            Screen::SubmitForm => {
                let job_id: u64 = match self.get_field(0).parse() {
                    Ok(v) => v,
                    Err(_) => {
                        self.message =
                            Some(("Invalid job ID".into(), MessageType::Error));
                        return;
                    }
                };
                solana::op_submit(&rpc, &kp, job_id, self.get_field(1))
            }

            Screen::ApproveForm => {
                let job_id: u64 = match self.get_field(0).parse() {
                    Ok(v) => v,
                    Err(_) => {
                        self.message =
                            Some(("Invalid job ID".into(), MessageType::Error));
                        return;
                    }
                };
                solana::op_approve(&rpc, &kp, job_id, self.get_field(1))
            }

            Screen::RejectForm => {
                let job_id: u64 = match self.get_field(0).parse() {
                    Ok(v) => v,
                    Err(_) => {
                        self.message =
                            Some(("Invalid job ID".into(), MessageType::Error));
                        return;
                    }
                };
                solana::op_reject(&rpc, &kp, job_id, self.get_field(1))
            }

            Screen::RaiseDisputeForm => {
                let job_id: u64 = match self.get_field(0).parse() {
                    Ok(v) => v,
                    Err(_) => {
                        self.message =
                            Some(("Invalid job ID".into(), MessageType::Error));
                        return;
                    }
                };
                solana::op_raise_dispute(&rpc, &kp, job_id, self.get_field(1), self.get_field(2))
            }

            Screen::ResolveDisputeForm => {
                let job_id: u64 = match self.get_field(0).parse() {
                    Ok(v) => v,
                    Err(_) => {
                        self.message =
                            Some(("Invalid job ID".into(), MessageType::Error));
                        return;
                    }
                };
                let pct: u8 = match self.get_field(3).parse() {
                    Ok(v) if v <= 100 => v,
                    _ => {
                        self.message = Some((
                            "Invalid percentage (0-100)".into(),
                            MessageType::Error,
                        ));
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
                )
            }

            Screen::CancelForm => {
                let job_id: u64 = match self.get_field(0).parse() {
                    Ok(v) => v,
                    Err(_) => {
                        self.message =
                            Some(("Invalid job ID".into(), MessageType::Error));
                        return;
                    }
                };
                solana::op_cancel(&rpc, &kp, job_id)
            }

            Screen::ShowForm => {
                let job_id: u64 = match self.get_field(0).parse() {
                    Ok(v) => v,
                    Err(_) => {
                        self.message =
                            Some(("Invalid job ID".into(), MessageType::Error));
                        return;
                    }
                };
                solana::op_show(&rpc, self.get_field(1), job_id).map(|info| info.to_string())
            }

            Screen::SettingsNetwork => {
                self.settings.rpc_url = self.get_field(0).to_string();
                let _ = self.settings.save();
                self.message = Some(("RPC URL saved!".into(), MessageType::Info));
                return;
            }

            Screen::AddWalletForm => {
                let name = self.get_field(0).to_string();
                let path = self.get_field(1).to_string();
                let role = self.get_field(2).to_string().to_lowercase();
                // Validate keypair exists
                if solana::load_keypair(&path).is_err() {
                    self.message =
                        Some((format!("Cannot load keypair at: {path}"), MessageType::Error));
                    return;
                }
                self.settings.wallets.push(WalletConfig { name, path, role });
                let _ = self.settings.save();
                self.message = Some(("Wallet added!".into(), MessageType::Success));
                self.pop_screen();
                self.rebuild_current_menu();
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
                        format!("Estado inválido: {} (debe ser Created o Funded)", info.status),
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
                        self.message = Some(("Monto inválido (usa e.g. 2.5)".into(), MessageType::Error));
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
                self.result_text = text;
                self.screen = Screen::Result;
                self.message = None;
            }
            Err(e) => {
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
                Screen::Result => self.handle_result_event(key.code),
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
                } else {
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
                self.role = match self.list_index {
                    0 => Role::Admin,
                    1 => Role::Client,
                    2 => Role::Freelancer,
                    3 => Role::Arbiter,
                    _ => Role::Admin,
                };
                // Update wallet role in settings
                if let Some(w) = self
                    .settings
                    .wallets
                    .get_mut(self.settings.active_wallet)
                {
                    w.role = self.role.label().to_lowercase();
                }
                let _ = self.settings.save();
                self.build_main_menu();
                self.go_to(Screen::MainMenu);
            }
            KeyCode::Esc => self.pop_screen(),
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
            KeyCode::Esc | KeyCode::Char('q') => {
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
                        self.build_form_for_screen(&Screen::SettingsNetwork);
                        self.push_screen(Screen::SettingsNetwork);
                    }
                    Screen::SettingsWallets => {
                        self.build_wallet_list();
                        self.push_screen(Screen::SettingsWallets);
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
                    self.message = Some((
                        format!("Theme changed to '{name}'"),
                        MessageType::Success,
                    ));
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

    fn handle_form_event(&mut self, code: KeyCode) {
        match code {
            KeyCode::Tab => {
                if !self.form_fields.is_empty() && self.form_index < self.form_fields.len() - 1 {
                    self.form_index += 1;
                }
            }
            KeyCode::BackTab => {
                if self.form_index > 0 {
                    self.form_index -= 1;
                }
            }
            KeyCode::Up => {
                if self.form_index > 0 {
                    self.form_index -= 1;
                }
            }
            KeyCode::Down => {
                if !self.form_fields.is_empty() && self.form_index < self.form_fields.len() - 1 {
                    self.form_index += 1;
                }
            }
            KeyCode::Char(c) => {
                if let Some(field) = self.form_fields.get_mut(self.form_index) {
                    field.value.push(c);
                    self.message = None;
                }
            }
            KeyCode::Backspace => {
                if let Some(field) = self.form_fields.get_mut(self.form_index) {
                    field.value.pop();
                    self.message = None;
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
                self.result_text = text;
                self.screen = Screen::Result;
                self.message = None;
            }
            Err(e) => {
                self.result_text = format!("❌ Error:\n{e}");
                self.screen = Screen::Result;
                self.message = None;
            }
        }
    }
}
