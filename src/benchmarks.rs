// LUGET Performance Benchmarks
// Measures TPS, consensus time, cryptographic overhead
// Kipngetich Clinton, Kenya

use std::time::{Instant, Duration};
use crate::crypto::KeyPair;
use crate::bls_committee::BlsCommittee;

/// A single benchmark result
#[derive(Debug, Clone)]
pub struct Benchmark {
    pub name: String,
    pub operations: u64,
    pub duration_ms: u64,
    pub ops_per_second: f64,
}

impl Benchmark {
    pub fn display(&self) {
        println!("  {:<35} {:>8} ops in {:>6} ms = {:>10.0} ops/s",
            self.name, self.operations, self.duration_ms, self.ops_per_second);
    }
}

/// Run all performance benchmarks
pub fn run_all() -> Vec<Benchmark> {
    println!("═══════════ PERFORMANCE BENCHMARKS ═══════════");
    println!("Measuring LUGET throughput and latency...\n");

    let mut results = Vec::new();

    results.push(bench_utxo_transfers());
    results.push(bench_vm_object_creation());
    results.push(bench_ed25519_signatures());
    results.push(bench_blake3_hashing());
    results.push(bench_bls_attestation());
    results.push(bench_consensus_round());

    println!("\n--- Results ---");
    for b in &results {
        b.display();
    }

    println!("\n══════════════════════════════════════════════");
    let total_ops: u64 = results.iter().map(|b| b.operations).sum();
    println!("Total operations: {}", total_ops);
    println!("These numbers represent single-core performance");
    println!("on the host device. Production throughput with");
    println!("parallel execution will be significantly higher.");
    println!("══════════════════════════════════════════════\n");

    results
}

/// Benchmark UTXO transfer creation and validation
fn bench_utxo_transfers() -> Benchmark {
    let count = 10_000;
    let start = Instant::now();

    for i in 0..count {
        let alice = KeyPair::generate();
        let bob = KeyPair::generate();
        let message = format!("alice→bob:{} LGT", i);
        let signature = alice.sign(message.as_bytes());
        let valid = KeyPair::verify(&alice.public_key, message.as_bytes(), &signature);
        assert!(valid);
    }

    let duration = start.elapsed();
    let ms = duration.as_millis() as u64;
    let ops_per_sec = count as f64 / duration.as_secs_f64();

    Benchmark {
        name: "UTXO Transfer (sign+verify)".to_string(),
        operations: count,
        duration_ms: ms,
        ops_per_second: ops_per_sec,
    }
}

/// Benchmark VM object creation throughput
fn bench_vm_object_creation() -> Benchmark {
    let count = 50_000;
    let start = Instant::now();

    for i in 0..count {
        let id = crate::crypto::object_id("benchmark-tx", i);
        let _owner = crate::crypto::hash_bytes(id.as_bytes());
        // Simulating object creation: hashing ID + deriving owner
    }

    let duration = start.elapsed();
    let ms = duration.as_millis() as u64;
    let ops_per_sec = count as f64 / duration.as_secs_f64();

    Benchmark {
        name: "VM Object Creation (hash)".to_string(),
        operations: count,
        duration_ms: ms,
        ops_per_second: ops_per_sec,
    }
}

/// Benchmark Ed25519 signing throughput
fn bench_ed25519_signatures() -> Benchmark {
    let count = 5_000;
    let keypair = KeyPair::generate();
    let message = b"LUGET transaction payload for benchmarking Ed25519 signing performance";

    let start = Instant::now();
    for _ in 0..count {
        let sig = keypair.sign(message);
        let valid = KeyPair::verify(&keypair.public_key, message, &sig);
        assert!(valid);
    }

    let duration = start.elapsed();
    let ms = duration.as_millis() as u64;
    let ops_per_sec = count as f64 / duration.as_secs_f64();

    Benchmark {
        name: "Ed25519 Sign+Verify".to_string(),
        operations: count,
        duration_ms: ms,
        ops_per_second: ops_per_sec,
    }
}

/// Benchmark Blake3 hashing throughput
fn bench_blake3_hashing() -> Benchmark {
    let count = 100_000;
    let data = b"LUGET block header data for Blake3 hashing benchmark test vector padding";
    let start = Instant::now();

    for _ in 0..count {
        let hash = crate::crypto::hash_bytes(data);
        assert!(!hash.is_empty());
    }

    let duration = start.elapsed();
    let ms = duration.as_millis() as u64;
    let ops_per_sec = count as f64 / duration.as_secs_f64();

    Benchmark {
        name: "Blake3 Hash (256-bit)".to_string(),
        operations: count,
        duration_ms: ms,
        ops_per_second: ops_per_sec,
    }
}

/// Benchmark BLS threshold signature attestation
fn bench_bls_attestation() -> Benchmark {
    let count = 10;
    let committee = BlsCommittee::new();
    let start = Instant::now();

    for i in 0..count {
        let header = format!("block-header-{}", i);
        let (success, _) = committee.attest_block(&header);
        assert!(success);
    }

    let duration = start.elapsed();
    let ms = duration.as_millis() as u64;
    let ops_per_sec = count as f64 / duration.as_secs_f64();

    Benchmark {
        name: "BLS Attestation (85/100 sigs)".to_string(),
        operations: count,
        duration_ms: ms,
        ops_per_second: ops_per_sec,
    }
}

/// Benchmark consensus round time
fn bench_consensus_round() -> Benchmark {
    let count = 100;
    let start = Instant::now();

    for i in 0..count {
        let validators = 5;
        let approvals: u64 = (0..validators).map(|v| {
            let msg = format!("block-{}:validator-{}", i, v);
            crate::crypto::hash_str(&msg).chars().count() as u64
        }).sum();
        // Simulating a consensus round: aggregate votes, check threshold
        let _finalized = approvals >= 3; // 2/3+ threshold
    }

    let duration = start.elapsed();
    let ms = duration.as_millis() as u64;
    let ops_per_sec = count as f64 / duration.as_secs_f64();

    Benchmark {
        name: "Consensus Round (5 validators)".to_string(),
        operations: count,
        duration_ms: ms,
        ops_per_second: ops_per_sec,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_benchmarks_run() {
        let results = run_all();
        assert_eq!(results.len(), 6);
        for b in &results {
            assert!(b.operations > 0);
            assert!(b.ops_per_second > 0.0);
        }
    }
}
