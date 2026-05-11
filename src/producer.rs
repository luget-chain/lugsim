// LUGET Block Producer Module
// Continuous block production loop
// Kipngetich Clinton, Kenya

use crate::consensus::ConsensusState;
use crate::mempool::Mempool;

/// Configuration for the block producer
#[derive(Debug, Clone)]
pub struct ProducerConfig {
    pub block_time_ms: u64,        // Target time between blocks (e.g., 12000 for 12 seconds)
    pub max_txs_per_block: usize,  // Maximum transactions per block
    pub max_rounds: u64,           // Maximum number of rounds to run (0 = infinite)
}

impl Default for ProducerConfig {
    fn default() -> Self {
        ProducerConfig {
            block_time_ms: 12000,   // 12 seconds (LUGET target block time)
            max_txs_per_block: 10,
            max_rounds: 0,          // Infinite
        }
    }
}

/// Statistics from block production
#[derive(Debug, Clone, Default)]
pub struct ProducerStats {
    pub blocks_produced: u64,
    pub blocks_finalized: u64,
    pub total_transactions: u64,
    pub total_fees: u64,
    pub skipped_rounds: u64,
    pub start_time: u64,
}

/// The block producer engine
pub struct BlockProducer {
    pub config: ProducerConfig,
    pub consensus: ConsensusState,
    pub mempool: Mempool,
    pub stats: ProducerStats,
    pub running: bool,
}

impl BlockProducer {
    /// Create a new block producer
    pub fn new(config: ProducerConfig, consensus: ConsensusState, mempool: Mempool) -> Self {
        BlockProducer {
            config,
            consensus,
            mempool,
            stats: ProducerStats::default(),
            running: false,
        }
    }

    /// Run a single block production round
    pub fn produce_block(&mut self) -> Result<bool, String> {
        // Select transactions from mempool
        let txs = self.mempool.select_for_block(self.config.max_txs_per_block);
        
        if txs.is_empty() {
            self.stats.skipped_rounds += 1;
            println!("[PRODUCER] No transactions in mempool. Skipping round.");
            return Ok(false);
        }

        // Convert transactions to strings for consensus
        let tx_strings: Vec<String> = txs.iter()
            .map(|tx| format!("{}→{}:{}", tx.from.chars().take(8).collect::<String>(), tx.to.chars().take(8).collect::<String>(), tx.amount))
            .collect();

        let tx_count = tx_strings.len();
        let round_fees: u64 = txs.iter().map(|tx| tx.fee).sum();

        // Run consensus round
        let finalized = self.consensus.run_round(tx_strings)?;

        if finalized {
            self.stats.blocks_produced += 1;
            self.stats.blocks_finalized += 1;
            self.stats.total_transactions += tx_count as u64;
            self.stats.total_fees += round_fees;

            println!("[PRODUCER] Block {} produced and finalized ({} txs, {} LGT fees)",
                self.consensus.finalized_height, tx_count, round_fees);
        }

        Ok(finalized)
    }

    /// Run the block producer in a loop (simulated; no real threading needed for Simulator)
    pub fn run_loop(&mut self) {
        self.running = true;
        self.stats.start_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        println!("[PRODUCER] Block production started.");
        println!("[PRODUCER] Block time: {}ms, Max txs/block: {}",
            self.config.block_time_ms, self.config.max_txs_per_block);

        let max_rounds = if self.config.max_rounds == 0 {
            u64::MAX
        } else {
            self.config.max_rounds
        };

        for round in 1..=max_rounds {
            if !self.running {
                break;
            }

            println!("\n--- Round {} ---", round);
            
            match self.produce_block() {
                Ok(true) => {
                    // Block produced successfully
                }
                Ok(false) => {
                    // No transactions, will retry
                    if round > 3 && self.mempool.pending_count() == 0 {
                        println!("[PRODUCER] Mempool drained. Stopping production.");
                        break;
                    }
                }
                Err(e) => {
                    println!("[PRODUCER] Error: {}", e);
                }
            }

            // In production: sleep for block_time_ms
            // For simulator: just continue
        }

        self.running = false;
        println!("[PRODUCER] Block production stopped.");
    }

    /// Print production statistics
    pub fn print_stats(&self) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let elapsed = now - self.stats.start_time;
        let tps = if elapsed > 0 {
            self.stats.total_transactions as f64 / elapsed as f64
        } else {
            0.0
        };

        println!("=== PRODUCER STATS ===");
        println!("Blocks Produced:   {}", self.stats.blocks_produced);
        println!("Blocks Finalized:  {}", self.stats.blocks_finalized);
        println!("Total Txs:         {}", self.stats.total_transactions);
        println!("Total Fees:        {} LGT", self.stats.total_fees);
        println!("Skipped Rounds:    {}", self.stats.skipped_rounds);
        println!("Runtime:           {}s", elapsed);
        println!("Throughput:        {:.1} txs/s", tps);
        println!("======================");
    }

    /// Stop the block producer
    pub fn stop(&mut self) {
        self.running = false;
        println!("[PRODUCER] Stopping block production...");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::consensus::ConsensusState;
    use crate::mempool::Mempool;

    fn setup_producer() -> BlockProducer {
        let mut consensus = ConsensusState::new();
        consensus.add_validator("v1", 100);
        consensus.add_validator("v2", 100);
        consensus.add_validator("v3", 100);

        let mut mempool = Mempool::new(100, 1);
        
        BlockProducer::new(
            ProducerConfig {
                block_time_ms: 100,
                max_txs_per_block: 5,
                max_rounds: 3,
            },
            consensus,
            mempool,
        )
    }

    #[test]
    fn test_producer_creation() {
        let producer = setup_producer();
        assert_eq!(producer.consensus.validators.len(), 3);
        assert!(!producer.running);
    }

    #[test]
    fn test_produce_empty_block() {
        let mut producer = setup_producer();
        // No transactions in mempool
        let result = producer.produce_block();
        assert!(result.is_ok());
        assert!(!result.unwrap()); // false = no block produced
        assert_eq!(producer.stats.skipped_rounds, 1);
    }

    #[test]
    fn test_produce_block_with_txs() {
        let mut producer = setup_producer();
        
        // Add transactions to mempool
        producer.mempool.submit("alice", "bob", 100, 5, 0, vec![1,2,3]).unwrap();
        producer.mempool.submit("charlie", "dave", 50, 3, 0, vec![4,5,6]).unwrap();

        let result = producer.produce_block();
        assert!(result.is_ok());
        assert!(result.unwrap()); // true = block produced
        assert_eq!(producer.stats.blocks_produced, 1);
        assert_eq!(producer.stats.total_transactions, 2);
    }

    #[test]
    fn test_multiple_blocks() {
        let mut producer = setup_producer();

        for i in 0..5 {
            producer.mempool.submit(
                "alice", "bob", 10 * i, 5, i as u64, vec![i as u8]
            ).unwrap();
        }

        // Run producer loop (max 3 rounds)
        producer.run_loop();

        assert!(producer.stats.blocks_produced > 0);
        assert!(producer.stats.total_transactions > 0);
    }

    #[test]
    fn test_producer_stats() {
        let mut producer = setup_producer();
        producer.mempool.submit("alice", "bob", 100, 5, 0, vec![1]).unwrap();
        producer.produce_block().unwrap();
        producer.print_stats(); // Should not panic
    }
}
