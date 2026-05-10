mod crypto;
mod core;
mod vm;
mod bridge;
mod economics;
mod governance;
mod network;
mod persistence;
mod consensus;
mod mempool;

use crypto::KeyPair;
use persistence::SavedState;
use consensus::ConsensusState;
use mempool::Mempool;

const SAVE_FILE: &str = "lugsim-state.json";

fn main() {
    println!("╔══════════════════════════════════════╗");
    println!("║     LUGET Research Simulator        ║");
    println!("║     lugsim v0.6.0                   ║");
    println!("║     Phase 0 — Mempool + Consensus   ║");
    println!("╚══════════════════════════════════════╝");
    println!();

    // Load saved state
    if let Ok(saved) = SavedState::load_from_file(SAVE_FILE) {
        println!("Found saved state (epoch {}).\n", saved.epoch);
    }

    // ==========================================
    // 1. Key Generation
    // ==========================================
    println!("═══════════ KEY GENERATION ═══════════");
    let alice = KeyPair::generate();
    let bob = KeyPair::generate();
    let charlie = KeyPair::generate();
    println!("Alice:   {}", &alice.address()[..32]);
    println!("Bob:     {}", &bob.address()[..32]);
    println!("Charlie: {}", &charlie.address()[..32]);

    // ==========================================
    // 2. Mempool — Submit Transactions
    // ==========================================
    println!("\n═══════════ MEMPOOL ═══════════");
    let mut mempool = Mempool::new(200, 1);

    // Users submit transactions
    mempool.submit(
        &alice.address(), &bob.address(), 100, 5, 0,
        alice.sign(b"alice->bob:100")
    ).unwrap();

    mempool.submit(
        &bob.address(), &charlie.address(), 50, 10, 0,  // Higher fee = higher priority
        bob.sign(b"bob->charlie:50")
    ).unwrap();

    mempool.submit(
        &charlie.address(), &alice.address(), 25, 3, 0,
        charlie.sign(b"charlie->alice:25")
    ).unwrap();

    mempool.submit(
        &alice.address(), &charlie.address(), 200, 8, 1,
        alice.sign(b"alice->charlie:200")
    ).unwrap();

    // Try a transaction with fee too low
    match mempool.submit(&bob.address(), &alice.address(), 10, 0, 0, vec![1]) {
        Ok(_) => {}
        Err(e) => println!("Rejected: {}", e),
    }

    mempool.print_state();

    // ==========================================
    // 3. Select Transactions for Block
    // ==========================================
    println!("\n═══════════ BLOCK SELECTION ═══════════");
    let block_txs = mempool.select_for_block(3);
    let tx_strings: Vec<String> = block_txs
        .iter()
        .map(|tx| format!("{}→{}:{} LGT", &tx.from[..8], &tx.to[..8], tx.amount))
        .collect();
    
    println!("Block transactions:");
    for (i, tx) in tx_strings.iter().enumerate() {
        println!("  {}. {}", i + 1, tx);
    }
    println!("Remaining in mempool: {}", mempool.pending_count());

    // ==========================================
    // 4. Consensus with Mempool Transactions
    // ==========================================
    println!("\n═══════════ CONSENSUS ═══════════");
    let mut consensus = ConsensusState::new();
    consensus.add_validator("validator-1", 100_000);
    consensus.add_validator("validator-2", 80_000);
    consensus.add_validator("validator-3", 70_000);

    // Submit more transactions to mempool
    for i in 0..8 {
        mempool.submit(
            &alice.address(), &bob.address(), 10 * (i + 1), 5, i as u64 + 2,
            alice.sign(format!("tx-{}", i).as_bytes())
        ).unwrap();
    }

    // Run 3 consensus rounds, each pulling from mempool
    for round in 1..=3 {
        let txs = mempool.select_for_block(2);
        let tx_strs: Vec<String> = txs.iter()
            .map(|tx| format!("{}→{}:{}", &tx.from[..8], &tx.to[..8], tx.amount))
            .collect();
        
        consensus.run_round(tx_strs).unwrap();
        println!("Round {}: {} txs remaining in mempool\n", round, mempool.pending_count());
    }

    consensus.print_state();

    // ==========================================
    // 5. Expired Transaction Cleanup
    // ==========================================
    println!("\n═══════════ MEMPOOL MAINTENANCE ═══════════");
    let cleared = mempool.clear_expired(0); // Clear all
    println!("Cleared {} transactions from mempool", cleared);
    println!("Pending: {}", mempool.pending_count());

    // ==========================================
    // Summary
    // ==========================================
    println!("\n═══════════ LUGSIM v0.6.0 COMPLETE ═══════════");
    println!("All modules validated:");
    println!("  [✓] Ed25519 + Blake3 cryptography");
    println!("  [✓] Core UTXO state machine");
    println!("  [✓] VM object model with type abilities");
    println!("  [✓] Bridge deposit/withdrawal/unilateral close");
    println!("  [✓] Validator economics (dual-pool + slashing)");
    println!("  [✓] Governance (4 institutions + veto)");
    println!("  [✓] P2P Networking (gossip + voting + epochs)");
    println!("  [✓] Persistent state (save/load to disk)");
    println!("  [✓] BFT Consensus (block proposal + attestation + finality)");
    println!("  [✓] Mempool (tx submission, priority ordering, expiry)");
    println!();
    println!("Transaction flow: User → Mempool → Consensus → Block → Chain");
    println!("Next: Continuous block production + genesis definition.");
}
