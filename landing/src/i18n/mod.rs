use dioxus::prelude::*;
use std::str::FromStr;
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

/// Read persisted language, else browser language, else default ES.
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

#[derive(Clone, Copy)]
pub struct I18nContext {
    pub lang: Signal<Lang>,
}

pub fn use_i18n() -> I18nContext {
    use_context::<I18nContext>()
}

/// Resolve a dotted key for the active language.
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
        "hero.subtitle" => "Secure payments between employers and freelancers, settled on-chain with Solana.",
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
        "how.step1.body" => "The employer deposits funds into a Solana program. They stay locked and safe.",
        "how.step2.title" => "Deliver in milestones",
        "how.step2.body" => "The freelancer progresses and marks each milestone complete for review.",
        "how.step3.title" => "Release and get paid",
        "how.step3.body" => "Both approve and funds settle on-chain instantly.",
        "stats.title" => "Trust you can verify",
        "stats.tvl" => "Locked TVL",
        "stats.tx" => "Transactions",
        "stats.users" => "Users",
        "stats.chain" => "Network",
        "who.title" => "Who it's for",
        "who.freelancers.title" => "Freelancers",
        "who.freelancers.body" => "Get paid without fear of non-payment. Escrow guarantees your work from day one.",
        "who.employers.title" => "Employers",
        "who.employers.body" => "Pay only when the work is done. Full milestone control, no surprises.",
        "features.transparent.title" => "Full transparency",
        "features.transparent.body" => "Everything is recorded on-chain, auditable by anyone at any time.",
        "features.instant.title" => "Instant settlement",
        "features.instant.body" => "No bank waits: funds arrive instantly once approved.",
        "features.noncustodial.title" => "Non-custodial",
        "features.noncustodial.body" => "Your money never passes through us; it lives in the program until released.",
        _ => "",
    };
    match lang {
        Lang::Es => es,
        Lang::En => en,
    }
}
