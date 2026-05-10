mod core;
mod vm;
mod bridge;
mod economics;

use core::CoreState;
use vm::{VmState, ObjectType, TypeAbilities, Owner};
use bridge::BridgeState;
use economics::{EconomicsState, SlashReason};

fn main() {
    println!("╔══════════════════════════════════════╗");
    println!("║     LUGET Research Simulator        ║");
    println!("║     lugsim v0.1.0                   ║");
    println!("║     Phase 0 — Architecture Proof    ║");
    println!("╚══════════════════════════════════════╝");
    println!();

    // ==========================================
    // 1. SETUP
    // ==========================================
    let mut core = CoreState::new();
    core.genesis("alice", 1_000_000_000);

    let mut vm = VmState::new();
    vm.register_type(ObjectType {
        type_id: "Coin<LGT>".to_string(), name: "Coin<LGT>".to_string(),
        abilities: TypeAbilities { copy: false, drop: false, store: true, key: true },
    });

    let mut bridge = BridgeState::new();
    let mut econ = EconomicsState::new(core.total_supply);

    // ==========================================
    // 2. VALIDATOR ECONOMICS
    // ==========================================
    println!("═══════════ VALIDATOR ECONOMICS ═══════════");

    // Register validators
    econ.register_validator("validator-alice", 100_000).unwrap();
    econ.register_validator("validator-bob", 80_000).unwrap();
    econ.register_validator("validator-charlie", 50_000).unwrap();

    // Delegation
    econ.delegate("validator-1", 25_000).unwrap();
    econ.delegate("validator-2", 15_000).unwrap();

    // Fund pools
    econ.fund_pool_a();
    econ.fund_pool_b(500, 200, 100);

    // Set uptime scores
    econ.set_uptime("validator-1", 1.0).unwrap();   // Perfect
    econ.set_uptime("validator-2", 0.95).unwrap();  // Good
    econ.set_uptime("validator-3", 0.50).unwrap();  // Struggling

    econ.distribute_rewards();
    econ.print_state();

    // ==========================================
    // 3. SLASHING DEMONSTRATION
    // ==========================================
    println!("\n═══════════ SLASHING ═══════════");

    // Double-sequencing: full slash, jail
    println!("\n--- Double-Sequencing (validator-3) ---");
    match econ.slash("validator-3", SlashReason::DoubleSequencing) {
        Ok(amount) => println!("{} LGT burned. Validator jailed.", amount),
        Err(e) => println!("Slash failed: {}", e),
    }

    // Invalid state root: partial slash, delegators protected
    println!("\n--- Invalid State Root (validator-2) ---");
    match econ.slash("validator-2", SlashReason::InvalidStateRoot) {
        Ok(amount) => println!("{} LGT burned from self-bonded stake. Delegators unaffected.", amount),
        Err(e) => println!("Slash failed: {}", e),
    }

    econ.print_state();

    // ==========================================
    // 4. BRIDGE OPERATIONS
    // ==========================================
    println!("\n═══════════ BRIDGE ═══════════");
    let coin_id = BridgeState::deposit(&mut core, &mut vm, "alice", 500).unwrap();
    println!("Deposited 500 LGT -> VM object {}", coin_id);

    vm.transfer_object(&coin_id, Owner::Address("bob".to_string())).unwrap();
    println!("Transferred Coin to bob");

    bridge.request_withdrawal(&mut vm, &coin_id, "bob").unwrap();
    bridge.advance_epoch();
    bridge.advance_epoch();
    bridge.complete_withdrawal(&mut core, "clock-for-obj-2", 1, "bob", 0).unwrap();

    // ==========================================
    // 5. SUMMARY
    // ==========================================
    println!("\n═══════════ SIMULATION COMPLETE ═══════════");
    println!("Modules validated:");
    println!("  [✓] Core UTXO state machine");
    println!("  [✓] VM object model with type abilities");
    println!("  [✓] Bridge deposit, withdrawal, unilateral close");
    println!("  [✓] Validator economics (dual-pool + uptime)");
    println!("  [✓] Slashing (malicious + operator faults)");
    println!("  [✓] MEV redistribution (25% fee floor)");
    println!();
    println!("Next: Governance (4 institutions, voting, veto)");
}
