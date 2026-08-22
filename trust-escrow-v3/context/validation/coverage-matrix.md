# Validation coverage matrix — 19/08/2026 (Waves 7-10 cerradas)

| Area | Evidence | Status |
|---|---|---|
| Config bootstrap/rotation | `initialize_config` con `INITIAL_AUTHORITY 3whY...`, treasuries `SYSTEM_PROGRAM_ID` separadas, `fee_bps <=10000`, `update_treasury` validado; `tests/escrow.ts` rota `treasury`/`arb_treasury` a System separadas y rechaza `InvalidTreasury` (9/9) | **PASS** |
| Arbiter governance | `createArbiterPool`/`addArbiter`/`removeArbiter`/`assignArbiter` solo `Config.authority`, `ArbiterPool` PDA `[b"arbiter_pool"]`, pool único; `tests/escrow.ts` + `backend` `instructions_jobs` verifican authority | **PASS** |
| Submitted/deadline | `submit_work → Submitted`, `submitted_at` + `AUTO_APPROVAL_DELAY 604800` exacto, `auto_approve_work` bloqueado si `Dispute` existe, `approve/reject` ventana `604800`; `tests/escrow.ts` rechaza auto-approval antes del deadline y `flujo completo` | **PASS** |
| Pause | `pause`/`unpause` global + `pause_job` solo `Created`/`Funded` sin `freelancer` (`freelancer == None`), `unpause_job`/`expire_paused_job` con `MAX_PAUSE_DURATION 30d`; `tests/escrow.ts` `pause_job rechaza con freelancer` PASS | **PASS** |
| Evidence/payout conservación | `Evidence` PDA `10` (`MAX_EVIDENCE_COUNT`), `content_hash [u8;32]` off-chain, `cleanup_dispute_evidence` + `finalize_dispute_payouts` con `arbitration_treasury` separado, fee `250 bps` por parte, `compute_fee` sin overflow; `tests/escrow.ts` evidencia `0-9` + límite `10` + `contentHash` hash | **PASS** |
| Reproducibility/deploy | Rust `1.89` + Anchor `0.32.1` alineados, `solana-test-validator --reset --ledger /tmp/validator-ledger` (Agave 4.1.1), `anchor deploy` `7a2Y...` slot 26, `sha256 c0bf3fa9...` (581KB), `Anchor.toml`/`declare_id!`/`PROGRAM_ID_STR` `7a2Y...`, `solana program show` authority `3whY...`, `yarn preflight` localnet-only | **PASS** |
| Docs/IDL | IDL `69KB` `sha256 de4a6b13...`, `Anchor.toml`/`lib.rs`/`types.rs` `Vec<Pubkey>`, `MAX_APPLICATIONS 50`, `MAX_MILESTONES 20`, `MAX_EVIDENCE 10`, `AUTO_APPROVAL 604800` + `check:docs` `ok` (no `Received`, no `Vec<Evidence>` inline, fee a `arbitration_treasury`) | **PASS** |
| Applications runtime | `Job.applicants: Vec<Pubkey> #[max_len(50)]` compacto, `Application` PDA `[b"application", job, index, applicant]` individual, `apply_to_job` con `proposal_hash [u8;32]`, `accept_application` + `cleanup_applications` con `is_multiple_of` + `saturating_sub`; `tests/escrow.ts` `50 PDAs + 51 rechazada` + `PDA derives` PASS (9/9) + `backend` `Vec` alineado 6/6 | **PASS** |

No public network or credential was used. Validator localnet `http://127.0.0.1:8899` con `health: ok`, deploy `7a2YhCd7...` reproducible.
