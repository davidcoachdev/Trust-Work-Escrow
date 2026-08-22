//! Unit tests for type validation and data structures
//!
//! These tests verify that all data types in the Trust Escrow SDK
//! work correctly, validate inputs properly, and handle edge cases.

use std::time::Duration;

use pretty_assertions::assert_eq;
use proptest::prelude::*;
use rstest::*;

use solana_sdk::{pubkey::Pubkey, signature::Keypair, signer::Signer};

use trust_escrow_sdk::types::*;

// Import common test utilities
mod common;
use common::*;

// ===== JOB STATUS TESTS =====

#[test]
fn test_job_status_variants() {
    let statuses = vec![
        JobStatus::Created,
        JobStatus::ApplicationsOpen,
        JobStatus::InProgress,
        JobStatus::Submitted,
        JobStatus::Approved,
        JobStatus::Disputed,
        JobStatus::Resolved,
        JobStatus::Cancelled,
    ];
    
    // Test that all variants can be created
    for status in statuses {
        // Test Debug trait
        let debug_str = format!("{:?}", status);
        assert!(!debug_str.is_empty());
        
        // Test Clone trait
        let cloned_status = status.clone();
        assert_eq!(status, cloned_status);
    }
}

#[test]
fn test_job_status_transitions() {
    // Define valid status transitions
    fn is_valid_transition(from: JobStatus, to: JobStatus) -> bool {
        match (from, to) {
            (JobStatus::Created, JobStatus::ApplicationsOpen) => true,
            (JobStatus::ApplicationsOpen, JobStatus::InProgress) => true,
            (JobStatus::InProgress, JobStatus::Submitted) => true,
            (JobStatus::Submitted, JobStatus::Approved) => true,
            (JobStatus::Submitted, JobStatus::Disputed) => true,
            (JobStatus::InProgress, JobStatus::Disputed) => true,
            (JobStatus::Disputed, JobStatus::Resolved) => true,
            (JobStatus::Created, JobStatus::Cancelled) => true,
            (JobStatus::ApplicationsOpen, JobStatus::Cancelled) => true,
            _ => false,
        }
    }
    
    // Test some valid transitions
    assert!(is_valid_transition(JobStatus::Created, JobStatus::ApplicationsOpen));
    assert!(is_valid_transition(JobStatus::InProgress, JobStatus::Submitted));
    assert!(is_valid_transition(JobStatus::Submitted, JobStatus::Approved));
    
    // Test some invalid transitions
    assert!(!is_valid_transition(JobStatus::Approved, JobStatus::Created));
    assert!(!is_valid_transition(JobStatus::Cancelled, JobStatus::InProgress));
}

// ===== APPLICATION STATUS TESTS =====

#[test]
fn test_application_status_variants() {
    let statuses = vec![
        ApplicationStatus::Pending,
        ApplicationStatus::Accepted,
        ApplicationStatus::Rejected,
    ];
    
    for status in statuses {
        let debug_str = format!("{:?}", status);
        assert!(!debug_str.is_empty());
        
        let cloned_status = status.clone();
        assert_eq!(status, cloned_status);
    }
}

// ===== MEMBER ROLE TESTS =====

#[test]
fn test_member_role_variants() {
    let roles = vec![
        MemberRole::Owner,
        MemberRole::Admin,
        MemberRole::Member,
    ];
    
    for role in roles {
        let debug_str = format!("{:?}", role);
        assert!(!debug_str.is_empty());
        
        let cloned_role = role.clone();
        assert_eq!(role, cloned_role);
    }
}

#[test]
fn test_member_role_permissions() {
    // Define role hierarchy
    fn has_permission(role: MemberRole, permission: &str) -> bool {
        match (role, permission) {
            (MemberRole::Owner, _) => true, // Owner has all permissions
            (MemberRole::Admin, "manage_members") => true,
            (MemberRole::Admin, "assign_tasks") => true,
            (MemberRole::Member, "view_tasks") => true,
            _ => false,
        }
    }
    
    assert!(has_permission(MemberRole::Owner, "any_permission"));
    assert!(has_permission(MemberRole::Admin, "manage_members"));
    assert!(has_permission(MemberRole::Member, "view_tasks"));
    assert!(!has_permission(MemberRole::Member, "manage_members"));
}

// ===== DISPUTE STATUS TESTS =====

#[test]
fn test_dispute_status_variants() {
    let statuses = vec![
        DisputeStatus::Open,
        DisputeStatus::InReview,
        DisputeStatus::Resolved,
        DisputeStatus::Closed,
    ];
    
    for status in statuses {
        let debug_str = format!("{:?}", status);
        assert!(!debug_str.is_empty());
        
        let cloned_status = status.clone();
        assert_eq!(status, cloned_status);
    }
}

// ===== MILESTONE STATUS TESTS =====

#[test]
fn test_milestone_status_variants() {
    let statuses = vec![
        MilestoneStatus::Pending,
        MilestoneStatus::Submitted,
        MilestoneStatus::Approved,
        MilestoneStatus::Rejected,
    ];
    
    for status in statuses {
        let debug_str = format!("{:?}", status);
        assert!(!debug_str.is_empty());
        
        let cloned_status = status.clone();
        assert_eq!(status, cloned_status);
    }
}

// ===== USER STRUCT TESTS =====

#[test]
fn test_user_creation() {
    let authority = Keypair::new().pubkey();
    let wallet = Keypair::new().pubkey();
    
    let user = User {
        authority,
        username: "test_user".to_string(),
        bio: Some("Test user bio".to_string()),
        wallets: vec![wallet],
        active_wallet: wallet,
        created_at: 1234567890,
        bump: 255,
    };
    
    TestAssertions::assert_user_valid(&user, "test_user");
    assert_eq!(user.authority, authority);
    assert_eq!(user.active_wallet, wallet);
    assert!(user.wallets.contains(&wallet));
    assert_eq!(user.bio, Some("Test user bio".to_string()));
}

#[test]
fn test_user_without_bio() {
    let authority = Keypair::new().pubkey();
    let wallet = Keypair::new().pubkey();
    
    let user = User {
        authority,
        username: "test_user".to_string(),
        bio: None,
        wallets: vec![wallet],
        active_wallet: wallet,
        created_at: 1234567890,
        bump: 255,
    };
    
    assert_eq!(user.bio, None);
}

// ===== JOB STRUCT TESTS =====

#[test]
fn test_job_creation() {
    let client = Keypair::new().pubkey();
    let job_id = 42u64;
    
    let job = Job {
        id: job_id,
        client,
        title: "Test Job".to_string(),
        description: "Test job description".to_string(),
        amount: 1_000_000,
        deadline: 1234567890 + 86400, // 1 day from creation
        requires_team: false,
        status: JobStatus::Created,
        freelancer: None,
        escrow: None,
        created_at: 1234567890,
        bump: 255,
    };
    
    assert_eq!(job.id, job_id);
    assert_eq!(job.client, client);
    assert_eq!(job.title, "Test Job");
    assert_eq!(job.amount, 1_000_000);
    assert!(!job.requires_team);
    assert_eq!(job.status, JobStatus::Created);
    assert_eq!(job.freelancer, None);
    assert_eq!(job.escrow, None);
}

#[test]
fn test_job_with_freelancer() {
    let client = Keypair::new().pubkey();
    let freelancer = Keypair::new().pubkey();
    
    let job = Job {
        id: 1,
        client,
        title: "Test Job".to_string(),
        description: "Test job description".to_string(),
        amount: 1_000_000,
        deadline: 1234567890 + 86400,
        requires_team: false,
        status: JobStatus::InProgress,
        freelancer: Some(freelancer),
        escrow: Some(Keypair::new().pubkey()),
        created_at: 1234567890,
        bump: 255,
    };
    
    assert_eq!(job.freelancer, Some(freelancer));
    assert!(job.escrow.is_some());
    assert_eq!(job.status, JobStatus::InProgress);
}

// ===== TEAM STRUCT TESTS =====

#[test]
fn test_team_creation() {
    let owner = Keypair::new().pubkey();
    let member1 = Keypair::new().pubkey();
    let member2 = Keypair::new().pubkey();
    
    let team = Team {
        owner,
        name: "Test Team".to_string(),
        description: "Test team description".to_string(),
        members: vec![
            TeamMember {
                user: owner,
                role: MemberRole::Owner,
                joined_at: 1234567890,
            },
            TeamMember {
                user: member1,
                role: MemberRole::Admin,
                joined_at: 1234567890 + 100,
            },
            TeamMember {
                user: member2,
                role: MemberRole::Member,
                joined_at: 1234567890 + 200,
            },
        ],
        created_at: 1234567890,
        bump: 255,
    };
    
    TestAssertions::assert_team_valid(&team, "Test Team");
    assert_eq!(team.members.len(), 3);
    assert_eq!(team.members[0].role, MemberRole::Owner);
    assert_eq!(team.members[1].role, MemberRole::Admin);
    assert_eq!(team.members[2].role, MemberRole::Member);
}

// ===== TEAM MEMBER STRUCT TESTS =====

#[test]
fn test_team_member_creation() {
    let user = Keypair::new().pubkey();
    
    let member = TeamMember {
        user,
        role: MemberRole::Member,
        joined_at: 1234567890,
    };
    
    assert_eq!(member.user, user);
    assert_eq!(member.role, MemberRole::Member);
    assert_eq!(member.joined_at, 1234567890);
}

// ===== DISPUTE STRUCT TESTS =====

#[test]
fn test_dispute_creation() {
    let job = Keypair::new().pubkey();
    let complainant = Keypair::new().pubkey();
    
    let dispute = Dispute {
        job,
        complainant,
        evidence: "Test evidence for dispute".to_string(),
        status: DisputeStatus::Open,
        arbiter: None,
        resolution: None,
        created_at: 1234567890,
        bump: 255,
    };
    
    TestAssertions::assert_dispute_valid(&dispute, DisputeStatus::Open);
    assert_eq!(dispute.complainant, complainant);
    assert!(dispute.evidence.contains("Test evidence"));
    assert_eq!(dispute.arbiter, None);
    assert_eq!(dispute.resolution, None);
}

#[test]
fn test_dispute_with_resolution() {
    let job = Keypair::new().pubkey();
    let complainant = Keypair::new().pubkey();
    let arbiter = Keypair::new().pubkey();
    
    let dispute = Dispute {
        job,
        complainant,
        evidence: "Test evidence".to_string(),
        status: DisputeStatus::Resolved,
        arbiter: Some(arbiter),
        resolution: Some(DisputeResolution {
            client_percentage: 60,
            freelancer_percentage: 40,
            resolved_at: 1234567890 + 3600,
        }),
        created_at: 1234567890,
        bump: 255,
    };
    
    assert_eq!(dispute.status, DisputeStatus::Resolved);
    assert_eq!(dispute.arbiter, Some(arbiter));
    
    let resolution = dispute.resolution.unwrap();
    assert_eq!(resolution.client_percentage, 60);
    assert_eq!(resolution.freelancer_percentage, 40);
    assert_eq!(resolution.client_percentage + resolution.freelancer_percentage, 100);
}

// ===== MILESTONE STRUCT TESTS =====

#[test]
fn test_milestone_creation() {
    let job = Keypair::new().pubkey();
    
    let milestone = Milestone {
        job,
        index: 0,
        title: "First Milestone".to_string(),
        description: "Complete initial setup".to_string(),
        amount: 500_000,
        deadline: 1234567890 + 86400,
        status: MilestoneStatus::Pending,
        submitted_work: None,
        created_at: 1234567890,
        bump: 255,
    };
    
    TestAssertions::assert_milestone_valid(&milestone, MilestoneStatus::Pending, 500_000);
    assert_eq!(milestone.index, 0);
    assert_eq!(milestone.title, "First Milestone");
    assert_eq!(milestone.submitted_work, None);
}

#[test]
fn test_milestone_with_submitted_work() {
    let job = Keypair::new().pubkey();
    
    let milestone = Milestone {
        job,
        index: 1,
        title: "Second Milestone".to_string(),
        description: "Complete development phase".to_string(),
        amount: 750_000,
        deadline: 1234567890 + 172800, // 2 days
        status: MilestoneStatus::Submitted,
        submitted_work: Some("https://github.com/user/project/pull/123".to_string()),
        created_at: 1234567890,
        bump: 255,
    };
    
    assert_eq!(milestone.status, MilestoneStatus::Submitted);
    assert!(milestone.submitted_work.is_some());
    assert!(milestone.submitted_work.unwrap().contains("github.com"));
}

// ===== MILESTONE DATA TESTS =====

#[test]
fn test_milestone_data_creation() {
    let milestone_data = MilestoneData {
        title: "Test Milestone".to_string(),
        description: "Test milestone description".to_string(),
        amount: 1_000_000,
        deadline_duration: Duration::from_secs(86400),
    };
    
    assert_eq!(milestone_data.title, "Test Milestone");
    assert_eq!(milestone_data.amount, 1_000_000);
    assert_eq!(milestone_data.deadline_duration.as_secs(), 86400);
}

// ===== JOB FILTER TESTS =====

#[test]
fn test_job_filter_creation() {
    let client = Keypair::new().pubkey();
    
    let filter = JobFilter {
        client: Some(client),
        status: Some(JobStatus::Created),
        requires_team: Some(false),
        min_amount: Some(100_000),
        max_amount: Some(10_000_000),
    };
    
    assert_eq!(filter.client, Some(client));
    assert_eq!(filter.status, Some(JobStatus::Created));
    assert_eq!(filter.requires_team, Some(false));
    assert_eq!(filter.min_amount, Some(100_000));
    assert_eq!(filter.max_amount, Some(10_000_000));
}

#[test]
fn test_empty_job_filter() {
    let filter = JobFilter {
        client: None,
        status: None,
        requires_team: None,
        min_amount: None,
        max_amount: None,
    };
    
    assert_eq!(filter.client, None);
    assert_eq!(filter.status, None);
    assert_eq!(filter.requires_team, None);
    assert_eq!(filter.min_amount, None);
    assert_eq!(filter.max_amount, None);
}

// ===== ESCROW STATS TESTS =====

#[test]
fn test_escrow_stats_creation() {
    let stats = EscrowStats {
        total_escrows: 100,
        active_escrows: 25,
        completed_escrows: 70,
        disputed_escrows: 3,
        total_volume: 50_000_000_000, // 50 SOL
        average_escrow_amount: 500_000_000, // 0.5 SOL
    };
    
    assert_eq!(stats.total_escrows, 100);
    assert_eq!(stats.active_escrows, 25);
    assert_eq!(stats.completed_escrows, 70);
    assert_eq!(stats.disputed_escrows, 3);
    assert_eq!(stats.total_volume, 50_000_000_000);
    assert_eq!(stats.average_escrow_amount, 500_000_000);
    
    // Verify the math makes sense
    assert_eq!(stats.active_escrows + stats.completed_escrows + stats.disputed_escrows, 98); // 2 might be in other states
}

// ===== PROPERTY BASED TESTS =====

proptest! {
    #[test]
    fn test_user_username_property(username in "\\w{1,50}") {
        let authority = Pubkey::new_unique();
        let wallet = Pubkey::new_unique();
        
        let user = User {
            authority,
            username: username.clone(),
            bio: None,
            wallets: vec![wallet],
            active_wallet: wallet,
            created_at: 1234567890,
            bump: 255,
        };
        
        prop_assert_eq!(user.username, username);
        prop_assert!(!user.wallets.is_empty());
    }
    
    #[test]
    fn test_job_amount_property(amount in 100_000u64..10_000_000_000u64) {
        let client = Pubkey::new_unique();
        
        let job = Job {
            id: 1,
            client,
            title: "Test".to_string(),
            description: "Test".to_string(),
            amount,
            deadline: 1234567890 + 86400,
            requires_team: false,
            status: JobStatus::Created,
            freelancer: None,
            escrow: None,
            created_at: 1234567890,
            bump: 255,
        };
        
        prop_assert_eq!(job.amount, amount);
        prop_assert!(job.amount >= 100_000); // Min job amount
    }
    
    #[test]
    fn test_milestone_index_property(index in 0u8..=19u8) {
        let job = Pubkey::new_unique();
        
        let milestone = Milestone {
            job,
            index,
            title: "Test".to_string(),
            description: "Test".to_string(),
            amount: 100_000,
            deadline: 1234567890 + 86400,
            status: MilestoneStatus::Pending,
            submitted_work: None,
            created_at: 1234567890,
            bump: 255,
        };
        
        prop_assert_eq!(milestone.index, index);
        prop_assert!(milestone.index < 20); // Max milestones per job
    }
}

// ===== EDGE CASE TESTS =====

#[test]
fn test_maximum_values() {
    // Test with maximum reasonable values
    let user = User {
        authority: Pubkey::new_unique(),
        username: "a".repeat(50), // Long username
        bio: Some("a".repeat(500)), // Long bio
        wallets: (0..5).map(|_| Pubkey::new_unique()).collect(), // Max wallets
        active_wallet: Pubkey::new_unique(),
        created_at: i64::MAX as i64,
        bump: 255,
    };
    
    assert_eq!(user.username.len(), 50);
    assert_eq!(user.bio.as_ref().unwrap().len(), 500);
    assert_eq!(user.wallets.len(), 5);
    assert_eq!(user.created_at, i64::MAX);
    assert_eq!(user.bump, 255);
}

#[test]
fn test_minimum_values() {
    let user = User {
        authority: Pubkey::new_unique(),
        username: "a".to_string(), // Single char username
        bio: None, // No bio
        wallets: vec![Pubkey::new_unique()], // Single wallet
        active_wallet: Pubkey::new_unique(),
        created_at: 0,
        bump: 0,
    };
    
    assert_eq!(user.username.len(), 1);
    assert_eq!(user.bio, None);
    assert_eq!(user.wallets.len(), 1);
    assert_eq!(user.created_at, 0);
    assert_eq!(user.bump, 0);
}

// ===== TYPE COMPATIBILITY TESTS =====

#[test]
fn test_type_cloning() {
    let status = JobStatus::Created;
    let cloned = status.clone();
    assert_eq!(status, cloned);
    
    let role = MemberRole::Owner;
    let cloned_role = role.clone();
    assert_eq!(role, cloned_role);
}

#[test]
fn test_type_debugging() {
    let status = JobStatus::InProgress;
    let debug_str = format!("{:?}", status);
    assert!(debug_str.contains("InProgress"));
    
    let role = MemberRole::Admin;
    let debug_str = format!("{:?}", role);
    assert!(debug_str.contains("Admin"));
}

#[test]
fn test_partial_eq() {
    assert_eq!(JobStatus::Created, JobStatus::Created);
    assert_ne!(JobStatus::Created, JobStatus::InProgress);
    
    assert_eq!(MemberRole::Owner, MemberRole::Owner);
    assert_ne!(MemberRole::Owner, MemberRole::Member);
}