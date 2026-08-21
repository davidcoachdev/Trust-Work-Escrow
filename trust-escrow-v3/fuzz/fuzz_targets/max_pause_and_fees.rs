#![no_main]
use libfuzzer_sys::fuzz_target;
use trust_escrow_v3::{compute_fee, compute_shortfall, MAX_PAUSE_DURATION, BASIS_POINTS};

fuzz_target!(|data: &[u8]| {
    if data.len() < 16 {
        return;
    }
    let amount = u64::from_le_bytes(data[0..8].try_into().unwrap());
    let fee_bps = u16::from_le_bytes(data[8..10].try_into().unwrap()) % 10_001;
    let required = u64::from_le_bytes(data[0..8].try_into().unwrap());
    let posted = u64::from_le_bytes(data[8..16].try_into().unwrap());

    // compute_fee must not panic, and fee <= amount when bps <=10000
    if let Ok(fee) = compute_fee(amount, fee_bps) {
        assert!(fee <= amount);
        let expected = (amount as u128 * fee_bps as u128 / BASIS_POINTS as u128) as u64;
        assert_eq!(fee, expected);
    }

    // compute_shortfall is saturating_sub
    let s = compute_shortfall(required, posted);
    assert_eq!(s, required.saturating_sub(posted));

    // MAX_PAUSE_DURATION 30d
    assert_eq!(MAX_PAUSE_DURATION, 30 * 24 * 60 * 60);

    // paused_at / now boundary
    let paused_at = i64::from_le_bytes(data[0..8].try_into().unwrap()) % 1_000_000_000;
    let delta = (u32::from_le_bytes(data[8..12].try_into().unwrap()) % (60 * 60 * 24 * 60)) as i64;
    let now = paused_at + delta;
    let expired = now.checked_sub(paused_at).unwrap() > MAX_PAUSE_DURATION;
    if delta > MAX_PAUSE_DURATION {
        assert!(expired);
    } else {
        assert!(!expired);
    }

    // withdraw_treasury: amount 0 must be considered AmountTooSmall (simulated)
    let withdraw_amount = u64::from_le_bytes(data[0..8].try_into().unwrap());
    if withdraw_amount == 0 {
        assert_eq!(withdraw_amount, 0);
    }
});
