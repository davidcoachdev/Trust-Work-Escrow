use dioxus::prelude::*;
use crate::ui::{DashboardRole, Sidebar};

#[component]
pub fn ClientLayout() -> Element {
    rsx! {
        div { class: "flex min-h-screen bg-bg text-fg",
            Sidebar { role: DashboardRole::Client }
            div { class: "flex-1 p-8", Outlet::<crate::route::Route> {} }
        }
    }
}

#[component]
pub fn FreelancerLayout() -> Element {
    rsx! {
        div { class: "flex min-h-screen bg-bg text-fg",
            Sidebar { role: DashboardRole::Freelancer }
            div { class: "flex-1 p-8", Outlet::<crate::route::Route> {} }
        }
    }
}

#[component]
pub fn AdminLayout() -> Element {
    rsx! {
        div { class: "flex min-h-screen bg-bg text-fg",
            Sidebar { role: DashboardRole::Admin }
            div { class: "flex-1 p-8", Outlet::<crate::route::Route> {} }
        }
    }
}

#[component]
pub fn ClientDashboard() -> Element {
    rsx! {
        div { class: "space-y-6",
            h1 { class: "text-3xl font-bold text-primary", "Dashboard Cliente" }
            p { class: "text-muted", "Crea jobs, ve tu escrow JCR9... 0.115 SOL y acepta freelancers." }
            div { class: "bg-surface border border-border rounded-2xl p-6", "Próximo: crear job 0.1 SOL + ver PDA" }
        }
    }
}

#[component]
pub fn FreelancerDashboard() -> Element {
    rsx! {
        div { class: "space-y-6",
            h1 { class: "text-3xl font-bold text-primary", "Dashboard Freelancer" }
            p { class: "text-muted", "Jobs disponibles y tus postulaciones B5Ks..." }
            div { class: "bg-surface border border-border rounded-2xl p-6", "Próximo: listar jobs y apply" }
        }
    }
}

#[component]
pub fn AdminDashboard() -> Element {
    rsx! {
        div { class: "space-y-6",
            h1 { class: "text-3xl font-bold text-primary", "Dashboard Admin" }
            p { class: "text-muted", "Métricas, treasury 6KSy... y custody" }
            div { class: "bg-surface border border-border rounded-2xl p-6", "Solo manager/treasurer/custodian" }
        }
    }
}

pub use ClientLayout as ClientLayoutComponent;
pub use FreelancerLayout as FreelancerLayoutComponent;
pub use AdminLayout as AdminLayoutComponent;
