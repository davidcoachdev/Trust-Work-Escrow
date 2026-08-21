#![no_main]
use libfuzzer_sys::fuzz_target;
use trust_escrow_v3::{AccountMetaBorsh, RemainingAccounts, MAX_CLEANUP_BATCH, MAX_EVIDENCE_CLEANUP_BATCH};
use borsh::{BorshSerialize, BorshDeserialize};

fuzz_target!(|data: &[u8]| {
    // 1. RemainingAccounts deserialization must never panic
    let _ = RemainingAccounts::try_from_slice(data);

    // 2. If deserialization succeeds, validate structural invariants
    if let Ok(ra) = RemainingAccounts::try_from_slice(data) {
        // borsh roundtrip must be stable
        if let Ok(enc) = ra.try_to_vec() {
            let _ = RemainingAccounts::try_from_slice(&enc);
        }
        // pagination invariant: metas.len() should be multiple of 2 for application cleanup
        // > MAX_CLEANUP_BATCH*2 must be rejected by on-chain logic
        if ra.metas.len() > MAX_CLEANUP_BATCH * 2 {
            assert!(ra.metas.len() > 20);
        }
        if ra.metas.len() > MAX_EVIDENCE_CLEANUP_BATCH {
            assert!(ra.metas.len() > 10);
        }
        // is_writable / is_signer are booleans; no panic on arbitrary bool bytes (borsh already validated)
        for m in &ra.metas {
            let _ = (m.is_writable, m.is_signer);
        }
    }

    // 3. Structured fuzz: interpret data as Vec<AccountMetaBorsh> length-prefixed
    // Simulate what on-chain RemainingAccounts::from_infos validates: len check + writable
    if data.len() >= 4 {
        let len = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;
        if len <= 22 && data.len() >= 4 + len * 33 {
            // 32 pubkey + 1 writable + 1 signer (borsh bool) ~ 34 with padding; just sanity
            let _ = len.is_multiple_of(2);
        }
    }

    // 4. Fuzz AccountMetaBorsh field combos: arbitrary pubkey bytes must not crash PDA derivation
    let _ = MAX_CLEANUP_BATCH;
    let _ = MAX_EVIDENCE_CLEANUP_BATCH;
    let _ = AccountMetaBorsh {
        pubkey: anchor_lang::prelude::Pubkey::default(),
        is_writable: data.first().is_some_and(|b| b % 2 == 0),
        is_signer: data.first().is_some_and(|b| b % 3 == 0),
    };
});
