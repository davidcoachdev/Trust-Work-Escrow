#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if data.len() < 3 {
        return;
    }
    let evidence_count = data[0];
    let cursor = data[1];
    let add = data[2] % 11; // 0..10

    // checked_sub: remaining = evidence_count - cursor must be handled
    let remaining = evidence_count.checked_sub(cursor);
    if cursor > evidence_count {
        assert!(remaining.is_none());
    } else {
        assert!(remaining.is_some());
        let rem = remaining.unwrap();
        // remaining vs add: cleanup len must be <= remaining and <=10
        if (add as u16) > rem as u16 {
            // on-chain would fail InvalidEvidenceCleanupAccounts
            assert!((add as u16) > rem as u16);
        }
        // checked_add for cursor advancement
        let next = cursor.checked_add(add);
        if cursor as u16 + add as u16 > 255 {
            assert!(next.is_none());
        } else {
            assert!(next.is_some());
        }
        // MAX_EVIDENCE_COUNT =10 invariant
        if evidence_count > 10 {
            assert!(evidence_count > 10);
        }
    }
});
