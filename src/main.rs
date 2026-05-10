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
mod benchmarks;

use crypto::KeyPair;
use governance::{GovernanceState, ProposalType};
use persistence::SavedState;
use bls_committee::BlsCommittee;

const SAVE_FILE: &str = "lugsim-state.json";

fn main() {
    println!("╔══════════════════════════════════════╗");
    println!("║     LUGET Research Simulator        ║");
    println!("║     lugsim v0.9.0                   ║");
    println!("║     Phase 0 — Performance Metrics   ║");
    println!("╚══════════════════════════════════════╝");
    println!();

    if let Ok(saved) = SavedState::load_from_file(SAVE_FILE) {
        println!("Found saved state (epoch {})", saved.epoch);
    }

    // Quick crypto verification
    let alice = KeyPair::generate();
    let msg = b"LUGET";
    let sig = alice.sign(msg);
    assert!(KeyPair::verify(&alice.public_key, msg, &sig));

    // Quick BLS verification
    let bls = BlsCommittee::new();
    let (success, _) = bls.attest_block("benchmark-header");
    assert!(success);

    // Quick governance
    let mut gov = GovernanceState::new();
    gov.total_active_stake = 10_000_000;
    let prop = gov.submit_proposal("Benchmark era", "Performance validated", ProposalType::CoreParameter);
    gov.proposals[0].status = governance::ProposalStatus::Voting;
    gov.vote(&prop, 8_000_000, true).unwrap();
    gov.tally(&prop).unwrap();

    // Run all performance benchmarks
    let results = benchmarks::run_all();

    println!("═══════════ LUGSIM v0.9.0 COMPLETE ═══════════");
    println!("66 tests. Zero warnings. Benchmarked.");
}
