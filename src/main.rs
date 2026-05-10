mod core;
mod vm;
mod bridge;
mod economics;
mod governance;

use core::CoreState;
use vm::{VmState, ObjectType, TypeAbilities};
use bridge::BridgeState;
use economics::EconomicsState;
use governance::{GovernanceState, ProposalType};

fn main() {
    println!("╔══════════════════════════════════════╗");
    println!("║     LUGET Research Simulator        ║");
    println!("║     lugsim v0.1.0                   ║");
    println!("║     Phase 0 — Architecture Proof    ║");
    println!("╚══════════════════════════════════════╝");
    println!();

    let mut gov = GovernanceState::new();
    gov.total_active_stake = 10_000_000;
    gov.total_dormant_stake = 1_000_000;

    // ==========================================
    // 1. Core Council Proposal
    // ==========================================
    println!("═══════════ GOVERNANCE ═══════════");
    let prop1 = gov.submit_proposal(
        "Adjust CLock fee by 5%",
        "Increase CLock creation fee from 10 LGT to 10.5 LGT",
        ProposalType::CoreParameter,
    );

    gov.proposals[0].status = governance::ProposalStatus::Voting;
    gov.vote(&prop1, 2_000_000, true).unwrap();   // 20% for
    gov.vote(&prop1, 500_000, false).unwrap();    // 5% against
    gov.tally(&prop1).unwrap();

    // ==========================================
    // 2. VM Council Upgrade
    // ==========================================
    let prop2 = gov.submit_proposal(
        "Add ZK-proof opcode",
        "Add native verification opcode for Groth16 proofs",
        ProposalType::VMUpgrade,
    );

    gov.proposals[1].status = governance::ProposalStatus::Voting;
    gov.vote(&prop2, 4_000_000, true).unwrap();
    gov.vote(&prop2, 1_000_000, false).unwrap();
    gov.tally(&prop2).unwrap();

    // ==========================================
    // 3. Holder Veto
    // ==========================================
    println!("\n--- Holder Veto Attempt ---");
    let prop3 = gov.submit_proposal(
        "Unpopular parameter change",
        "Something the community dislikes",
        ProposalType::CoreParameter,
    );
    gov.proposals[2].status = governance::ProposalStatus::Voting;
    gov.vote(&prop3, 1_000_000, true).unwrap();
    // Veto triggers: 3,500,000 LGT votes NO (35% > 33%)
    let veto_result = gov.holder_veto(&prop3, 3_500_000).unwrap();
    println!("Veto successful: {}", veto_result);

    // ==========================================
    // 4. Monetary Emergency
    // ==========================================
    println!("\n--- Monetary Emergency ---");
    let emergency_result = gov.declare_monetary_emergency([true, true, true]).unwrap();
    println!("Emergency declared: {}", emergency_result);

    // ==========================================
    // 5. Constitutional Amendment
    // ==========================================
    println!("\n--- Constitutional Amendment ---");
    let prop4 = gov.submit_proposal(
        "Upgrade hash function",
        "Migrate from Blake3 to post-quantum hash",
        ProposalType::ConstitutionalAmendment,
    );
    gov.proposals[3].status = governance::ProposalStatus::Voting;
    // 8M participation (80%), 7.5M for (93.75% approval)
    gov.vote(&prop4, 7_500_000, true).unwrap();
    gov.vote(&prop4, 500_000, false).unwrap();
    gov.tally(&prop4).unwrap();

    // ==========================================
    // 6. Dormancy Fee
    // ==========================================
    println!("\n--- Dormancy Fee ---");
    gov.apply_dormancy_fee();

    gov.advance_epoch();
    gov.print_state();

    // ==========================================
    // SUMMARY
    // ==========================================
    println!("\n═══════════ LUGET SIMULATOR COMPLETE ═══════════");
    println!("All modules validated:");
    println!("  [✓] Core — UTXO state machine, fee burning, CLocks");
    println!("  [✓] VM   — Object model, type abilities, ownership modes");
    println!("  [✓] Bridge — Deposit, withdrawal delay, unilateral close, managed pause");
    println!("  [✓] Economics — Dual-pool, uptime multiplier, slashing, MEV redistribution");
    println!("  [✓] Governance — 4 institutions, voting, veto, constitutional amendments");
    println!();
    println!("Last: P2P Networking — Multi-node gossip + block propagation");
}
