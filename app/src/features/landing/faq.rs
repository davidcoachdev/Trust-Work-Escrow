use dioxus::prelude::*;

#[component]
pub fn Faq() -> Element {
    // 6 Q&A GEO-optimized — ES hardcode (tr keys no existen aún, fallback a ES)
    let items: [(&str, &str); 6] = [
        (
            "¿Qué es escrow en Solana?",
            "Un contrato que bloquea fondos (SOL/USDC) en un programa de Solana hasta que empleador y freelancer cumplen las condiciones. Sin intermediario custodial: el dinero no pasa por nuestra billetera.",
        ),
        (
            "¿Cómo se liberan los fondos?",
            "Por hitos: el empleador aprueba cada entrega y el programa libera automáticamente. Si ambas partes acuerdan, el saldo se liquida al cierre.",
        ),
        (
            "¿Qué fees cobra Trust Work Escrow?",
            "Comisión mínima sobre el monto liberado, muy por debajo del escrow tradicional, más el fee base de red de Solana.",
        ),
        (
            "¿Qué pasa si hay disputa?",
            "Cualquiera abre disputa; los fondos quedan bloqueados hasta resolución. El historial on-chain sirve como prueba auditable.",
        ),
        (
            "¿Cómo funciona el arbitraje?",
            "Un árbitro neutral revisa evidencias y vota liberación parcial o total. La decisión se ejecuta on-chain y es auditable.",
        ),
        (
            "¿Es seguro y no custodial?",
            "Sí: los fondos viven en el programa, no en nuestra custodia. Código auditable en devnet y claves solo en poder de las partes + árbitro.",
        ),
    ];

    rsx! {
        section { id: "faq", class: "py-24 bg-surface",
            div { class: "wrap",
                h2 { class: "text-3xl font-bold tracking-tight text-center mb-4", "Preguntas frecuentes" }
                p { class: "text-muted text-center max-w-[56ch] mx-auto mb-10", "Respuestas rápidas para GEO/AEO y motores de búsqueda generativos." }
                div { class: "grid gap-4 max-w-3xl mx-auto",
                    for (q, a) in items {
                        details { class: "group bg-bg border border-border rounded-2xl p-6 open:ring-1 open:ring-border",
                            summary { class: "font-medium cursor-pointer list-none flex justify-between items-center gap-4",
                                span { {q} }
                                span { class: "text-muted group-open:rotate-180 transition-transform", "▾" }
                            }
                            p { class: "text-muted mt-3 text-[15px] leading-relaxed", {a} }
                        }
                    }
                }
            }
        }
    }
}
