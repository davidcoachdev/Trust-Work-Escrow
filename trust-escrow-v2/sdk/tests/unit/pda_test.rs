//! Unit tests for PDA (Program Derived Address) functionality
//!
//! These tests verify that PDA derivation works correctly and consistently
//! for all account types in the Trust Escrow system.

use pretty_assertions::assert_eq;
use proptest::prelude::*;
use rstest::*;

use solana_sdk::{pubkey::Pubkey, signature::Keypair, signer::Signer};

use trust_escrow_sdk::pda::*;
use trust_escrow_sdk::PROGRAM_ID;

// Import common test utilities
mod common;
use common::*;

// ===== BASIC PDA DERIVATION TESTS =====

#[test]
fn test_find_user_pda_deterministic() {
    let authority = Keypair::new().pubkey();
    
    // PDA derivation should be deterministic
    let (pda1, bump1) = find_user_pda(&authority);
    let (pda2, bump2) = find_user_pda(&authority);
    
    assert_eq!(pda1, pda2, "PDA derivation should be deterministic");
    assert_eq!(bump1, bump2, "Bump should be deterministic");
    
    // PDA should not be the same as authority
    assert_ne!(pda1, authority, "PDA should be different from authority");
    
    // Bump should be valid (0-255)
    assert!(bump1 <= 255, "Bump should be valid");
    
    TestAssertions::assert_valid_pubkey(&pda1);
}

#[test]
fn test_find_job_pda_deterministic() {
    let client = Keypair::new().pubkey();
    let job_id = 42u64;
    
    let (pda1, bump1) = find_job_pda(&client, job_id);
    let (pda2, bump2) = find_job_pda(&client, job_id);
    
    assert_eq!(pda1, pda2, "Job PDA derivation should be deterministic");
    assert_eq!(bump1, bump2, "Job PDA bump should be deterministic");
    
    TestAssertions::assert_valid_pubkey(&pda1);
}

#[test]
fn test_find_team_pda_deterministic() {
    let owner = Keypair::new().pubkey();
    
    let (pda1, bump1) = find_team_pda(&owner);
    let (pda2, bump2) = find_team_pda(&owner);
    
    assert_eq!(pda1, pda2, "Team PDA derivation should be deterministic");
    assert_eq!(bump1, bump2, "Team PDA bump should be deterministic");
    
    TestAssertions::assert_valid_pubkey(&pda1);
}

#[test]
fn test_find_dispute_pda_deterministic() {
    let job = Keypair::new().pubkey();
    
    let (pda1, bump1) = find_dispute_pda(&job);
    let (pda2, bump2) = find_dispute_pda(&job);
    
    assert_eq!(pda1, pda2, "Dispute PDA derivation should be deterministic");
    assert_eq!(bump1, bump2, "Dispute PDA bump should be deterministic");
    
    TestAssertions::assert_valid_pubkey(&pda1);
}

#[test]
fn test_find_milestone_pda_deterministic() {
    let job = Keypair::new().pubkey();
    let index = 5u8;
    
    let (pda1, bump1) = find_milestone_pda(&job, index);
    let (pda2, bump2) = find_milestone_pda(&job, index);
    
    assert_eq!(pda1, pda2, "Milestone PDA derivation should be deterministic");
    assert_eq!(bump1, bump2, "Milestone PDA bump should be deterministic");
    
    TestAssertions::assert_valid_pubkey(&pda1);
}

#[test]
fn test_find_config_pda_deterministic() {
    let (pda1, bump1) = find_config_pda();
    let (pda2, bump2) = find_config_pda();
    
    assert_eq!(pda1, pda2, "Config PDA derivation should be deterministic");
    assert_eq!(bump1, bump2, "Config PDA bump should be deterministic");
    
    TestAssertions::assert_valid_pubkey(&pda1);
}

#[test] 
fn test_find_arbiter_pool_pda_deterministic() {
    let (pda1, bump1) = find_arbiter_pool_pda();
    let (pda2, bump2) = find_arbiter_pool_pda();
    
    assert_eq!(pda1, pda2, "Arbiter pool PDA derivation should be deterministic");
    assert_eq!(bump1, bump2, "Arbiter pool PDA bump should be deterministic");
    
    TestAssertions::assert_valid_pubkey(&pda1);
}

// ===== PDA UNIQUENESS TESTS =====

#[test]
fn test_user_pdas_are_unique() {
    let authority1 = Keypair::new().pubkey();
    let authority2 = Keypair::new().pubkey();
    
    let (pda1, _) = find_user_pda(&authority1);
    let (pda2, _) = find_user_pda(&authority2);
    
    assert_ne!(pda1, pda2, "Different users should have different PDAs");
}

#[test]
fn test_job_pdas_are_unique() {
    let client = Keypair::new().pubkey();
    let job_id1 = 1u64;
    let job_id2 = 2u64;
    
    let (pda1, _) = find_job_pda(&client, job_id1);
    let (pda2, _) = find_job_pda(&client, job_id2);
    
    assert_ne!(pda1, pda2, "Different job IDs should have different PDAs");
}

#[test]
fn test_milestone_pdas_are_unique() {
    let job = Keypair::new().pubkey();
    let index1 = 0u8;
    let index2 = 1u8;
    
    let (pda1, _) = find_milestone_pda(&job, index1);
    let (pda2, _) = find_milestone_pda(&job, index2);
    
    assert_ne!(pda1, pda2, "Different milestone indices should have different PDAs");
}

#[test]
fn test_different_account_types_have_different_pdas() {
    let authority = Keypair::new().pubkey();
    
    let (user_pda, _) = find_user_pda(&authority);
    let (team_pda, _) = find_team_pda(&authority);
    let (job_pda, _) = find_job_pda(&authority, 1);
    let (config_pda, _) = find_config_pda();
    let (arbiter_pool_pda, _) = find_arbiter_pool_pda();
    
    // All PDAs should be unique
    let pdas = vec![user_pda, team_pda, job_pda, config_pda, arbiter_pool_pda];
    for i in 0..pdas.len() {
        for j in (i + 1)..pdas.len() {
            assert_ne!(
                pdas[i], 
                pdas[j], 
                "Different account types should have different PDAs"
            );
        }
    }
}

// ===== EDGE CASES TESTS =====

#[test]
fn test_job_pda_with_zero_id() {
    let client = Keypair::new().pubkey();
    let job_id = 0u64;
    
    let (pda, bump) = find_job_pda(&client, job_id);
    
    TestAssertions::assert_valid_pubkey(&pda);
    assert!(bump <= 255, "Bump should be valid");
}

#[test]
fn test_job_pda_with_max_id() {
    let client = Keypair::new().pubkey();
    let job_id = u64::MAX;
    
    let (pda, bump) = find_job_pda(&client, job_id);
    
    TestAssertions::assert_valid_pubkey(&pda);
    assert!(bump <= 255, "Bump should be valid");
}

#[test]
fn test_milestone_pda_with_max_index() {
    let job = Keypair::new().pubkey();
    let index = u8::MAX;
    
    let (pda, bump) = find_milestone_pda(&job, index);
    
    TestAssertions::assert_valid_pubkey(&pda);
    assert!(bump <= 255, "Bump should be valid");
}

// ===== PROGRAM ID VALIDATION TESTS =====

#[test]
fn test_pdas_use_correct_program_id() {
    let authority = Keypair::new().pubkey();
    
    let (user_pda, _) = find_user_pda(&authority);
    
    // Verify PDA is derived using the correct program ID
    let seeds = &[b"user", authority.as_ref()];
    let (expected_pda, _) = Pubkey::find_program_address(seeds, &PROGRAM_ID);
    
    assert_eq!(user_pda, expected_pda, "PDA should be derived with correct program ID");
}

#[test]
fn test_all_pda_functions_use_program_id() {
    // This test ensures all PDA functions use PROGRAM_ID constant
    let authority = Keypair::new().pubkey();
    let job = Keypair::new().pubkey();
    
    // User PDA
    let (user_pda, _) = find_user_pda(&authority);
    let user_seeds = &[b"user", authority.as_ref()];
    let (expected_user_pda, _) = Pubkey::find_program_address(user_seeds, &PROGRAM_ID);
    assert_eq!(user_pda, expected_user_pda);
    
    // Job PDA  
    let job_id = 42u64;
    let (job_pda, _) = find_job_pda(&authority, job_id);
    let job_seeds = &[b"job", authority.as_ref(), &job_id.to_le_bytes()];
    let (expected_job_pda, _) = Pubkey::find_program_address(job_seeds, &PROGRAM_ID);
    assert_eq!(job_pda, expected_job_pda);
    
    // Team PDA
    let (team_pda, _) = find_team_pda(&authority);
    let team_seeds = &[b"team", authority.as_ref()];
    let (expected_team_pda, _) = Pubkey::find_program_address(team_seeds, &PROGRAM_ID);
    assert_eq!(team_pda, expected_team_pda);
    
    // Config PDA
    let (config_pda, _) = find_config_pda();
    let config_seeds = &[b"config"];
    let (expected_config_pda, _) = Pubkey::find_program_address(config_seeds, &PROGRAM_ID);
    assert_eq!(config_pda, expected_config_pda);
    
    // Arbiter Pool PDA
    let (arbiter_pool_pda, _) = find_arbiter_pool_pda();
    let arbiter_seeds = &[b"arbiter_pool"];
    let (expected_arbiter_pool_pda, _) = Pubkey::find_program_address(arbiter_seeds, &PROGRAM_ID);
    assert_eq!(arbiter_pool_pda, expected_arbiter_pool_pda);
    
    // Dispute PDA
    let (dispute_pda, _) = find_dispute_pda(&job);
    let dispute_seeds = &[b"dispute", job.as_ref()];
    let (expected_dispute_pda, _) = Pubkey::find_program_address(dispute_seeds, &PROGRAM_ID);
    assert_eq!(dispute_pda, expected_dispute_pda);
    
    // Milestone PDA
    let index = 5u8;
    let (milestone_pda, _) = find_milestone_pda(&job, index);
    let milestone_seeds = &[b"milestone", job.as_ref(), &[index]];
    let (expected_milestone_pda, _) = Pubkey::find_program_address(milestone_seeds, &PROGRAM_ID);
    assert_eq!(milestone_pda, expected_milestone_pda);
}

// ===== PERFORMANCE TESTS =====

#[tokio::test]
async fn test_pda_derivation_performance() {
    let iterations = 1000;
    
    let duration = TestPerformanceUtils::benchmark_pda_derivation(iterations).await;
    
    // Should derive 1000 PDAs in less than 1 second
    assert!(
        duration.as_millis() < 1000, 
        "PDA derivation should be fast: {}ms for {} iterations",
        duration.as_millis(),
        iterations
    );
    
    let per_operation = duration.as_micros() as f64 / (iterations * 2) as f64; // 2 PDAs per iteration
    println!("PDA derivation performance: {:.2}μs per operation", per_operation);
    
    // Should be less than 1ms per PDA derivation
    assert!(per_operation < 1000.0, "Each PDA derivation should be <1ms");
}

#[test]
fn test_bulk_pda_generation() {
    let start = std::time::Instant::now();
    
    // Generate many PDAs to test performance
    for i in 0..1000 {
        let authority = Keypair::new().pubkey();
        let _user_pda = find_user_pda(&authority);
        let _team_pda = find_team_pda(&authority);
        let _job_pda = find_job_pda(&authority, i as u64);
    }
    
    let duration = start.elapsed();
    println!("Generated 3000 PDAs in {:?}", duration);
    
    // Should complete bulk generation quickly
    assert!(duration.as_secs() < 5, "Bulk PDA generation should be fast");
}

// ===== PROPERTY-BASED TESTS =====

proptest! {
    #[test]
    fn test_user_pda_deterministic_property(authority_bytes in any::<[u8; 32]>()) {
        let authority = Pubkey::new_from_array(authority_bytes);
        
        let (pda1, bump1) = find_user_pda(&authority);
        let (pda2, bump2) = find_user_pda(&authority);
        
        prop_assert_eq!(pda1, pda2);
        prop_assert_eq!(bump1, bump2);
        prop_assert_ne!(pda1, authority);
    }
    
    #[test] 
    fn test_job_pda_deterministic_property(
        client_bytes in any::<[u8; 32]>(),
        job_id in any::<u64>()
    ) {
        let client = Pubkey::new_from_array(client_bytes);
        
        let (pda1, bump1) = find_job_pda(&client, job_id);
        let (pda2, bump2) = find_job_pda(&client, job_id);
        
        prop_assert_eq!(pda1, pda2);
        prop_assert_eq!(bump1, bump2);
        prop_assert_ne!(pda1, client);
    }
    
    #[test]
    fn test_milestone_pda_deterministic_property(
        job_bytes in any::<[u8; 32]>(),
        index in any::<u8>()
    ) {
        let job = Pubkey::new_from_array(job_bytes);
        
        let (pda1, bump1) = find_milestone_pda(&job, index);
        let (pda2, bump2) = find_milestone_pda(&job, index);
        
        prop_assert_eq!(pda1, pda2);
        prop_assert_eq!(bump1, bump2);
        prop_assert_ne!(pda1, job);
    }
    
    #[test]
    fn test_job_pdas_unique_property(
        client_bytes in any::<[u8; 32]>(),
        job_id1 in any::<u64>(),
        job_id2 in any::<u64>()
    ) {
        prop_assume!(job_id1 != job_id2);
        
        let client = Pubkey::new_from_array(client_bytes);
        
        let (pda1, _) = find_job_pda(&client, job_id1);
        let (pda2, _) = find_job_pda(&client, job_id2);
        
        prop_assert_ne!(pda1, pda2);
    }
}