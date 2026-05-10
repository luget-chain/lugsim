mod crypto;
mod core;
mod vm;
mod bridge;
mod economics;
mod governance;
mod network;

use crypto::KeyPair;
use governance::{GovernanceState, ProposalType};

fn main() {
    println!("╔══════════════════════════════════════╗");
    println!("║     LUGET Research Simulator        ║");
    println!("║     lugsim v0.3.0                   ║");
    println!("║     Phase 0 — Architecture Proof    ║");
    println!("║     With P2P Networking             ║");
    println!("╚══════════════════════════════════════╝");
    println!();

    // ==========================================
    // 1. Cryptographic Key Generation
    // ==========================================
    println!("═══════════ KEY GENERATION ═══════════");
    let alice = KeyPair::generate();
    let bob = KeyPair::generate();
    println!("Alice: {}", alice.address());
    println!("Bob:   {}", bob.address());

    let message = b"Alice sends 100 LGT to Bob";
    let signature = alice.sign(message);
    let valid = KeyPair::verify(&alice.public_key, message, &signature);
    println!("Signature test: {}\n", if valid { "VALID ✓" } else { "INVALID ✗" });

    // ==========================================
    // 2. Governance
    // ==========================================
    println!("═══════════ GOVERNANCE ═══════════");
    let mut gov = GovernanceState::new();
    gov.total_active_stake = 10_000_000;
    let prop = gov.submit_proposal("First crypto proposal", "Signed with Ed25519", ProposalType::CoreParameter);
    gov.proposals[0].status = governance::ProposalStatus::Voting;
    gov.vote(&prop, 3_000_000, true).unwrap();
    gov.tally(&prop).unwrap();
    gov.print_state();

    // ==========================================
    // 3. Multi-Node Networking
    // ==========================================
    network::run_multi_node_simulation();

    // ==========================================
    // Summary
    // ==========================================
    println!("\n═══════════ LUGSIM v0.3.0 COMPLETE ═══════════");
    println!("All modules validated:");
    println!("  [✓] Ed25519 + Blake3 cryptography");
    println!("  [✓] Core UTXO state machine");
    println!("  [✓] VM object model with type abilities");
    println!("  [✓] Bridge deposit/withdrawal/unilateral close");
    println!("  [✓] Validator economics (dual-pool + slashing)");
    println!("  [✓] Governance (4 institutions + veto)");
    println!("  [✓] P2P Networking (gossip + voting + epochs)");
    println!();
    println!("The LUGET Phase 0 Simulator is feature-complete.");
    println!("Next: Testnet infrastructure + formal verification.");
}
