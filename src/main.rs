mod crypto;
mod core;
mod vm;
mod bridge;
mod economics;
mod governance;

use crypto::KeyPair;
use governance::{GovernanceState, ProposalType};

fn main() {
    println!("╔══════════════════════════════════════╗");
    println!("║     LUGET Research Simulator        ║");
    println!("║     lugsim v0.2.0                   ║");
    println!("║     Phase 0 — Architecture Proof    ║");
    println!("║     With Ed25519 + Blake3           ║");
    println!("╚══════════════════════════════════════╝");
    println!();

    // Generate real cryptographic keypairs
    println!("═══════════ KEY GENERATION ═══════════");
    let alice = KeyPair::generate();
    let bob = KeyPair::generate();
    let charlie = KeyPair::generate();
    println!("Alice:   {}", alice.address());
    println!("Bob:     {}", bob.address());
    println!("Charlie: {}", charlie.address());

    // Sign a message
    let message = b"Alice sends 100 LGT to Bob";
    let signature = alice.sign(message);
    let valid = KeyPair::verify(&alice.public_key, message, &signature);
    println!("\nSignature test: {}", if valid { "VALID ✓" } else { "INVALID ✗" });

    // Hash a transaction
    let tx_hash = crypto::hash_str("genesis-transaction");
    let utxo_id = crypto::utxo_id(&tx_hash, 0);
    println!("Genesis UTXO ID: {}", &utxo_id[..32]);

    // Governance simulation
    println!("\n═══════════ GOVERNANCE ═══════════");
    let mut gov = GovernanceState::new();
    gov.total_active_stake = 10_000_000;

    let prop = gov.submit_proposal(
        "First cryptographic proposal",
        "Proposal signed with Ed25519 keys",
        ProposalType::CoreParameter,
    );
    gov.proposals[0].status = governance::ProposalStatus::Voting;
    gov.vote(&prop, 3_000_000, true).unwrap();
    gov.tally(&prop).unwrap();
    gov.print_state();

    println!("\n═══════════ LUGSIM v0.2.0 COMPLETE ═══════════");
    println!("Real cryptography integrated:");
    println!("  [✓] Ed25519 key generation");
    println!("  [✓] Message signing and verification");
    println!("  [✓] Blake3 hashing for UTXO IDs, state roots");
    println!("  [✓] Cryptographic address derivation");
    println!();
    println!("Next: Multi-node networking + consensus");
}
