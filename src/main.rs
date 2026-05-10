#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_mut)]

mod crypto;
mod core;
mod vm;
mod bridge;
mod economics;
mod governance;
mod network;
mod persistence;
mod producer;
mod mempool;
mod consensus;
mod bls_committee;

use crypto::KeyPair;
use governance::{GovernanceState, ProposalType};
use persistence::SavedState;
use bls_committee::BlsCommittee;

const SAVE_FILE: &str = "lugsim-state.json";

fn main() {
    println!("╔══════════════════════════════════════╗");
    println!("║     LUGET Research Simulator        ║");
    println!("║     lugsim v0.8.0                   ║");
    println!("║     Phase 0 — BLS Threshold Sigs    ║");
    println!("╚══════════════════════════════════════╝");
    println!();

    if let Ok(saved) = SavedState::load_from_file(SAVE_FILE) {
        println!("Found saved state:");
        saved.display();
        println!();
    }

    // Ed25519
    println!("═══════════ KEY GENERATION ═══════════");
    let alice = KeyPair::generate();
    println!("Alice: {}", alice.address());
    let msg = b"Alice signs with Ed25519";
    let sig = alice.sign(msg);
    println!("Ed25519: {} ✓", if KeyPair::verify(&alice.public_key, msg, &sig) { "valid" } else { "invalid" });

    // BLS Threshold Signatures
    println!("\n═══════════ BLS THRESHOLD SIGNATURES ═══════════");
    println!("Generating Bridge Light Client Committee...");
    let bls = BlsCommittee::new();
    bls.print_status();

    println!("\n--- BLCC Attestation Round ---");
    let (success, sig_hex) = bls.attest_block("core-block-0xdeadbeefcafebabe");
    if success {
        println!("BLCC attestation: SUCCESS ✓");
        println!("Aggregated signature: {}...", &sig_hex[..32]);
    } else {
        println!("BLCC attestation: FAILED ✗");
    }

    // Governance
    println!("\n═══════════ GOVERNANCE ═══════════");
    let mut gov = GovernanceState::new();
    gov.total_active_stake = 10_000_000;
    let prop = gov.submit_proposal("BLS integration", "Add BLS threshold signatures to BLCC", ProposalType::CoreParameter);
    gov.proposals[0].status = governance::ProposalStatus::Voting;
    gov.vote(&prop, 8_000_000, true).unwrap();
    gov.tally(&prop).unwrap();

    // Summary
    println!("\n═══════════ LUGSIM v0.8.0 COMPLETE ═══════════");
    println!("Cryptographic primitives:");
    println!("  [✓] Ed25519 — user transactions");
    println!("  [✓] Blake3 — hashing");
    println!("  [✓] BLS — BLCC threshold signatures (80/100)");
    println!();
    println!("Next: Formal verification of bridge invariants.");
}
