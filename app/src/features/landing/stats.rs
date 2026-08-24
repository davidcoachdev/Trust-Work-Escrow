use crate::i18n::{tr, use_i18n};
use crate::ui::Reveal;
use dioxus::prelude::*;

// Per-character roulette: each char cycles through its charset (A-Z, 0-9, symbols) then lands
fn charset_for(c: char) -> Vec<char> {
    if c.is_ascii_digit() {
        ('0'..='9').collect()
    } else if c.is_ascii_uppercase() {
        ('A'..='Z').collect()
    } else if c.is_ascii_lowercase() {
        ('a'..='z').collect()
    } else {
        // symbols: include target char plus common symbols for roulette
        let mut v: Vec<char> = vec![
            '$', '.', '+', '%', '#', '*', '&', '!', '?', '@', '-', '/', 'M', 'k',
        ];
        if !v.contains(&c) {
            v.push(c);
        }
        v
    }
}

#[component]
fn Counter(value: String, delay: u64) -> Element {
    let mut display = use_signal(|| String::new());

    let target_str = value.clone();
    use_effect(move || {
        let target = target_str.clone();
        spawn(async move {
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
                // Start with blanks
                let len = target.chars().count();
                let mut current: Vec<char> = vec![' '; len];
                display.set(current.iter().collect());

                // Animate each character sequentially like a slot machine
                for (char_idx, target_char) in target.chars().enumerate() {
                    let charset = charset_for(target_char);
                    let target_pos = charset.iter().position(|&x| x == target_char).unwrap_or(0);
                    // Show ~5 random chars + 2 full cycles before landing
                    let cycles = 2;
                    let extra = 5;
                    let total_steps = charset.len() * cycles + target_pos + extra;
                    // Stagger per character
                    let char_delay = if char_idx > 0 { 60 } else { 0 };
                    if char_delay > 0 {
                        gloo_timers::future::TimeoutFuture::new(char_delay as u32).await;
                    }
                    for step in 0..total_steps {
                        let idx = step % charset.len();
                        // For the last few steps, ease to target
                        let display_char = if step >= total_steps - 5 {
                            // Slow down: show target approaching
                            if step == total_steps - 1 {
                                target_char
                            } else {
                                charset[(target_pos + charset.len() - (total_steps - step))
                                    % charset.len()]
                            }
                        } else {
                            charset[idx]
                        };
                        current[char_idx] = display_char;
                        // Add counter-like vertical jitter via random offset for first 70%
                        display.set(current.iter().collect());
                        let step_delay = if step < total_steps - 5 { 35 } else { 80 };
                        gloo_timers::future::TimeoutFuture::new(step_delay as u32).await;
                    }
                    current[char_idx] = target_char;
                    display.set(current.iter().collect());
                }
                // Ensure final exact value
                display.set(target.clone());
            }
        });
    });

    rsx! {
        span { class: "text-3xl md:text-4xl font-bold tracking-tight gradient bg-clip-text text-transparent tabular-nums tracking-wider",
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
