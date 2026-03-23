//! Performance benchmarks for PDA derivation
//!
//! These benchmarks measure the performance of Program Derived Address
//! derivation operations to ensure they meet performance targets.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use solana_sdk::{pubkey::Pubkey, signature::Keypair, signer::Signer};
use trust_escrow_sdk::pda::*;

/// Benchmark PDA derivation for different account types
fn benchmark_pda_derivation(c: &mut Criterion) {
    let mut group = c.benchmark_group("pda_derivation");

    // Test with different batch sizes
    for size in [1, 10, 100, 1000].iter() {
        group.throughput(Throughput::Elements(*size as u64));

        group.bench_with_input(BenchmarkId::new("user_pda", size), size, |b, &size| {
            b.iter(|| {
                for _ in 0..size {
                    let authority = Keypair::new().pubkey();
                    let _ = black_box(find_user_pda(&authority));
                }
            });
        });

        group.bench_with_input(BenchmarkId::new("job_pda", size), size, |b, &size| {
            b.iter(|| {
                for i in 0..size {
                    let client = Keypair::new().pubkey();
                    let job_id = i as u64;
                    let _ = black_box(find_job_pda(&client, job_id));
                }
            });
        });

        group.bench_with_input(BenchmarkId::new("team_pda", size), size, |b, &size| {
            b.iter(|| {
                for _ in 0..size {
                    let owner = Keypair::new().pubkey();
                    let _ = black_box(find_team_pda(&owner));
                }
            });
        });
    }

    group.finish();
}

/// Benchmark different PDA types with the same parameters
fn benchmark_pda_types(c: &mut Criterion) {
    let mut group = c.benchmark_group("pda_types");

    let authority = Keypair::new().pubkey();
    let job = Keypair::new().pubkey();

    group.bench_function("user_pda_single", |b| {
        b.iter(|| {
            let _ = black_box(find_user_pda(&authority));
        });
    });

    group.bench_function("job_pda_single", |b| {
        b.iter(|| {
            let _ = black_box(find_job_pda(&authority, 42));
        });
    });

    group.bench_function("team_pda_single", |b| {
        b.iter(|| {
            let _ = black_box(find_team_pda(&authority));
        });
    });

    group.bench_function("dispute_pda_single", |b| {
        b.iter(|| {
            let _ = black_box(find_dispute_pda(&job));
        });
    });

    group.bench_function("milestone_pda_single", |b| {
        b.iter(|| {
            let _ = black_box(find_milestone_pda(&job, 0));
        });
    });

    group.bench_function("config_pda_single", |b| {
        b.iter(|| {
            let _ = black_box(find_config_pda());
        });
    });

    group.bench_function("arbiter_pool_pda_single", |b| {
        b.iter(|| {
            let _ = black_box(find_arbiter_pool_pda());
        });
    });

    group.finish();
}

/// Benchmark PDA derivation with varying job IDs
fn benchmark_job_id_variance(c: &mut Criterion) {
    let mut group = c.benchmark_group("job_id_variance");

    let client = Keypair::new().pubkey();

    // Test with different job ID ranges
    let job_id_ranges = [
        ("small_ids", 0u64..10u64),
        ("medium_ids", 1000u64..1010u64),
        ("large_ids", 1_000_000u64..1_000_010u64),
        ("max_ids", (u64::MAX - 10)..u64::MAX),
    ];

    for (name, range) in job_id_ranges {
        group.bench_with_input(
            BenchmarkId::new("job_pda_range", name),
            &range.clone().collect::<Vec<_>>(),
            |b, job_ids| {
                b.iter(|| {
                    for &job_id in job_ids {
                        let _ = black_box(find_job_pda(&client, job_id));
                    }
                });
            },
        );
    }

    group.finish();
}

/// Benchmark milestone PDA derivation with different indices
fn benchmark_milestone_indices(c: &mut Criterion) {
    let mut group = c.benchmark_group("milestone_indices");

    let job = Keypair::new().pubkey();

    // Test all possible milestone indices (0-19)
    for index in 0u8..20u8 {
        group.bench_with_input(
            BenchmarkId::new("milestone_pda", index),
            &index,
            |b, &index| {
                b.iter(|| {
                    let _ = black_box(find_milestone_pda(&job, index));
                });
            },
        );
    }

    group.finish();
}

/// Benchmark PDA derivation memory usage patterns
fn benchmark_pda_memory_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_patterns");

    // Pre-allocate many keypairs to test memory access patterns
    let keypairs: Vec<Pubkey> = (0..1000).map(|_| Keypair::new().pubkey()).collect();

    group.bench_function("sequential_access", |b| {
        b.iter(|| {
            for pubkey in &keypairs {
                let _ = black_box(find_user_pda(pubkey));
            }
        });
    });

    group.bench_function("random_access", |b| {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        b.iter(|| {
            for _ in 0..1000 {
                let index = rng.gen_range(0..keypairs.len());
                let _ = black_box(find_user_pda(&keypairs[index]));
            }
        });
    });

    group.finish();
}

/// Benchmark PDA derivation with concurrent operations
fn benchmark_concurrent_pda_derivation(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_derivation");

    group.bench_function("parallel_user_pdas", |b| {
        b.iter(|| {
            use rayon::prelude::*;

            (0..100).into_par_iter().for_each(|_| {
                let authority = Keypair::new().pubkey();
                let _ = black_box(find_user_pda(&authority));
            });
        });
    });

    group.finish();
}

/// Benchmark comparison with direct Pubkey operations
fn benchmark_comparison_baseline(c: &mut Criterion) {
    let mut group = c.benchmark_group("comparison_baseline");

    let authority = Keypair::new().pubkey();

    // Baseline: Just creating pubkeys
    group.bench_function("pubkey_generation", |b| {
        b.iter(|| {
            let _ = black_box(Keypair::new().pubkey());
        });
    });

    // Baseline: Direct find_program_address call
    group.bench_function("direct_find_program_address", |b| {
        b.iter(|| {
            let seeds = &[b"user", authority.as_ref()];
            let _ = black_box(Pubkey::find_program_address(
                seeds,
                &trust_escrow_sdk::PROGRAM_ID,
            ));
        });
    });

    // Our optimized version
    group.bench_function("optimized_user_pda", |b| {
        b.iter(|| {
            let _ = black_box(find_user_pda(&authority));
        });
    });

    group.finish();
}

/// Benchmark PDA derivation error handling
fn benchmark_error_scenarios(c: &mut Criterion) {
    let mut group = c.benchmark_group("error_scenarios");

    // All our PDA functions should succeed, but test edge cases

    group.bench_function("zero_pubkey", |b| {
        let zero_pubkey = Pubkey::default();
        b.iter(|| {
            let _ = black_box(find_user_pda(&zero_pubkey));
        });
    });

    group.bench_function("max_pubkey", |b| {
        let max_pubkey = Pubkey::new_from_array([255u8; 32]);
        b.iter(|| {
            let _ = black_box(find_user_pda(&max_pubkey));
        });
    });

    group.bench_function("alternating_pubkey", |b| {
        let alternating_pubkey = Pubkey::new_from_array([0xAA; 32]);
        b.iter(|| {
            let _ = black_box(find_user_pda(&alternating_pubkey));
        });
    });

    group.finish();
}

/// Benchmark realistic usage patterns
fn benchmark_realistic_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("realistic_usage");

    // Simulate a typical escrow creation flow
    group.bench_function("escrow_creation_flow", |b| {
        b.iter(|| {
            let client = Keypair::new().pubkey();
            let freelancer = Keypair::new().pubkey();
            let job_id = 42u64;

            // Derive all PDAs needed for escrow creation
            let _ = black_box(find_user_pda(&client));
            let _ = black_box(find_user_pda(&freelancer));
            let _ = black_box(find_job_pda(&client, job_id));
            let _ = black_box(find_config_pda());
        });
    });

    // Simulate a milestone-heavy project
    group.bench_function("milestone_project_flow", |b| {
        b.iter(|| {
            let client = Keypair::new().pubkey();
            let job_id = 1u64;

            let job_pda = black_box(find_job_pda(&client, job_id));

            // Create 10 milestones
            for i in 0u8..10u8 {
                let _ = black_box(find_milestone_pda(&job_pda.0, i));
            }
        });
    });

    // Simulate dispute resolution flow
    group.bench_function("dispute_resolution_flow", |b| {
        b.iter(|| {
            let client = Keypair::new().pubkey();
            let job_id = 1u64;

            let job_pda = black_box(find_job_pda(&client, job_id));
            let _ = black_box(find_dispute_pda(&job_pda.0));
            let _ = black_box(find_arbiter_pool_pda());
            let _ = black_box(find_config_pda());
        });
    });

    group.finish();
}

criterion_group!(
    pda_benches,
    benchmark_pda_derivation,
    benchmark_pda_types,
    benchmark_job_id_variance,
    benchmark_milestone_indices,
    benchmark_pda_memory_patterns,
    benchmark_concurrent_pda_derivation,
    benchmark_comparison_baseline,
    benchmark_error_scenarios,
    benchmark_realistic_usage
);

criterion_main!(pda_benches);
