use dioxus::prelude::*;
use crate::i18n::{tr, use_i18n};
use crate::ui::Reveal;

// Animated counter that roulettes from 0 to target when scrolled into view
#[component]
fn Counter(value: String, delay: u64) -> Element {
    let mut display = use_signal(|| String::new());
    let is_numeric = value.chars().any(|c| c.is_ascii_digit()) && value != "Solana";

    // Parse target and format for animation
    let target_str = value.clone();
    use_effect(move || {
        if !is_numeric {
            display.set(target_str.clone());
            return;
        }
        // Delay start for stagger effect
        let target = target_str.clone();
        spawn(async move {
            // Server (SSR) — no animation, just final value
            #[cfg(not(target_arch = "wasm32"))]
            {
                display.set(target.clone());
                return;
            }
            #[cfg(target_arch = "wasm32")]
            {
                if delay > 0 {
                    gloo_timers::future::TimeoutFuture::new(delay as u32).await;
                }
                // Parse numeric parts: e.g. "$2.4M" -> 2.4, "18k+" -> 18, "3.2k" -> 3.2
                let numeric_part: String = target.chars().filter(|c| c.is_ascii_digit() || *c == '.').collect();
                let prefix: String = target.chars().take_while(|c| !c.is_ascii_digit()).collect();
                let suffix: String = target.chars().skip_while(|c| c.is_ascii_digit() || *c == '.' || *c == '$').collect();
                // Handle $ prefix specially
                let clean_prefix = if target.starts_with('$') { "$".to_string() } else { prefix };
                let clean_suffix = if target.starts_with('$') {
                    target.chars().skip(1).skip_while(|c| c.is_ascii_digit() || *c == '.').collect()
                } else {
                    suffix
                };
                let target_num: f64 = numeric_part.parse().unwrap_or(0.0);
                let is_decimal = numeric_part.contains('.');
                let steps = 40;
                let step_delay = 30; // ms per step

                for i in 0..=steps {
                    let progress = i as f64 / steps as f64;
                    // Ease out cubic for smooth roulette slowdown
                    let eased = 1.0 - (1.0 - progress).powi(3);
                    let current = target_num * eased;
                    // Add roulette jitter for first 70% of animation (simple pseudo-random via step)
                    let jitter = if progress < 0.7 {
                        let pseudo = ((i * 37 + 13) % 10) as f64 / 10.0 - 0.5;
                        let jitter_val = pseudo * 0.3 * target_num * (1.0 - progress);
                        (current + jitter_val).max(0.0)
                    } else {
                        current
                    };
                    let formatted = if is_decimal {
                        format!("{}{:.1}{}", clean_prefix, jitter, clean_suffix)
                    } else {
                        format!("{}{}{}", clean_prefix, jitter as u64, clean_suffix)
                    };
                    // Final frame: exact value
                    let final_display = if i == steps {
                        target.clone()
                    } else {
                        formatted
                    };
                    display.set(final_display);
                    gloo_timers::future::TimeoutFuture::new(step_delay as u32).await;
                }
            }
        });
    });

    rsx! {
        span { class: "text-3xl md:text-4xl font-bold tracking-tight gradient bg-clip-text text-transparent tabular-nums",
            {display.read().clone()}
        }
    }
}

#[component]
pub fn Stats() -> Element {
    let l = *use_i18n().lang.read();
    let items = [
        ("$2.4M", "stats.tvl", 0u64),
        ("18k+", "stats.tx", 100u64),
        ("3.2k", "stats.users", 200u64),
        ("Solana", "stats.chain", 300u64),
    ];
    rsx! {
        section { class: "py-24",
            div { class: "wrap",
                Reveal {
                    h2 { class: "text-3xl font-bold tracking-tight text-center mb-12", {tr(l, "stats.title")} }
                }
                div { class: "grid gap-6 text-center grid-cols-2 lg:grid-cols-4",
                    for (value, label, delay) in items {
                        Reveal { delay: delay,
                            div { class: "flex flex-col gap-1.5",
                                Counter { value: value.to_string(), delay: delay }
                                span { class: "text-sm text-muted", {tr(l, label)} }
                            }
                        }
                    }
                }
            }
        }
    }
}
