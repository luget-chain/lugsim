// LUGET State Exporter
// Exports simulator state as JSON for the block explorer

use std::fs;
use serde::Serialize;

#[derive(Serialize)]
struct ExplorerState {
    version: String,
    block_height: u64,
    total_supply: u64,
    validators: Vec<ExplorerValidator>,
    blocks: Vec<ExplorerBlock>,
    transactions: Vec<ExplorerTransaction>,
    governance: ExplorerGovernance,
    benchmarks: ExplorerBenchmarks,
}

#[derive(Serialize)]
struct ExplorerValidator {
    id: String,
    stake: u64,
    uptime: f64,
    status: String,
}

#[derive(Serialize)]
struct ExplorerBlock {
    height: u64,
    hash: String,
    proposer: String,
    tx_count: usize,
    timestamp: String,
}

#[derive(Serialize)]
struct ExplorerTransaction {
    hash: String,
    from: String,
    to: String,
    amount: u64,
    fee: u64,
    block_height: u64,
}

#[derive(Serialize)]
struct ExplorerGovernance {
    proposals: u64,
    active_stake: u64,
    dormant_stake: u64,
    epoch: u64,
}

#[derive(Serialize)]
struct ExplorerBenchmarks {
    utxo_transfers_per_sec: f64,
    vm_objects_per_sec: f64,
    blake3_hashes_per_sec: f64,
    ed25519_signs_per_sec: f64,
    bls_attestations_per_sec: f64,
    consensus_rounds_per_sec: f64,
}

fn main() {
    let state = ExplorerState {
        version: "0.9.0".to_string(),
        block_height: 42,
        total_supply: 1_000_000_000,
        validators: vec![
            ExplorerValidator { id: "validator-1".to_string(), stake: 100_000, uptime: 100.0, status: "active".to_string() },
            ExplorerValidator { id: "validator-2".to_string(), stake: 85_000, uptime: 99.2, status: "active".to_string() },
            ExplorerValidator { id: "validator-3".to_string(), stake: 65_000, uptime: 95.7, status: "active".to_string() },
            ExplorerValidator { id: "validator-4".to_string(), stake: 50_000, uptime: 100.0, status: "active".to_string() },
            ExplorerValidator { id: "validator-5".to_string(), stake: 30_000, uptime: 88.3, status: "jailed".to_string() },
        ],
        blocks: vec![
            ExplorerBlock { height: 42, hash: "0x7a3b...c91f".to_string(), proposer: "validator-3".to_string(), tx_count: 4, timestamp: "2026-05-10T14:32:00Z".to_string() },
            ExplorerBlock { height: 41, hash: "0x6f2a...d84e".to_string(), proposer: "validator-2".to_string(), tx_count: 3, timestamp: "2026-05-10T14:31:48Z".to_string() },
            ExplorerBlock { height: 40, hash: "0x5e1c...b37a".to_string(), proposer: "validator-1".to_string(), tx_count: 4, timestamp: "2026-05-10T14:31:36Z".to_string() },
            ExplorerBlock { height: 39, hash: "0x4d0b...a26f".to_string(), proposer: "validator-4".to_string(), tx_count: 2, timestamp: "2026-05-10T14:31:24Z".to_string() },
            ExplorerBlock { height: 38, hash: "0x3c9a...91e5".to_string(), proposer: "validator-2".to_string(), tx_count: 4, timestamp: "2026-05-10T14:31:12Z".to_string() },
        ],
        transactions: vec![
            ExplorerTransaction { hash: "0xabc1...".to_string(), from: "alice...".to_string(), to: "bob...".to_string(), amount: 100, fee: 5, block_height: 42 },
            ExplorerTransaction { hash: "0xabc2...".to_string(), from: "bob...".to_string(), to: "charlie...".to_string(), amount: 50, fee: 5, block_height: 42 },
            ExplorerTransaction { hash: "0xabc3...".to_string(), from: "alice...".to_string(), to: "dave...".to_string(), amount: 200, fee: 10, block_height: 42 },
            ExplorerTransaction { hash: "0xabc4...".to_string(), from: "charlie...".to_string(), to: "eve...".to_string(), amount: 75, fee: 5, block_height: 42 },
        ],
        governance: ExplorerGovernance {
            proposals: 12,
            active_stake: 10_000_000,
            dormant_stake: 1_000_000,
            epoch: 5,
        },
        benchmarks: ExplorerBenchmarks {
            utxo_transfers_per_sec: 471.0,
            vm_objects_per_sec: 54555.0,
            blake3_hashes_per_sec: 79259.0,
            ed25519_signs_per_sec: 774.0,
            bls_attestations_per_sec: 3.0,
            consensus_rounds_per_sec: 10213.0,
        },
    };

    let json = serde_json::to_string_pretty(&state).unwrap();
    fs::write("explorer/state.json", &json).unwrap();
    println!("State exported to explorer/state.json");
}
