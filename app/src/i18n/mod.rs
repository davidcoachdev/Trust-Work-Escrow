use dioxus::prelude::*;
use std::str::FromStr;
#[cfg(target_arch = "wasm32")]
use web_sys::{window, Window};

/// Languages. ES is the default (Latam); EN is the second.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Lang {
    Es,
    En,
}

impl Lang {
    pub fn as_str(self) -> &'static str {
        match self {
            Lang::Es => "es",
            Lang::En => "en",
        }
    }

    /// Short label shown in the switcher (ES / EN).
    pub fn label(self) -> &'static str {
        match self {
            Lang::Es => "ES",
            Lang::En => "EN",
        }
    }

    pub fn all() -> &'static [Lang] {
        &[Lang::Es, Lang::En]
    }
}

impl FromStr for Lang {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, ()> {
        match s.to_lowercase().as_str() {
            "es" => Ok(Lang::Es),
            "en" => Ok(Lang::En),
            _ => Err(()),
        }
    }
}

pub const LANG_KEY: &str = "twe-lang";

/// Apply the language to <html lang> and persist it.
#[cfg(target_arch = "wasm32")]
pub fn apply_lang(lang: Lang) {
    if let Some(win) = window() {
        if let Some(doc) = win.document() {
            if let Some(el) = doc.document_element() {
                let _ = el.set_attribute("lang", lang.as_str());
            }
            if let Ok(Some(storage)) = win.local_storage() {
                let _ = storage.set_item(LANG_KEY, lang.as_str());
            }
        }
    }
}

/// SSR: no browser DOM to touch.
#[cfg(not(target_arch = "wasm32"))]
pub fn apply_lang(_lang: Lang) {}

/// Read persisted language, else browser language, else default ES.
#[cfg(target_arch = "wasm32")]
pub fn load_lang() -> Lang {
    if let Some(value) = window()
        .and_then(|w: Window| w.local_storage().ok().flatten())
        .and_then(|s| s.get_item(LANG_KEY).ok().flatten())
        .and_then(|v| Lang::from_str(&v).ok())
    {
        return value;
    }
    if let Some(nav) = window().and_then(|w| w.navigator().language()) {
        if nav.to_lowercase().starts_with("es") {
            return Lang::Es;
        }
    }
    Lang::Es
}

/// SSR fallback.
#[cfg(not(target_arch = "wasm32"))]
pub fn load_lang() -> Lang {
    Lang::Es
}

#[derive(Clone, Copy)]
pub struct I18nContext {
    pub lang: Signal<Lang>,
}

pub fn use_i18n() -> I18nContext {
    use_context::<I18nContext>()
}

/// Resolve a dotted key for the active language. Fallback to key when missing (prevents blank UI).
pub fn tr(lang: Lang, key: &str) -> &'static str {
    let es = match key {
        "brand" => "Trust Work Escrow",
        "nav.home" => "Inicio",
        "nav.jobs" => "Trabajos",
        "nav.docs" => "Documentación",
        "nav.login" => "Ingresar",
        "nav.signup" => "Registrarse",
        "nav.contact" => "Contacto",
        "hero.title" => "Escrow descentralizado para trabajo real",
        "hero.subtitle" => "Pagos seguros entre empleadores y freelancers, liquidados on-chain con Solana.",
        "hero.cta" => "Empezar",
        "hero.ctaSecondary" => "Ver documentación",
        "features.title" => "Por qué Trust Work Escrow",
        "features.secure.title" => "Escrow seguro",
        "features.secure.body" => "Los fondos se bloquean en un programa hasta que ambas partes acuerdan.",
        "features.proof.title" => "Prueba on-chain",
        "features.proof.body" => "Cada hito se liquida de forma transparente en Solana.",
        "features.fees.title" => "Comisiones bajas",
        "features.fees.body" => "Una fracción de los costos de escrow tradicional.",
        "cta.title" => "¿Listo para empezar?",
        "cta.body" => "Crea tu primer escrow en minutos.",
        "cta.button" => "Conectar billetera",
        "contact.title" => "Contacto",
        "auth.name" => "Nombre",
        "auth.email" => "Email",
        "auth.password" => "Contraseña",
        "auth.message" => "Mensaje",
        "auth.send" => "Enviar",
        "footer.copyright" => "© 2026 Trust Work Escrow",
        "footer.onSolana" => "escrow en Solana",
        "footer.links.docs" => "Documentación",
        "footer.links.github" => "GitHub",
        "footer.links.status" => "Estado",
        "switcher.language" => "Idioma",
        "switcher.theme" => "Tema",
        "switcher.light" => "Claro",
        "switcher.dark" => "Oscuro",
        "hero.badge" => "Escrow on-chain en Solana",
        "hero.trust" => "Sin intermediarios. Los fondos se liberan solo cuando ambas partes acuerdan.",
        "how.title" => "Cómo funciona",
        "how.step1.title" => "Creás el escrow",
        "how.step1.body" => "El empleador deposita los fondos en un programa de Solana. Quedan bloqueados y seguros.",
        "how.step2.title" => "Entregás por hitos",
        "how.step2.body" => "El freelancer avanza y marca cada hito completado para su revisión.",
        "how.step3.title" => "Liberás y cobrás",
        "how.step3.body" => "Ambos aprueban y los fondos se liquidan on-chain al instante.",
        "stats.title" => "Confianza que se puede verificar",
        "stats.tvl" => "TVL bloqueado",
        "stats.tx" => "Transacciones",
        "stats.users" => "Usuarios",
        "stats.chain" => "Red",
        "who.title" => "Para quien es",
        "who.freelancers.title" => "Freelancers",
        "who.freelancers.body" => "Cobrá sin miedo a quedarte sin pago. El escrow garantiza tu trabajo desde el primer día.",
        "who.employers.title" => "Empleadores",
        "who.employers.body" => "Pagá solo cuando el trabajo está hecho. Control total y por hitos, sin sorpresas.",
        "features.transparent.title" => "Transparencia total",
        "features.transparent.body" => "Todo queda registrado en la blockchain, auditable por cualquiera en cualquier momento.",
        "features.instant.title" => "Liquidación instantánea",
        "features.instant.body" => "Sin esperas bancarias: los fondos llegan al instante en cuanto se aprueba.",
        "features.noncustodial.title" => "Non-custodial",
        "features.noncustodial.body" => "Tu dinero no pasa por nosotros; vive en el programa hasta que se libera.",
        // FAQ 6 Q&A
        "faq.title" => "Preguntas frecuentes",
        "faq.subtitle" => "Respuestas rápidas para GEO/AEO y motores de búsqueda generativos.",
        "faq.q1" => "¿Qué es escrow en Solana?",
        "faq.a1" => "Un contrato que bloquea fondos (SOL/USDC) en un programa de Solana hasta que empleador y freelancer cumplen las condiciones. Sin intermediario custodial: el dinero no pasa por nuestra billetera.",
        "faq.q2" => "¿Cómo se liberan los fondos?",
        "faq.a2" => "Por hitos: el empleador aprueba cada entrega y el programa libera automáticamente. Si ambas partes acuerdan, el saldo se liquida al cierre.",
        "faq.q3" => "¿Qué fees cobra Trust Work Escrow?",
        "faq.a3" => "Comisión mínima sobre el monto liberado, muy por debajo del escrow tradicional, más el fee base de red de Solana.",
        "faq.q4" => "¿Qué pasa si hay disputa?",
        "faq.a4" => "Cualquiera abre disputa; los fondos quedan bloqueados hasta resolución. El historial on-chain sirve como prueba auditable.",
        "faq.q5" => "¿Cómo funciona el arbitraje?",
        "faq.a5" => "Un árbitro neutral revisa evidencias y vota liberación parcial o total. La decisión se ejecuta on-chain y es auditable.",
        "faq.q6" => "¿Es seguro y no custodial?",
        "faq.a6" => "Sí: los fondos viven en el programa, no en nuestra custodia. Código auditable en devnet y claves solo en poder de las partes + árbitro.",
        // Docs sidebar + sections
        "docs.modules" => "Módulos",
        "docs.search.placeholder" => "Buscar (mock) — ej. escrow",
        "docs.breadcrumb.docs" => "docs",
        "docs.overview" => "overview",
        "docs.architecture" => "architecture",
        "docs.gettingStarted" => "getting_started",
        "docs.api" => "backend::api",
        "docs.sdk" => "backend::sdk",
        "docs.app" => "app (Dioxus)",
        "docs.deployment" => "deployment",
        "docs.overview.title" => "Crate trust_work_escrow",
        "docs.overview.version" => "Versión 0.1.0 · devnet · Solana · Axum · Dioxus 0.7",
        "docs.overview.body" => "Trust Work Escrow es un sistema de escrow descentralizado para trabajo freelance. Los fondos se bloquean on-chain en un programa de Solana (devnet 7a2Y…) hasta que cliente y freelancer acuerdan la liberación. El backend verifica, el SDK firma, y la app Dioxus orquesta la experiencia.",
        "docs.architecture.title" => "Arquitectura",
        "docs.architecture.body" => "Tres capas: programa on-chain, backend Axum, y app Dioxus (fullstack).",
        "docs.gettingStarted.title" => "Primeros pasos",
        "docs.api.title" => "API Docs — backend/api",
        "docs.api.body" => "Base: /api. Auth via cookie twe-jwt o twe-guest (guest 24h). Mutaciones requieren JWT.",
        "docs.sdk.title" => "SDK Docs — backend/sdk",
        "docs.sdk.body" => "El backend usa trust-escrow-sdk (features = [solana]) para construir instrucciones para 7a2Y; el navegador solo solicita, firma con Phantom y retransmite.",
        "docs.app.title" => "App — Dioxus 0.7",
        "docs.deployment.title" => "Despliegue — devnet",
        "docs.devnetOnly" => "Solo devnet",
        "docs.devnetOnly.body" => "Sin fondos reales. Links abren devnet.",
        // Dashboard shell
        "dashboard.createWallet" => "Crear billetera",
        "dashboard.readOnly" => "Solo lectura",
        "dashboard.guestReadOnly" => "Invitado · Solo lectura",
        "dashboard.loading" => "Cargando...",
        "dashboard.brand" => "Trust Work Escrow",
        "dashboard.toggleSidebar" => "Alternar barra lateral",
        "dashboard.header.noWallet" => "Sin billetera",
        // Wallet placeholder (Wave2 uses, but keys added now for i18n completeness)
        "wallet.create" => "Crear billetera",
        "wallet.seed.warning" => "Guardá estas 12 palabras en un lugar seguro. Nunca las compartas ni las subas a internet.",
        "wallet.seed.copy" => "Copiar",
        "wallet.seed.copied" => "Copiado",
        "wallet.seed.confirm" => "Confirmo que guardé la semilla en lugar seguro",
        "wallet.phantom.title" => "Importar en Phantom",
        "wallet.phantom.step1" => "Abrí Phantom → Agregar billetera",
        "wallet.phantom.step2" => "Importar frase semilla",
        "wallet.phantom.step3" => "Pegá las 12 palabras y confirmá",
        _ => "",
    };
    let en = match key {
        "brand" => "Trust Work Escrow",
        "nav.home" => "Home",
        "nav.jobs" => "Jobs",
        "nav.docs" => "Docs",
        "nav.login" => "Log in",
        "nav.signup" => "Sign up",
        "nav.contact" => "Contact",
        "hero.title" => "Decentralized escrow for real work",
        "hero.subtitle" => {
            "Secure payments between employers and freelancers, settled on-chain with Solana."
        }
        "hero.cta" => "Get started",
        "hero.ctaSecondary" => "View docs",
        "features.title" => "Why Trust Work Escrow",
        "features.secure.title" => "Secure escrow",
        "features.secure.body" => "Funds locked in a program until both parties agree.",
        "features.proof.title" => "On-chain proof",
        "features.proof.body" => "Every milestone settled transparently on Solana.",
        "features.fees.title" => "Low fees",
        "features.fees.body" => "Fraction of traditional escrow costs.",
        "cta.title" => "Ready to start?",
        "cta.body" => "Create your first escrow in minutes.",
        "cta.button" => "Connect wallet",
        "contact.title" => "Contact",
        "auth.name" => "Name",
        "auth.email" => "Email",
        "auth.password" => "Password",
        "auth.message" => "Message",
        "auth.send" => "Send",
        "footer.copyright" => "© 2026 Trust Work Escrow",
        "footer.onSolana" => "escrow on Solana",
        "footer.links.docs" => "Docs",
        "footer.links.github" => "GitHub",
        "footer.links.status" => "Status",
        "switcher.language" => "Language",
        "switcher.theme" => "Theme",
        "switcher.light" => "Light",
        "switcher.dark" => "Dark",
        "hero.badge" => "On-chain escrow on Solana",
        "hero.trust" => "No middlemen. Funds are released only when both parties agree.",
        "how.title" => "How it works",
        "how.step1.title" => "Create the escrow",
        "how.step1.body" => {
            "The employer deposits funds into a Solana program. They stay locked and safe."
        }
        "how.step2.title" => "Deliver in milestones",
        "how.step2.body" => {
            "The freelancer progresses and marks each milestone complete for review."
        }
        "how.step3.title" => "Release and get paid",
        "how.step3.body" => "Both approve and funds settle on-chain instantly.",
        "stats.title" => "Trust you can verify",
        "stats.tvl" => "Locked TVL",
        "stats.tx" => "Transactions",
        "stats.users" => "Users",
        "stats.chain" => "Network",
        "who.title" => "Who it's for",
        "who.freelancers.title" => "Freelancers",
        "who.freelancers.body" => {
            "Get paid without fear of non-payment. Escrow guarantees your work from day one."
        }
        "who.employers.title" => "Employers",
        "who.employers.body" => {
            "Pay only when the work is done. Full milestone control, no surprises."
        }
        "features.transparent.title" => "Full transparency",
        "features.transparent.body" => {
            "Everything is recorded on-chain, auditable by anyone at any time."
        }
        "features.instant.title" => "Instant settlement",
        "features.instant.body" => "No bank waits: funds arrive instantly once approved.",
        "features.noncustodial.title" => "Non-custodial",
        "features.noncustodial.body" => {
            "Your money never passes through us; it lives in the program until released."
        }
        "faq.title" => "Frequently asked questions",
        "faq.subtitle" => "Quick answers for GEO/AEO and generative search engines.",
        "faq.q1" => "What is escrow on Solana?",
        "faq.a1" => "A contract that locks funds (SOL/USDC) in a Solana program until employer and freelancer meet conditions. No custodial middleman: money never passes through our wallet.",
        "faq.q2" => "How are funds released?",
        "faq.a2" => "By milestones: the employer approves each delivery and the program releases automatically. If both agree, the balance settles on close.",
        "faq.q3" => "What fees does Trust Work Escrow charge?",
        "faq.a3" => "Minimal fee on the released amount, well below traditional escrow, plus Solana base network fee.",
        "faq.q4" => "What happens if there's a dispute?",
        "faq.a4" => "Anyone can open a dispute; funds stay locked until resolution. On-chain history serves as auditable proof.",
        "faq.q5" => "How does arbitration work?",
        "faq.a5" => "A neutral arbiter reviews evidence and votes partial or full release. The decision executes on-chain and is auditable.",
        "faq.q6" => "Is it safe and non-custodial?",
        "faq.a6" => "Yes: funds live in the program, not our custody. Auditable code on devnet and keys only held by parties + arbiter.",
        "docs.modules" => "Modules",
        "docs.search.placeholder" => "Search (mock) — e.g. escrow",
        "docs.breadcrumb.docs" => "docs",
        "docs.overview" => "overview",
        "docs.architecture" => "architecture",
        "docs.gettingStarted" => "getting_started",
        "docs.api" => "backend::api",
        "docs.sdk" => "backend::sdk",
        "docs.app" => "app (Dioxus)",
        "docs.deployment" => "deployment",
        "docs.overview.title" => "Crate trust_work_escrow",
        "docs.overview.version" => "Version 0.1.0 · devnet · Solana · Axum · Dioxus 0.7",
        "docs.overview.body" => "Trust Work Escrow is a decentralized escrow system for freelance work. Funds are locked on-chain in a Solana program (devnet 7a2Y…) until client and freelancer agree to release. Backend verifies, SDK signs, and Dioxus app orchestrates the experience.",
        "docs.architecture.title" => "Architecture",
        "docs.architecture.body" => "Three layers: on-chain program, Axum backend, and Dioxus app (fullstack).",
        "docs.gettingStarted.title" => "Getting Started",
        "docs.api.title" => "API Docs — backend/api",
        "docs.api.body" => "Base: /api. Auth via twe-jwt or twe-guest cookie (guest 24h). Mutations require JWT.",
        "docs.sdk.title" => "SDK Docs — backend/sdk",
        "docs.sdk.body" => "Backend uses trust-escrow-sdk (features = [solana]) to build instructions for 7a2Y; browser only requests, signs with Phantom and relays.",
        "docs.app.title" => "App — Dioxus 0.7",
        "docs.deployment.title" => "Deployment — devnet",
        "docs.devnetOnly" => "Devnet only",
        "docs.devnetOnly.body" => "No real funds. Links open devnet.",
        "dashboard.createWallet" => "Create wallet",
        "dashboard.readOnly" => "Read-only",
        "dashboard.guestReadOnly" => "Guest · Read-only",
        "dashboard.loading" => "Loading...",
        "dashboard.brand" => "Trust Work Escrow",
        "dashboard.toggleSidebar" => "Toggle sidebar",
        "dashboard.header.noWallet" => "No wallet",
        "wallet.create" => "Create wallet",
        "wallet.seed.warning" => "Save these 12 words in a safe place. Never share or upload them online.",
        "wallet.seed.copy" => "Copy",
        "wallet.seed.copied" => "Copied",
        "wallet.seed.confirm" => "I confirm I saved the seed in a safe place",
        "wallet.phantom.title" => "Import into Phantom",
        "wallet.phantom.step1" => "Open Phantom → Add wallet",
        "wallet.phantom.step2" => "Import seed phrase",
        "wallet.phantom.step3" => "Paste the 12 words and confirm",
        _ => "",
    };
    let val = match lang {
        Lang::Es => es,
        Lang::En => en,
    };
    if val.is_empty() {
        // Intentional leak: tr must return &'static str for Dioxus RSX (zero-copy).
        // Missing keys are bounded (dev typo only, never user input), so 1 alloc + leak per
        // distinct missing key is acceptable vs changing signature to String/Cow.
        // Expected path is the `else` branch (val from match), which allocates nothing.
        Box::leak(key.to_owned().into_boxed_str())
    } else {
        val
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tr_fallback_returns_key_not_empty() {
        let missing = "does.not.exist";
        let v_es = tr(Lang::Es, missing);
        let v_en = tr(Lang::En, missing);
        // Must return the key itself, not "" (prevents blank UI).
        assert_eq!(v_es, missing);
        assert_eq!(v_en, missing);
        assert!(!v_es.is_empty());
        assert!(!v_en.is_empty());
    }

    #[test]
    fn tr_known_returns_translation() {
        assert_eq!(tr(Lang::Es, "nav.home"), "Inicio");
        assert_eq!(tr(Lang::En, "nav.home"), "Home");
        assert_eq!(tr(Lang::Es, "faq.q1"), "¿Qué es escrow en Solana?");
        assert_eq!(tr(Lang::En, "faq.q1"), "What is escrow on Solana?");
    }
}
