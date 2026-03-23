//! Performance benchmarks for client operations
//!
//! These benchmarks measure the performance of CofreClient operations
//! to ensure they meet performance targets and identify bottlenecks.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::sync::Arc;
use std::time::Duration;

use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig, pubkey::Pubkey, signature::Keypair, signer::Signer,
};

use trust_escrow_sdk::{types::*, CofreClient};

/// Setup for benchmarks - creates mock client
fn setup_benchmark_client() -> CofreClient {
    // Use a mock URL since we're benchmarking client-side operations
    let rpc_client = Arc::new(RpcClient::new_with_commitment(
        "http://localhost:8899".to_string(),
        CommitmentConfig::confirmed(),
    ));
    let payer = Arc::new(Keypair::new());

    CofreClient::new(rpc_client, payer).expect("Failed to create client")
}

/// Benchmark client creation and initialization
fn benchmark_client_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("client_creation");

    group.bench_function("create_client", |b| {
        b.iter(|| {
            let _ = black_box(setup_benchmark_client());
        });
    });

    group.bench_function("create_client_with_keypair", |b| {
        let keypair = Keypair::new();
        b.iter(|| {
            let rpc_client = Arc::new(RpcClient::new_with_commitment(
                "http://localhost:8899".to_string(),
                CommitmentConfig::confirmed(),
            ));
            let payer = Arc::new(keypair);

            let _ = black_box(CofreClient::new(rpc_client, payer));
        });
    });

    group.finish();
}

/// Benchmark transaction preparation (without sending)
fn benchmark_transaction_preparation(c: &mut Criterion) {
    let mut group = c.benchmark_group("transaction_preparation");
    let client = setup_benchmark_client();

    // Test various operation types
    group.bench_function("prepare_create_user", |b| {
        b.iter(|| {
            // This creates the transaction but doesn't send it (will fail at send)
            let rt = tokio::runtime::Runtime::new().unwrap();
            let _ = rt.block_on(async {
                // The actual benchmarking would test transaction building
                // For now, we test the client method call overhead
                client.create_user("benchmark_user", None).await
            });
        });
    });

    group.bench_function("prepare_create_job", |b| {
        b.iter(|| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let _ = rt.block_on(async {
                client
                    .create_job(
                        "Benchmark Job",
                        "Description",
                        1_000_000,
                        Duration::from_secs(86400),
                        false,
                    )
                    .await
            });
        });
    });

    group.finish();
}

/// Benchmark PDA derivation within client operations
fn benchmark_client_pda_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("client_pda_operations");

    let client = setup_benchmark_client();
    let user_key = client.payer().pubkey();

    // Benchmark operations that involve PDA derivation
    group.bench_function("derive_user_pda", |b| {
        b.iter(|| {
            let _ = black_box(trust_escrow_sdk::pda::find_user_pda(&user_key));
        });
    });

    group.bench_function("derive_job_pda", |b| {
        b.iter(|| {
            let _ = black_box(trust_escrow_sdk::pda::find_job_pda(&user_key, 42));
        });
    });

    group.bench_function("batch_pda_derivation", |b| {
        b.iter(|| {
            // Simulate deriving all PDAs for a complex operation
            let _ = black_box(trust_escrow_sdk::pda::find_user_pda(&user_key));
            let _ = black_box(trust_escrow_sdk::pda::find_job_pda(&user_key, 1));
            let _ = black_box(trust_escrow_sdk::pda::find_config_pda());
            let _ = black_box(trust_escrow_sdk::pda::find_arbiter_pool_pda());
        });
    });

    group.finish();
}

/// Benchmark data validation operations
fn benchmark_validation_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("validation_operations");

    let usernames = ["alice", "bob", "charlie", "diana", "eve"];
    let amounts = [100_000u64, 1_000_000, 5_000_000, 10_000_000];

    group.bench_function("validate_usernames", |b| {
        b.iter(|| {
            for username in &usernames {
                let _ = black_box(trust_escrow_sdk::utils::ValidationUtils::validate_username(
                    username,
                ));
            }
        });
    });

    group.bench_function("validate_amounts", |b| {
        b.iter(|| {
            for &amount in &amounts {
                let _ = black_box(
                    trust_escrow_sdk::utils::ValidationUtils::validate_job_amount(amount),
                );
            }
        });
    });

    group.bench_function("validate_evidence", |b| {
        let evidence = "This is some evidence for a dispute resolution case.";
        b.iter(|| {
            let _ = black_box(trust_escrow_sdk::utils::ValidationUtils::validate_evidence(
                evidence,
            ));
        });
    });

    group.finish();
}

/// Benchmark conversion operations
fn benchmark_conversion_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("conversion_operations");

    let sol_amounts = [0.001f64, 0.1, 1.0, 10.0, 100.0];
    let lamport_amounts = [100_000u64, 100_000_000, 1_000_000_000, 10_000_000_000];

    group.bench_function("sol_to_lamports", |b| {
        b.iter(|| {
            for &sol in &sol_amounts {
                let _ = black_box(trust_escrow_sdk::utils::ConversionUtils::sol_to_lamports(
                    sol,
                ));
            }
        });
    });

    group.bench_function("lamports_to_sol", |b| {
        b.iter(|| {
            for &lamports in &lamport_amounts {
                let _ = black_box(trust_escrow_sdk::utils::ConversionUtils::lamports_to_sol(
                    lamports,
                ));
            }
        });
    });

    group.bench_function("roundtrip_conversion", |b| {
        b.iter(|| {
            for &sol in &sol_amounts {
                let lamports = black_box(
                    trust_escrow_sdk::utils::ConversionUtils::sol_to_lamports(sol),
                );
                let _ = black_box(trust_escrow_sdk::utils::ConversionUtils::lamports_to_sol(
                    lamports,
                ));
            }
        });
    });

    group.finish();
}

/// Benchmark formatting operations
fn benchmark_formatting_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("formatting_operations");

    let lamport_amounts = [1u64, 1_000, 1_000_000, 1_000_000_000, 10_000_000_000];
    let durations = [
        Duration::from_secs(60),
        Duration::from_secs(3600),
        Duration::from_secs(86400),
        Duration::from_secs(604800),
    ];

    group.bench_function("format_lamports", |b| {
        b.iter(|| {
            for &amount in &lamport_amounts {
                let _ = black_box(trust_escrow_sdk::utils::FormattingUtils::format_lamports(
                    amount,
                ));
            }
        });
    });

    group.bench_function("format_lamports_compact", |b| {
        b.iter(|| {
            for &amount in &lamport_amounts {
                let _ = black_box(
                    trust_escrow_sdk::utils::FormattingUtils::format_lamports_compact(amount),
                );
            }
        });
    });

    group.bench_function("format_duration", |b| {
        b.iter(|| {
            for duration in &durations {
                let _ = black_box(trust_escrow_sdk::utils::FormattingUtils::format_duration(
                    *duration,
                ));
            }
        });
    });

    group.finish();
}

/// Benchmark math utilities
fn benchmark_math_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("math_operations");

    let amounts = [100_000u64, 1_000_000, 5_000_000, 10_000_000];
    let percentages = [10u8, 25, 50, 75, 100];

    group.bench_function("calculate_fee", |b| {
        b.iter(|| {
            for &amount in &amounts {
                for &percentage in &percentages {
                    let fee_basis_points = (percentage as u16) * 100;
                    let _ = black_box(trust_escrow_sdk::utils::MathUtils::calculate_fee(
                        amount,
                        fee_basis_points,
                    ));
                }
            }
        });
    });

    group.bench_function("calculate_percentage", |b| {
        b.iter(|| {
            for &amount in &amounts {
                for &percentage in &percentages {
                    let _ = black_box(trust_escrow_sdk::utils::MathUtils::calculate_percentage(
                        amount, percentage,
                    ));
                }
            }
        });
    });

    group.bench_function("safe_math_operations", |b| {
        b.iter(|| {
            for &amount in &amounts {
                let _ = black_box(trust_escrow_sdk::utils::MathUtils::safe_add(amount, 1000));
                let _ = black_box(trust_escrow_sdk::utils::MathUtils::safe_sub(amount, 1000));
                let _ = black_box(trust_escrow_sdk::utils::MathUtils::safe_mul(amount, 2));
            }
        });
    });

    group.finish();
}

/// Benchmark batch operations
fn benchmark_batch_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_operations");

    let client = setup_benchmark_client();

    // Test with different batch sizes
    for size in [1, 5, 10, 20].iter() {
        group.throughput(Throughput::Elements(*size as u64));

        group.bench_with_input(
            BenchmarkId::new("batch_milestone_creation", size),
            size,
            |b, &size| {
                let milestones: Vec<MilestoneData> = (0..size)
                    .map(|i| MilestoneData {
                        title: format!("Milestone {}", i),
                        description: format!("Description {}", i),
                        amount: 100_000 + (i as u64 * 50_000),
                        deadline_duration: Duration::from_secs(86400 * (i as u64 + 1)),
                    })
                    .collect();

                b.iter(|| {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    let _ = rt.block_on(async {
                        client.batch_create_milestones(1, milestones.clone()).await
                    });
                });
            },
        );
    }

    group.finish();
}

/// Benchmark client memory usage patterns
fn benchmark_memory_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_patterns");

    group.bench_function("multiple_clients", |b| {
        b.iter(|| {
            // Create multiple clients to test memory allocation patterns
            let clients: Vec<CofreClient> = (0..10)
                .map(|_| black_box(setup_benchmark_client()))
                .collect();

            // Use the clients briefly
            for client in &clients {
                let _ = black_box(client.payer().pubkey());
            }

            drop(clients);
        });
    });

    group.bench_function("client_reuse", |b| {
        let client = setup_benchmark_client();

        b.iter(|| {
            // Reuse the same client for multiple operations
            for i in 0..10 {
                let username = format!("user_{}", i);
                let rt = tokio::runtime::Runtime::new().unwrap();
                let _ = rt.block_on(async { client.create_user(&username, None).await });
            }
        });
    });

    group.finish();
}

/// Benchmark concurrent client operations
fn benchmark_concurrent_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_operations");

    group.bench_function("parallel_pda_derivation", |b| {
        let client = setup_benchmark_client();
        let user_key = client.payer().pubkey();

        b.iter(|| {
            use rayon::prelude::*;

            (0..100).into_par_iter().for_each(|i| {
                let _ = black_box(trust_escrow_sdk::pda::find_job_pda(&user_key, i as u64));
            });
        });
    });

    group.bench_function("parallel_validation", |b| {
        let usernames: Vec<String> = (0..100).map(|i| format!("user_{}", i)).collect();

        b.iter(|| {
            use rayon::prelude::*;

            usernames.par_iter().for_each(|username| {
                let _ = black_box(trust_escrow_sdk::utils::ValidationUtils::validate_username(
                    username,
                ));
            });
        });
    });

    group.finish();
}

/// Benchmark error handling performance
fn benchmark_error_handling(c: &mut Criterion) {
    let mut group = c.benchmark_group("error_handling");

    group.bench_function("validation_errors", |b| {
        let invalid_usernames = ["", "user with spaces", "🚀user", &"a".repeat(100)];

        b.iter(|| {
            for username in &invalid_usernames {
                let result = trust_escrow_sdk::utils::ValidationUtils::validate_username(username);
                let _ = black_box(result.is_err());
            }
        });
    });

    group.bench_function("amount_errors", |b| {
        let invalid_amounts = [0u64, 99_999, u64::MAX];

        b.iter(|| {
            for &amount in &invalid_amounts {
                let result = trust_escrow_sdk::utils::ValidationUtils::validate_job_amount(amount);
                // Some of these should pass, some should fail
                let _ = black_box(result.is_ok());
            }
        });
    });

    group.finish();
}

/// Benchmark realistic workflow scenarios
fn benchmark_realistic_workflows(c: &mut Criterion) {
    let mut group = c.benchmark_group("realistic_workflows");

    let client = setup_benchmark_client();

    group.bench_function("full_escrow_creation", |b| {
        b.iter(|| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let _ = rt.block_on(async {
                // Simulate full escrow creation workflow
                let user_creation = client.create_user("workflow_user", Some("Test")).await;
                let job_creation = client
                    .create_job(
                        "Workflow Job",
                        "Test job for workflow",
                        2_000_000,
                        Duration::from_secs(86400),
                        false,
                    )
                    .await;
                let fund_result = client.fund_escrow(1).await;

                // All will fail without validator, but we measure the overhead
                (user_creation, job_creation, fund_result)
            });
        });
    });

    group.bench_function("milestone_project_setup", |b| {
        b.iter(|| {
            let milestones = vec![
                MilestoneData {
                    title: "Setup".to_string(),
                    description: "Project setup".to_string(),
                    amount: 1_000_000,
                    deadline_duration: Duration::from_secs(86400 * 3),
                },
                MilestoneData {
                    title: "Development".to_string(),
                    description: "Core development".to_string(),
                    amount: 3_000_000,
                    deadline_duration: Duration::from_secs(86400 * 7),
                },
                MilestoneData {
                    title: "Testing".to_string(),
                    description: "Testing and QA".to_string(),
                    amount: 1_000_000,
                    deadline_duration: Duration::from_secs(86400 * 10),
                },
            ];

            let rt = tokio::runtime::Runtime::new().unwrap();
            let _ = rt.block_on(async { client.batch_create_milestones(1, milestones).await });
        });
    });

    group.finish();
}

criterion_group!(
    client_benches,
    benchmark_client_creation,
    benchmark_transaction_preparation,
    benchmark_client_pda_operations,
    benchmark_validation_operations,
    benchmark_conversion_operations,
    benchmark_formatting_operations,
    benchmark_math_operations,
    benchmark_batch_operations,
    benchmark_memory_patterns,
    benchmark_concurrent_operations,
    benchmark_error_handling,
    benchmark_realistic_workflows
);

criterion_main!(client_benches);
