# Tasks: Polish Front i18n Wallet Dashboard

## Review Workload Forecast

| Field | Value |
|-------|-------|
| Estimated changed lines | 520–620 |
| 400-line budget risk | High |
| Chained PRs recommended | Yes |
| Suggested split | PR1 Wave1 visual → PR2 Wave2 wallet |
| Delivery strategy | ask-on-risk → chained (user chose Dividir en PRs) |
| Chain strategy | feature-branch-chain (tracker: feat/polish-front-i18n-wallet-dashboard, PR1: ...-wave1 → tracker) |

Decision needed before apply: Resolved — chained feature-branch-chain
Chained PRs recommended: Yes
Chain strategy: feature-branch-chain
400-line budget risk: High

### Suggested Work Units

| Unit | Goal | Likely PR | Focused test command | Runtime harness | Rollback boundary |
|------|------|-----------|----------------------|-----------------|-------------------|
| 1 | Wave1 visual: i18n/FAQ/Docs/sidebar/badge | PR1 → tracker | `cargo test -p trust-work-escrow-app -- tr lang fallback` | `dx serve 0.0.0.0:3001` toggle ES→EN, 390px collapse, header badge matrix | Revert `i18n/mod.rs`, `faq.rs`, `docs/mod.rs`, `dashboard_layout.rs`, `sidebar.rs` |
| 2 | Wave2 wallet: wasm keygen/seed/zeroize | PR2 → PR1 branch | `cargo test -p trust-work-escrow-app -- wallet mnemonic pubkey zeroize` | `dx serve 0.0.0.0:3001` Generate→grid→Copy→Confirm→Phantom→Close; `localStorage` grep | Remove `features/wallet/` + wasm deps in `app/Cargo.toml` |

## Phase 1: Foundation

- [ ] 1.1 Add `[target.wasm32.dependencies] bip39="2" zeroize="1"` to `app/Cargo.toml` scoped wasm-only, verify `cargo tree | grep ed25519` not in `api` (PR2)
- [x] 1.2 Extend `app/src/i18n/mod.rs` `tr` with ~40 keys (`faq.q/a 1-6`, `docs.*`, `dashboard.*`, `wallet.*`) fallback `key` when `_=>`, no `es.json` migration — PR1 wave1 620026c
- [ ] 1.3 Create `app/src/features/wallet/mod.rs` re-export + register `pub mod wallet;` in `app/src/features/mod.rs` (PR2)

## Phase 2: Wave 1 Visual & i18n

- [x] 2.1 Refactor `app/src/features/landing/faq.rs` to `let lang=*use_i18n().lang.read()` + `tr(lang,"faq.q{n}")`/`a{n}`, delete 6 hard-coded ES tuples — PR1 620026c
- [x] 2.2 Modify `app/src/features/docs/mod.rs` translate menu+bodies via `tr`, filter `main` by `selected` Signal (only `id==selected` visible), sync `href="#api"` to `selected` + `scrollIntoView` — PR1 620026c
- [x] 2.3 Modify `app/src/ui/dashboard_layout.rs` own `collapsed: Signal<bool>` (`twe-sidebar-collapsed` + `<768px`), resize listener, header badges `tr` `Crear billetera`/`Solo lectura` + pubkey chip `short_pubkey` — PR1 620026c
- [x] 2.4 Modify `app/src/ui/sidebar.rs` prop `collapsed: Signal<bool>`, `w-64↔w-16`, `translate-x-0`/`-translate-x-full md:translate-x-0`, `aria-expanded`, 200ms `transition-all`, `title` tooltips when collapsed — PR1 620026c

## Phase 3: Wave 2 Wallet Frontend-Only

- [ ] 3.1 Implement `app/src/features/wallet/create.rs` `generate_mnemonic()->Zeroizing<String>` via `bip39` 128-bit 12 words, `mnemonic_to_pubkey()->Result<String,String>` via `ed25519-dalek` bs58 44 chars (PR2)
- [ ] 3.2 Build `CreateWalletCard` seed grid 2×6 + Copy `web_sys::Clipboard::write_text` + warning `tr(wallet.seed.warning)` + Confirm checkbox + Phantom steps `tr(wallet.phantom.*)` (PR2)
- [ ] 3.3 Wire zeroize on `onunmount` + route `use_effect` + Forget/Close: `seed.write().as_mut().map(|z| z.zeroize()); seed.set(None)`; guard no `localStorage`/`console.log`/`api:3000` seed (PR2)
- [ ] 3.4 Embed `CreateWalletCard` in `app/src/features/dashboard/config.rs` when `wallet_pubkey.is_none()`, add `validate_base58_public_key` in `app/src/solana/phantom.rs` (PR2)

## Phase 4: Testing / Verification

- [ ] 4.1 Unit `cargo test -p trust-work-escrow-app`: `tr` fallback, `Lang` persist, 12-word, bs58 32B, `Zeroizing` Drop, `short_pubkey`, `validate_base58` (PR1 partial: tr fallback manual verified, 9 tests ok)
- [ ] 4.2 Manual `dx serve 0.0.0.0:3001` (don't kill): FAQ `lang.set(En)` reactive, Docs `selected="api"` filter + `#architecture` hash, sidebar `w-16` persisted before paint, 390px auto-collapse, header CTA/badge matrix (pending verify)
- [ ] 4.3 E2E toggle ES→EN one frame, Generate→grid→Copy→Confirm→Phantom→Close zeroizes, `localStorage` has no `mnemonic` keys, `console` has no seed, `cargo tree` wasm-only (PR2)

## Phase 5: Cleanup

- [ ] 5.1 Grep hard-coded literals `faq.rs` zero, `cargo audit`, `dx build` release verify, `aria-expanded`/transition audit (PR1 grep done, audit pending)
