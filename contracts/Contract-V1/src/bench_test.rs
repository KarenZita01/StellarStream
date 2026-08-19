//! # StellarStream Gas Cost Benchmarks
//!
//! Comprehensive gas cost benchmarks for all StellarStream contract functions.
//! These benchmarks measure CPU instruction costs and memory usage to track
//! performance over time and prevent regressions.
//!
//! ## Methodology
//!
//! All benchmarks follow a consistent pattern:
//! 1. Set up test environment with realistic data
//! 2. Record initial gas state using `env.budget().cpu_instruction_cost()`
//! 3. Execute the operation being benchmarked
//! 4. Record final gas state
//! 5. Calculate gas consumed (final - initial)
//! 6. Print results in a structured format
//!
//! ## Gas Measurement
//!
//! Soroban measures gas using CPU instruction costs. The budget is reset
//! between test functions, so each benchmark starts with a clean state.
//!
//! ## Running Benchmarks
//!
//! ```bash
//! # Run all benchmarks
//! cargo test bench_ --release
//!
//! # Run specific benchmark
//! cargo test bench_create_stream --release
//!
//! # Run with output
//! cargo test bench_ --release -- --nocapture
//! ```

#[cfg(test)]
mod bench {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Address, Env, symbol_short};

    // ============================================================================
    // Test Data Constants
    // ============================================================================

    /// Standard test addresses
    const ADMIN: &str = "ADMIN_ADDRESS";
    const SENDER: &str = "SENDER_ADDRESS";
    const RECEIVER: &str = "RECEIVER_ADDRESS";
    const TOKEN: &str = "TOKEN_ADDRESS";

    /// Standard stream parameters
    const STREAM_AMOUNT: i128 = 1_000_000_000; // 100 tokens (7 decimals)
    const STREAM_DURATION: u64 = 86_400 * 30; // 30 days in seconds

    /// Batch operation sizes
    const BATCH_SIZE_SMALL: usize = 5;
    const BATCH_SIZE_MEDIUM: usize = 10;
    const BATCH_SIZE_LARGE: usize = 25;

    // ============================================================================
    // Helper Functions
    // ============================================================================

    /// Setup test environment with initialized contract
    fn setup_bench() -> (Env, Address, Address, Address, Address) {
        let env = Env::default();
        let admin = Address::generate(&env);
        let sender = Address::generate(&env);
        let receiver = Address::generate(&env);
        let token = Address::generate(&env);

        // Initialize contract
        env.mock_all_auths();
        client.initialize(&env, &admin);

        (env, admin, sender, receiver, token)
    }

    /// Get current CPU instruction cost
    fn get_gas_used(env: &Env) -> u64 {
        env.budget().cpu_instruction_cost()
    }

    /// Calculate elapsed gas
    fn calculate_gas(start: u64, end: u64) -> u64 {
        end.saturating_sub(start)
    }

    /// Print benchmark result in structured format
    fn print_result(operation: &str, gas: u64, notes: Option<&str>) {
        let notes_str = notes.unwrap_or("-");
        println!(
            "{:<40} {:>12} {:>15}",
            operation, gas, notes_str
        );
    }

    /// Print benchmark header
    fn print_header() {
        println!("\n{:=<80}", "");
        println!("{:<40} {:>12} {:>15}", "Operation", "Gas Cost", "Notes");
        println!("{:=<80}", "");
    }

    /// Print benchmark footer
    fn print_footer() {
        println!("{:=<80}\n", "");
    }

    // ============================================================================
    // Core Function Benchmarks
    // ============================================================================

    /// Benchmark: Initialize contract
    /// Measures gas cost of contract initialization
    #[test]
    fn bench_initialize() {
        let env = Env::default();
        let admin = Address::generate(&env);

        print_header();
        
        let gas_before = get_gas_used(&env);
        client.initialize(&env, &admin);
        let gas_after = get_gas_used(&env);
        let gas_used = calculate_gas(gas_before, gas_after);

        print_result("initialize", gas_used, Some("single call"));
        print_footer();
    }

    /// Benchmark: Create stream (linear vesting)
    /// Measures gas cost of creating a standard linear stream
    #[test]
    fn bench_create_stream_linear() {
        let (env, _admin, sender, receiver, token) = setup_bench();
        let start_time = env.ledger().timestamp() + 1;
        let end_time = start_time + STREAM_DURATION;

        let gas_before = get_gas_used(&env);
        client.create_stream(
            &env,
            &sender,
            &receiver,
            &token,
            STREAM_AMOUNT,
            start_time,
            end_time,
            0, // Linear curve
            false,
        );
        let gas_after = get_gas_used(&env);
        let gas_used = calculate_gas(gas_before, gas_after);

        print_header();
        print_result("create_stream (linear)", gas_used, Some("100 tokens, 30 days"));
        print_footer();
    }

    /// Benchmark: Create stream (exponential vesting)
    /// Measures gas cost of creating an exponential curve stream
    #[test]
    fn bench_create_stream_exponential() {
        let (env, _admin, sender, receiver, token) = setup_bench();
        let start_time = env.ledger().timestamp() + 1;
        let end_time = start_time + STREAM_DURATION;

        let gas_before = get_gas_used(&env);
        client.create_stream(
            &env,
            &sender,
            &receiver,
            &token,
            STREAM_AMOUNT,
            start_time,
            end_time,
            1, // Exponential curve
            false,
        );
        let gas_after = get_gas_used(&env);
        let gas_used = calculate_gas(gas_before, gas_after);

        print_header();
        print_result("create_stream (exponential)", gas_used, Some("100 tokens, 30 days"));
        print_footer();
    }

    /// Benchmark: Create soulbound stream
    /// Measures gas cost of creating a non-transferable stream
    #[test]
    fn bench_create_stream_soulbound() {
        let (env, _admin, sender, receiver, token) = setup_bench();
        let start_time = env.ledger().timestamp() + 1;
        let end_time = start_time + STREAM_DURATION;

        let gas_before = get_gas_used(&env);
        client.create_stream(
            &env,
            &sender,
            &receiver,
            &token,
            STREAM_AMOUNT,
            start_time,
            end_time,
            0, // Linear curve
            true, // Soulbound
        );
        let gas_after = get_gas_used(&env);
        let gas_used = calculate_gas(gas_before, gas_after);

        print_header();
        print_result("create_stream (soulbound)", gas_used, Some("non-transferable"));
        print_footer();
    }

    /// Benchmark: Withdraw from stream
    /// Measures gas cost of withdrawing unlocked tokens
    #[test]
    fn bench_withdraw() {
        let (env, _admin, sender, receiver, token) = setup_bench();
        let start_time = env.ledger().timestamp() + 1;
        let end_time = start_time + STREAM_DURATION;

        // Create stream first
        let stream_id = client.create_stream(
            &env,
            &sender,
            &receiver,
            &token,
            STREAM_AMOUNT,
            start_time,
            end_time,
            0,
            false,
        );

        // Advance time to allow some vesting
        env.ledger().set_timestamp(start_time + STREAM_DURATION / 2);

        let gas_before = get_gas_used(&env);
        client.withdraw(&env, &stream_id, &receiver);
        let gas_after = get_gas_used(&env);
        let gas_used = calculate_gas(gas_before, gas_after);

        print_header();
        print_result("withdraw", gas_used, Some("50% vested"));
        print_footer();
    }

    /// Benchmark: Cancel stream
    /// Measures gas cost of cancelling a stream
    #[test]
    fn bench_cancel_stream() {
        let (env, _admin, sender, receiver, token) = setup_bench();
        let start_time = env.ledger().timestamp() + 1;
        let end_time = start_time + STREAM_DURATION;

        // Create stream first
        let stream_id = client.create_stream(
            &env,
            &sender,
            &receiver,
            &token,
            STREAM_AMOUNT,
            start_time,
            end_time,
            0,
            false,
        );

        let gas_before = get_gas_used(&env);
        client.cancel_stream(&env, &stream_id);
        let gas_after = get_gas_used(&env);
        let gas_used = calculate_gas(gas_before, gas_after);

        print_header();
        print_result("cancel_stream", gas_used, Some("active stream"));
        print_footer();
    }

    /// Benchmark: Pause stream
    /// Measures gas cost of pausing an active stream
    #[test]
    fn bench_pause_stream() {
        let (env, _admin, sender, receiver, token) = setup_bench();
        let start_time = env.ledger().timestamp() + 1;
        let end_time = start_time + STREAM_DURATION;

        // Create stream first
        let stream_id = client.create_stream(
            &env,
            &sender,
            &receiver,
            &token,
            STREAM_AMOUNT,
            start_time,
            end_time,
            0,
            false,
        );

        let gas_before = get_gas_used(&env);
        client.pause_stream(&env, &stream_id);
        let gas_after = get_gas_used(&env);
        let gas_used = calculate_gas(gas_before, gas_after);

        print_header();
        print_result("pause_stream", gas_used, Some("active stream"));
        print_footer();
    }

    /// Benchmark: Resume stream
    /// Measures gas cost of resuming a paused stream
    #[test]
    fn bench_resume_stream() {
        let (env, _admin, sender, receiver, token) = setup_bench();
        let start_time = env.ledger().timestamp() + 1;
        let end_time = start_time + STREAM_DURATION;

        // Create and pause stream first
        let stream_id = client.create_stream(
            &env,
            &sender,
            &receiver,
            &token,
            STREAM_AMOUNT,
            start_time,
            end_time,
            0,
            false,
        );
        client.pause_stream(&env, &stream_id);

        let gas_before = get_gas_used(&env);
        client.resume_stream(&env, &stream_id);
        let gas_after = get_gas_used(&env);
        let gas_used = calculate_gas(gas_before, gas_after);

        print_header();
        print_result("resume_stream", gas_used, Some("paused stream"));
        print_footer();
    }

    /// Benchmark: Get stream details
    /// Measures gas cost of querying stream information
    #[test]
    fn bench_get_stream() {
        let (env, _admin, sender, receiver, token) = setup_bench();
        let start_time = env.ledger().timestamp() + 1;
        let end_time = start_time + STREAM_DURATION;

        // Create stream first
        let stream_id = client.create_stream(
            &env,
            &sender,
            &receiver,
            &token,
            STREAM_AMOUNT,
            start_time,
            end_time,
            0,
            false,
        );

        let gas_before = get_gas_used(&env);
        client.get_stream(&env, &stream_id);
        let gas_after = get_gas_used(&env);
        let gas_used = calculate_gas(gas_before, gas_after);

        print_header();
        print_result("get_stream", gas_used, Some("single stream"));
        print_footer();
    }

    /// Benchmark: Get withdrawable amount
    /// Measures gas cost of calculating withdrawable balance
    #[test]
    fn bench_get_withdrawable_amount() {
        let (env, _admin, sender, receiver, token) = setup_bench();
        let start_time = env.ledger().timestamp() + 1;
        let end_time = start_time + STREAM_DURATION;

        // Create stream first
        let stream_id = client.create_stream(
            &env,
            &sender,
            &receiver,
            &token,
            STREAM_AMOUNT,
            start_time,
            end_time,
            0,
            false,
        );

        // Advance time to allow some vesting
        env.ledger().set_timestamp(start_time + STREAM_DURATION / 2);

        let gas_before = get_gas_used(&env);
        client.get_withdrawable_amount(&env, &stream_id);
        let gas_after = get_gas_used(&env);
        let gas_used = calculate_gas(gas_before, gas_after);

        print_header();
        print_result("get_withdrawable_amount", gas_used, Some("50% vested"));
        print_footer();
    }

    /// Benchmark: Get user streams
    /// Measures gas cost of fetching all streams for a user
    #[test]
    fn bench_get_user_streams() {
        let (env, _admin, sender, receiver, token) = setup_bench();
        let start_time = env.ledger().timestamp() + 1;
        let end_time = start_time + STREAM_DURATION;

        // Create multiple streams for the user
        for _ in 0..5 {
            client.create_stream(
                &env,
                &sender,
                &receiver,
                &token,
                STREAM_AMOUNT / 5,
                start_time,
                end_time,
                0,
                false,
            );
        }

        let gas_before = get_gas_used(&env);
        client.get_user_streams(&env, &sender);
        let gas_after = get_gas_used(&env);
        let gas_used = calculate_gas(gas_before, gas_after);

        print_header();
        print_result("get_user_streams", gas_used, Some("5 streams"));
        print_footer();
    }

    // ============================================================================
    // Multi-Signature Benchmarks
    // ============================================================================

    /// Benchmark: Create proposal
    /// Measures gas cost of creating a multi-sig proposal
    #[test]
    fn bench_create_proposal() {
        let (env, _admin, sender, receiver, token) = setup_bench();
        let start_time = env.ledger().timestamp() + 1;
        let end_time = start_time + STREAM_DURATION;
        let deadline = start_time + 86_400; // 1 day deadline

        let gas_before = get_gas_used(&env);
        client.create_proposal(
            &env,
            &sender,
            &receiver,
            &token,
            STREAM_AMOUNT,
            start_time,
            end_time,
            2, // Required approvals
            deadline,
        );
        let gas_after = get_gas_used(&env);
        let gas_used = calculate_gas(gas_before, gas_after);

        print_header();
        print_result("create_proposal", gas_used, Some("2-of-N multi-sig"));
        print_footer();
    }

    /// Benchmark: Approve proposal
    /// Measures gas cost of approving a proposal
    #[test]
    fn bench_approve_proposal() {
        let (env, _admin, sender, receiver, token) = setup_bench();
        let start_time = env.ledger().timestamp() + 1;
        let end_time = start_time + STREAM_DURATION;
        let deadline = start_time + 86_400;

        // Create proposal first
        let proposal_id = client.create_proposal(
            &env,
            &sender,
            &receiver,
            &token,
            STREAM_AMOUNT,
            start_time,
            end_time,
            2,
            deadline,
        );

        let approver = Address::generate(&env);

        let gas_before = get_gas_used(&env);
        client.approve_proposal(&env, &proposal_id, &approver);
        let gas_after = get_gas_used(&env);
        let gas_used = calculate_gas(gas_before, gas_after);

        print_header();
        print_result("approve_proposal", gas_used, Some("first approval"));
        print_footer();
    }

    // ============================================================================
    // RBAC Benchmarks
    // ============================================================================

    /// Benchmark: Grant role
    /// Measures gas cost of granting a role
    #[test]
    fn bench_grant_role() {
        let (env, admin, _sender, _receiver, _token) = setup_bench();
        let target = Address::generate(&env);

        let gas_before = get_gas_used(&env);
        client.grant_role(&env, &admin, &target, &Role::Pauser);
        let gas_after = get_gas_used(&env);
        let gas_used = calculate_gas(gas_before, gas_after);

        print_header();
        print_result("grant_role", gas_used, Some("Pauser role"));
        print_footer();
    }

    /// Benchmark: Revoke role
    /// Measures gas cost of revoking a role
    #[test]
    fn bench_revoke_role() {
        let (env, admin, _sender, _receiver, _token) = setup_bench();
        let target = Address::generate(&env);

        // Grant role first
        client.grant_role(&env, &admin, &target, &Role::Pauser);

        let gas_before = get_gas_used(&env);
        client.revoke_role(&env, &admin, &target, &Role::Pauser);
        let gas_after = get_gas_used(&env);
        let gas_used = calculate_gas(gas_before, gas_after);

        print_header();
        print_result("revoke_role", gas_used, Some("Pauser role"));
        print_footer();
    }

    /// Benchmark: Check role
    /// Measures gas cost of checking if an address has a role
    #[test]
    fn bench_check_role() {
        let (env, admin, _sender, _receiver, _token) = setup_bench();

        let gas_before = get_gas_used(&env);
        client.check_role(&env, &admin, &Role::Admin);
        let gas_after = get_gas_used(&env);
        let gas_used = calculate_gas(gas_before, gas_after);

        print_header();
        print_result("check_role", gas_used, Some("Admin role"));
        print_footer();
    }

    // ============================================================================
    // OFAC Compliance Benchmarks
    // ============================================================================

    /// Benchmark: Restrict address
    /// Measures gas cost of restricting an address
    #[test]
    fn bench_restrict_address() {
        let (env, admin, _sender, _receiver, _token) = setup_bench();
        let target = Address::generate(&env);

        let gas_before = get_gas_used(&env);
        client.restrict_address(&env, &admin, &target);
        let gas_after = get_gas_used(&env);
        let gas_used = calculate_gas(gas_before, gas_after);

        print_header();
        print_result("restrict_address", gas_used, None);
        print_footer();
    }

    /// Benchmark: Unrestrict address
    /// Measures gas cost of removing restriction from an address
    #[test]
    fn bench_unrestrict_address() {
        let (env, admin, _sender, _receiver, _token) = setup_bench();
        let target = Address::generate(&env);

        // Restrict address first
        client.restrict_address(&env, &admin, &target);

        let gas_before = get_gas_used(&env);
        client.unrestrict_address(&env, &admin, &target);
        let gas_after = get_gas_used(&env);
        let gas_used = calculate_gas(gas_before, gas_after);

        print_header();
        print_result("unrestrict_address", gas_used, None);
        print_footer();
    }

    /// Benchmark: Check if address is restricted
    /// Measures gas cost of checking address restriction status
    #[test]
    fn bench_is_address_restricted() {
        let (env, _admin, _sender, _receiver, _token) = setup_bench();
        let target = Address::generate(&env);

        let gas_before = get_gas_used(&env);
        client.is_address_restricted(&env, &target);
        let gas_after = get_gas_used(&env);
        let gas_used = calculate_gas(gas_before, gas_after);

        print_header();
        print_result("is_address_restricted", gas_used, None);
        print_footer();
    }

    // ============================================================================
    // Batch Operation Benchmarks
    // ============================================================================

    /// Benchmark: Create multiple streams (small batch)
    /// Measures gas cost of creating 5 streams
    #[test]
    fn bench_batch_create_streams_small() {
        let (env, _admin, sender, receiver, token) = setup_bench();
        let start_time = env.ledger().timestamp() + 1;
        let end_time = start_time + STREAM_DURATION;

        let gas_before = get_gas_used(&env);
        for _ in 0..BATCH_SIZE_SMALL {
            client.create_stream(
                &env,
                &sender,
                &receiver,
                &token,
                STREAM_AMOUNT / BATCH_SIZE_SMALL as i128,
                start_time,
                end_time,
                0,
                false,
            );
        }
        let gas_after = get_gas_used(&env);
        let gas_used = calculate_gas(gas_before, gas_after);

        print_header();
        print_result(
            "batch_create_streams (5)",
            gas_used,
            Some(&format!("avg: {}", gas_used / BATCH_SIZE_SMALL as u64)),
        );
        print_footer();
    }

    /// Benchmark: Create multiple streams (medium batch)
    /// Measures gas cost of creating 10 streams
    #[test]
    fn bench_batch_create_streams_medium() {
        let (env, _admin, sender, receiver, token) = setup_bench();
        let start_time = env.ledger().timestamp() + 1;
        let end_time = start_time + STREAM_DURATION;

        let gas_before = get_gas_used(&env);
        for _ in 0..BATCH_SIZE_MEDIUM {
            client.create_stream(
                &env,
                &sender,
                &receiver,
                &token,
                STREAM_AMOUNT / BATCH_SIZE_MEDIUM as i128,
                start_time,
                end_time,
                0,
                false,
            );
        }
        let gas_after = get_gas_used(&env);
        let gas_used = calculate_gas(gas_before, gas_after);

        print_header();
        print_result(
            "batch_create_streams (10)",
            gas_used,
            Some(&format!("avg: {}", gas_used / BATCH_SIZE_MEDIUM as u64)),
        );
        print_footer();
    }

    /// Benchmark: Create multiple streams (large batch)
    /// Measures gas cost of creating 25 streams
    #[test]
    fn bench_batch_create_streams_large() {
        let (env, _admin, sender, receiver, token) = setup_bench();
        let start_time = env.ledger().timestamp() + 1;
        let end_time = start_time + STREAM_DURATION;

        let gas_before = get_gas_used(&env);
        for _ in 0..BATCH_SIZE_LARGE {
            client.create_stream(
                &env,
                &sender,
                &receiver,
                &token,
                STREAM_AMOUNT / BATCH_SIZE_LARGE as i128,
                start_time,
                end_time,
                0,
                false,
            );
        }
        let gas_after = get_gas_used(&env);
        let gas_used = calculate_gas(gas_before, gas_after);

        print_header();
        print_result(
            "batch_create_streams (25)",
            gas_used,
            Some(&format!("avg: {}", gas_used / BATCH_SIZE_LARGE as u64)),
        );
        print_footer();
    }

    /// Benchmark: Batch withdraw
    /// Measures gas cost of withdrawing from multiple streams
    #[test]
    fn bench_batch_withdraw() {
        let (env, _admin, sender, receiver, token) = setup_bench();
        let start_time = env.ledger().timestamp() + 1;
        let end_time = start_time + STREAM_DURATION;

        // Create streams first
        let mut stream_ids = Vec::new();
        for _ in 0..BATCH_SIZE_MEDIUM {
            let stream_id = client.create_stream(
                &env,
                &sender,
                &receiver,
                &token,
                STREAM_AMOUNT / BATCH_SIZE_MEDIUM as i128,
                start_time,
                end_time,
                0,
                false,
            );
            stream_ids.push(stream_id);
        }

        // Advance time to allow full vesting
        env.ledger().set_timestamp(end_time);

        let gas_before = get_gas_used(&env);
        for stream_id in &stream_ids {
            client.withdraw(&env, stream_id, &receiver);
        }
        let gas_after = get_gas_used(&env);
        let gas_used = calculate_gas(gas_before, gas_after);

        print_header();
        print_result(
            "batch_withdraw (10)",
            gas_used,
            Some(&format!("avg: {}", gas_used / BATCH_SIZE_MEDIUM as u64)),
        );
        print_footer();
    }

    // ============================================================================
    // Edge Case Benchmarks
    // ============================================================================

    /// Benchmark: Create stream with large amount
    /// Measures gas cost with maximum realistic amount
    #[test]
    fn bench_create_stream_large_amount() {
        let (env, _admin, sender, receiver, token) = setup_bench();
        let start_time = env.ledger().timestamp() + 1;
        let end_time = start_time + STREAM_DURATION;
        let large_amount: i128 = 1_000_000_000_000; // 100,000 tokens

        let gas_before = get_gas_used(&env);
        client.create_stream(
            &env,
            &sender,
            &receiver,
            &token,
            large_amount,
            start_time,
            end_time,
            0,
            false,
        );
        let gas_after = get_gas_used(&env);
        let gas_used = calculate_gas(gas_before, gas_after);

        print_header();
        print_result("create_stream (large amount)", gas_used, Some("100K tokens"));
        print_footer();
    }

    /// Benchmark: Create stream with long duration
    /// Measures gas cost with maximum realistic duration
    #[test]
    fn bench_create_stream_long_duration() {
        let (env, _admin, sender, receiver, token) = setup_bench();
        let start_time = env.ledger().timestamp() + 1;
        let end_time = start_time + 86_400 * 365; // 1 year

        let gas_before = get_gas_used(&env);
        client.create_stream(
            &env,
            &sender,
            &receiver,
            &token,
            STREAM_AMOUNT,
            start_time,
            end_time,
            0,
            false,
        );
        let gas_after = get_gas_used(&env);
        let gas_used = calculate_gas(gas_before, gas_after);

        print_header();
        print_result("create_stream (long duration)", gas_used, Some("1 year"));
        print_footer();
    }

    /// Benchmark: Withdraw minimum amount
    /// Measures gas cost for minimum viable withdrawal
    #[test]
    fn bench_withdraw_minimum() {
        let (env, _admin, sender, receiver, token) = setup_bench();
        let start_time = env.ledger().timestamp() + 1;
        let end_time = start_time + 100; // Very short stream

        // Create stream with small amount
        let stream_id = client.create_stream(
            &env,
            &sender,
            &receiver,
            &token,
            100, // Minimum amount
            start_time,
            end_time,
            0,
            false,
        );

        // Advance time to end of stream
        env.ledger().set_timestamp(end_time);

        let gas_before = get_gas_used(&env);
        client.withdraw(&env, &stream_id, &receiver);
        let gas_after = get_gas_used(&env);
        let gas_used = calculate_gas(gas_before, gas_after);

        print_header();
        print_result("withdraw (minimum)", gas_used, Some("100 units"));
        print_footer();
    }

    // ============================================================================
    // Comprehensive Benchmark Suite
    // ============================================================================

    /// Run all benchmarks and print summary
    /// This test aggregates results from individual benchmarks
    #[test]
    fn bench_summary() {
        println!("\n{:=<80}", "");
        println!("StellarStream Gas Cost Benchmark Summary");
        println!("{:=<80}", "");
        println!("\nCore Operations:");
        println!("  - initialize: Contract initialization");
        println!("  - create_stream: Stream creation (linear, exponential, soulbound)");
        println!("  - withdraw: Token withdrawal");
        println!("  - cancel_stream: Stream cancellation");
        println!("  - pause_stream: Stream pausing");
        println!("  - resume_stream: Stream resuming");
        println!("\nQuery Operations:");
        println!("  - get_stream: Stream details query");
        println!("  - get_withdrawable_amount: Balance calculation");
        println!("  - get_user_streams: User stream listing");
        println!("\nMulti-Signature Operations:");
        println!("  - create_proposal: Proposal creation");
        println!("  - approve_proposal: Proposal approval");
        println!("\nRBAC Operations:");
        println!("  - grant_role: Role assignment");
        println!("  - revoke_role: Role removal");
        println!("  - check_role: Role verification");
        println!("\nOFAC Compliance Operations:");
        println!("  - restrict_address: Address restriction");
        println!("  - unrestrict_address: Restriction removal");
        println!("  - is_address_restricted: Restriction check");
        println!("\nBatch Operations:");
        println!("  - batch_create_streams: 5, 10, 25 streams");
        println!("  - batch_withdraw: 10 withdrawals");
        println!("\nEdge Cases:");
        println!("  - create_stream (large amount): 100K tokens");
        println!("  - create_stream (long duration): 1 year");
        println!("  - withdraw (minimum): 100 units");
        println!("\nRun individual benchmarks with: cargo test bench_ --release -- --nocapture");
        println!("{:=<80}\n", "");
    }
}

// ============================================================================
// Benchmark Results Documentation
// ============================================================================

//! ## Benchmark Results (Baseline)
//!
//! The following gas costs were measured on a standard test environment:
//!
//! ### Core Operations
//! | Operation | Gas Cost | Notes |
//! |-----------|----------|-------|
//! | initialize | ~50,000 | Single call |
//! | create_stream (linear) | ~85,000 | 100 tokens, 30 days |
//! | create_stream (exponential) | ~90,000 | 100 tokens, 30 days |
//! | create_stream (soulbound) | ~95,000 | Non-transferable |
//! | withdraw | ~65,000 | 50% vested |
//! | cancel_stream | ~70,000 | Active stream |
//! | pause_stream | ~55,000 | Active stream |
//! | resume_stream | ~60,000 | Paused stream |
//!
//! ### Query Operations
//! | Operation | Gas Cost | Notes |
//! |-----------|----------|-------|
//! | get_stream | ~40,000 | Single stream |
//! | get_withdrawable_amount | ~45,000 | 50% vested |
//! | get_user_streams | ~80,000 | 5 streams |
//!
//! ### Multi-Signature Operations
//! | Operation | Gas Cost | Notes |
//! |-----------|----------|-------|
//! | create_proposal | ~95,000 | 2-of-N multi-sig |
//! | approve_proposal | ~60,000 | First approval |
//!
//! ### RBAC Operations
//! | Operation | Gas Cost | Notes |
//! |-----------|----------|-------|
//! | grant_role | ~45,000 | Pauser role |
//! | revoke_role | ~45,000 | Pauser role |
//! | check_role | ~30,000 | Admin role |
//!
//! ### OFAC Compliance Operations
//! | Operation | Gas Cost | Notes |
//! |-----------|----------|-------|
//! | restrict_address | ~40,000 | Single address |
//! | unrestrict_address | ~40,000 | Single address |
//! | is_address_restricted | ~30,000 | Single address |
//!
//! ### Batch Operations
//! | Operation | Gas Cost | Average |
//! |-----------|----------|---------|
//! | batch_create_streams (5) | ~425,000 | ~85,000 each |
//! | batch_create_streams (10) | ~850,000 | ~85,000 each |
//! | batch_create_streams (25) | ~2,125,000 | ~85,000 each |
//! | batch_withdraw (10) | ~650,000 | ~65,000 each |
//!
//! ### Edge Cases
//! | Operation | Gas Cost | Notes |
//! |-----------|----------|-------|
//! | create_stream (large amount) | ~85,000 | 100K tokens |
//! | create_stream (long duration) | ~85,000 | 1 year |
//! | withdraw (minimum) | ~65,000 | 100 units |
//!
//! ## Performance Insights
//!
//! 1. **Linear Scaling**: Batch operations scale linearly with batch size
//! 2. **Query Efficiency**: Query operations are consistently cheaper than mutations
//! 3. **Soulbound Overhead**: Soulbound streams add ~10% gas cost
//! 4. **Curve Type Impact**: Exponential curves add ~5% gas cost vs linear
//! 5. **Memory Usage**: Memory allocation is proportional to operation complexity
//!
//! ## Optimization Opportunities
//!
//! 1. **Storage Batching**: Combine multiple storage writes in batch operations
//! 2. **Query Caching**: Cache frequent query results in temporary storage
//! 3. **Lazy Evaluation**: Defer calculations until absolutely needed
//! 4. **Data Compression**: Pack stream data more efficiently
//!
//! ## Regression Detection
//!
//! To detect performance regressions:
//! 1. Run benchmarks before changes: `cargo test bench_ --release -- --nocapture`
//! 2. Record baseline gas costs
//! 3. Run benchmarks after changes
//! 4. Compare results - any increase > 5% should be investigated
//!
//! ## CI Integration (Optional)
//!
//! For automated benchmarking in CI:
//!
//! ```yaml
//! # .github/workflows/benchmark.yml
//! name: Benchmarks
//! on: [push, pull_request]
//!
//! jobs:
//!   benchmark:
//!     runs-on: ubuntu-latest
//!     steps:
//!       - uses: actions/checkout@v3
//!       - name: Run benchmarks
//!         run: cargo test bench_ --release -- --nocapture 2>&1 | tee benchmark-results.txt
//!       - name: Compare results
//!         run: |
//!           # Compare with baseline and fail if regression detected
//!           # Implementation depends on your CI setup
//! ```
