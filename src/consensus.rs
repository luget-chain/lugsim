// LUGET Consensus Module
// BFT consensus engine: block proposal, attestation, finality
// Kipngetich Clinton, Waigeri, Bomet County, Kenya

use std::collections::HashMap;
use crate::crypto::{hash_str, KeyPair};

/// A validator in the consensus set
#[derive(Debug, Clone)]
pub struct ConsensusValidator {
    pub id: String,
    pub keypair: KeyPair,
    pub stake: u64,
    pub is_proposer: bool,
}

/// A block proposed by the leader
#[derive(Debug, Clone)]
pub struct Block {
    pub height: u64,
    pub proposer: String,
    pub previous_hash: String,
    pub transactions: Vec<String>,
    pub timestamp: u64,
    pub signature: Vec<u8>,  // Proposer's Ed25519 signature
}

impl Block {
    /// Create a new block proposal
    pub fn new(height: u64, proposer: &str, previous_hash: &str, transactions: Vec<String>) -> Self {
        Block {
            height,
            proposer: proposer.to_string(),
            previous_hash: previous_hash.to_string(),
            transactions,
            timestamp: 0, // Will be set when signed
            signature: Vec::new(),
        }
    }

    /// Compute the block hash (what validators sign)
    pub fn block_hash(&self) -> String {
        let data = format!("{}:{}:{}:{:?}:{}",
            self.height, self.proposer, self.previous_hash, self.transactions, self.timestamp);
        hash_str(&data)
    }

    /// Sign the block with the proposer's keypair
    pub fn sign(&mut self, keypair: &KeyPair) {
        self.timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let hash = self.block_hash();
        self.signature = keypair.sign(hash.as_bytes());
    }

    /// Verify the block signature
    pub fn verify(&self, public_key: &ed25519_dalek::VerifyingKey) -> bool {
        let hash = self.block_hash();
        KeyPair::verify(public_key, hash.as_bytes(), &self.signature)
    }
}

/// An attestation (vote) on a proposed block
#[derive(Debug, Clone)]
pub struct Attestation {
    pub block_height: u64,
    pub block_hash: String,
    pub validator_id: String,
    pub signature: Vec<u8>,
    pub approve: bool,
}

impl Attestation {
    /// Create a new attestation
    pub fn new(block: &Block, validator: &ConsensusValidator, approve: bool) -> Self {
        let message = format!("{}:{}:{}", block.height, block.block_hash(), approve);
        let signature = validator.keypair.sign(message.as_bytes());
        Attestation {
            block_height: block.height,
            block_hash: block.block_hash(),
            validator_id: validator.id.clone(),
            signature,
            approve,
        }
    }

    /// Verify this attestation
    pub fn verify(&self, public_key: &ed25519_dalek::VerifyingKey) -> bool {
        let message = format!("{}:{}:{}", self.block_height, self.block_hash, self.approve);
        KeyPair::verify(public_key, message.as_bytes(), &self.signature)
    }
}

/// The complete consensus state
#[derive(Debug, Clone)]
pub struct ConsensusState {
    pub validators: Vec<ConsensusValidator>,
    pub current_height: u64,
    pub current_proposer_index: usize,
    pub blocks: HashMap<u64, Block>,
    pub attestations: HashMap<u64, Vec<Attestation>>,
    pub finalized_height: u64,
    pub total_stake: u64,
    pub threshold: f64,  // 2/3 for BFT finality
}

impl ConsensusState {
    /// Create a new consensus state with genesis
    pub fn new() -> Self {
        ConsensusState {
            validators: Vec::new(),
            current_height: 0,
            current_proposer_index: 0,
            blocks: HashMap::new(),
            attestations: HashMap::new(),
            finalized_height: 0,
            total_stake: 0,
            threshold: 2.0 / 3.0,
        }
    }

    /// Add a validator to the consensus set
    pub fn add_validator(&mut self, id: &str, stake: u64) -> &ConsensusValidator {
        let keypair = KeyPair::generate();
        let validator = ConsensusValidator {
            id: id.to_string(),
            keypair,
            stake,
            is_proposer: false,
        };
        self.total_stake += stake;
        self.validators.push(validator);
        self.validators.last().unwrap()
    }

    /// Select the next proposer (round-robin for now, weighted by stake in production)
    pub fn select_proposer(&mut self) -> String {
        if self.validators.is_empty() {
            return "none".to_string();
        }
        // Reset all proposer flags
        for v in &mut self.validators {
            v.is_proposer = false;
        }
        let index = self.current_proposer_index % self.validators.len();
        self.validators[index].is_proposer = true;
        self.current_proposer_index += 1;
        self.validators[index].id.clone()
    }

    /// Propose a new block
    pub fn propose_block(&mut self, transactions: Vec<String>) -> Result<Block, String> {
        let proposer_id = self.select_proposer();
        let proposer = self.validators
            .iter()
            .find(|v| v.id == proposer_id)
            .ok_or("Proposer not found")?;

        let previous_hash = if self.current_height == 0 {
            "genesis".to_string()
        } else {
            self.blocks
                .get(&self.current_height)
                .map(|b| b.block_hash())
                .unwrap_or_else(|| "unknown".to_string())
        };

        let new_height = self.current_height + 1;
        let mut block = Block::new(new_height, &proposer_id, &previous_hash, transactions);
        block.sign(&proposer.keypair);

        // Store the block
        self.blocks.insert(new_height, block.clone());
        self.attestations.insert(new_height, Vec::new());

        println!("[CONSENSUS] Block {} proposed by {}", new_height, proposer_id);
        println!("[CONSENSUS] Block hash: {}", &block.block_hash()[..32]);

        Ok(block)
    }

    /// A validator attests to a proposed block
    pub fn attest(&mut self, block_height: u64, validator_id: &str, approve: bool) -> Result<Attestation, String> {
        let block = self.blocks
            .get(&block_height)
            .ok_or("Block not found")?;

        let validator = self.validators
            .iter()
            .find(|v| v.id == validator_id)
            .ok_or("Validator not found")?;

        let attestation = Attestation::new(block, validator, approve);

        // Verify it immediately
        if !attestation.verify(&validator.keypair.public_key) {
            return Err("Attestation signature verification failed".to_string());
        }

        self.attestations
            .get_mut(&block_height)
            .ok_or("Attestation pool not found")?
            .push(attestation.clone());

        println!("[CONSENSUS] {} attests block {}: {}",
            validator_id, block_height, if approve { "APPROVE ✓" } else { "REJECT ✗" });

        Ok(attestation)
    }

    /// Check if a block has reached BFT finality (2/3 of stake)
    pub fn check_finality(&self, block_height: u64) -> bool {
        let attestations = match self.attestations.get(&block_height) {
            Some(a) => a,
            None => return false,
        };

        let approved_stake: u64 = attestations
            .iter()
            .filter(|a| a.approve)
            .filter_map(|a| {
                self.validators
                    .iter()
                    .find(|v| v.id == a.validator_id)
                    .map(|v| v.stake)
            })
            .sum();

        let ratio = approved_stake as f64 / self.total_stake as f64;
        let finalized = ratio >= self.threshold;

        if finalized {
            println!("[CONSENSUS] Block {} FINALIZED: {:.1}% of stake approved",
                block_height, ratio * 100.0);
        }

        finalized
    }

    /// Finalize a block if threshold is met
    pub fn finalize_block(&mut self, block_height: u64) -> Result<bool, String> {
        if !self.check_finality(block_height) {
            return Ok(false);
        }

        self.finalized_height = block_height;
        self.current_height = block_height;
        println!("[CONSENSUS] Block {} committed to chain", block_height);
        Ok(true)
    }

    /// Byzentine fault tolerance check: can we tolerate f faults with current stake?
    pub fn max_fault_tolerance(&self) -> u64 {
        // BFT requires honest stake > 2/3, so faults < 1/3
        (self.total_stake as f64 * (1.0 - self.threshold)) as u64
    }

    /// Run a complete consensus round
    pub fn run_round(&mut self, transactions: Vec<String>) -> Result<bool, String> {
        println!("\n═══════════ CONSENSUS ROUND {} ═══════════", self.current_height + 1);

        // Step 1: Propose
        let block = self.propose_block(transactions)?;

        // Step 2: All validators attest
        let validator_ids: Vec<String> = self.validators.iter().map(|v| v.id.clone()).collect();
        for vid in &validator_ids {
            // Honest validators approve; a byzantine one could reject
            let approve = true;
            self.attest(block.height, vid, approve)?;
        }

        // Step 3: Check finality
        let finalized = self.finalize_block(block.height)?;

        if finalized {
            println!("[CONSENSUS] Round {} complete. Chain height: {}",
                block.height, self.finalized_height);
        } else {
            println!("[CONSENSUS] Round {} failed: insufficient attestations", block.height);
        }

        Ok(finalized)
    }

    /// Print consensus state
    pub fn print_state(&self) {
        println!("=== CONSENSUS STATE ===");
        println!("Current Height:  {}", self.current_height);
        println!("Finalized Height: {}", self.finalized_height);
        println!("Validators: {} (total stake: {} LGT)", self.validators.len(), self.total_stake);
        println!("Fault Tolerance: {} LGT ({} can be byzantine)", self.max_fault_tolerance(), self.validators.len() / 3);
        for v in &self.validators {
            let role = if v.is_proposer { "PROPOSER" } else { "validator" };
            println!("  {} — {} LGT — {}", v.id, v.stake, role);
        }
        println!("Blocks stored: {}", self.blocks.len());
        for (height, block) in &self.blocks {
            let finalized = if *height <= self.finalized_height { "✓" } else { " " };
            println!("  Block {} [{}] — proposer: {}, txs: {}",
                height, finalized, block.proposer, block.transactions.len());
        }
        println!("==============================");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_validator() {
        let mut cs = ConsensusState::new();
        cs.add_validator("v1", 100);
        assert_eq!(cs.validators.len(), 1);
        assert_eq!(cs.total_stake, 100);
    }

    #[test]
    fn test_propose_and_atest() {
        let mut cs = ConsensusState::new();
        cs.add_validator("v1", 100);
        cs.add_validator("v2", 100);
        cs.add_validator("v3", 100);
        cs.add_validator("v4", 100);

        let block = cs.propose_block(vec!["tx1".to_string()]).unwrap();
        assert_eq!(block.height, 1);

        // All 4 validators attest
        cs.attest(1, "v1", true).unwrap();
        cs.attest(1, "v2", true).unwrap();
        cs.attest(1, "v3", true).unwrap();
        cs.attest(1, "v4", true).unwrap();

        assert!(cs.check_finality(1));
    }

    #[test]
    fn test_byzantine_validator() {
        let mut cs = ConsensusState::new();
        cs.add_validator("v1", 100); // honest
        cs.add_validator("v2", 100); // honest
        cs.add_validator("v3", 100); // honest
        cs.add_validator("v4", 100); // byzantine

        let block = cs.propose_block(vec!["tx1".to_string()]).unwrap();

        // 3 approve, 1 rejects
        cs.attest(1, "v1", true).unwrap();
        cs.attest(1, "v2", true).unwrap();
        cs.attest(1, "v3", true).unwrap();
        cs.attest(1, "v4", false).unwrap();

        // 3/4 = 75% > 66.7% threshold
        assert!(cs.check_finality(1));
    }

    #[test]
    fn test_consensus_round() {
        let mut cs = ConsensusState::new();
        cs.add_validator("v1", 100);
        cs.add_validator("v2", 100);
        cs.add_validator("v3", 100);

        let result = cs.run_round(vec!["alice→bob:50".to_string()]);
        assert!(result.is_ok());
        assert!(result.unwrap());
        assert_eq!(cs.finalized_height, 1);
    }

    #[test]
    fn test_multiple_rounds() {
        let mut cs = ConsensusState::new();
        cs.add_validator("v1", 100);
        cs.add_validator("v2", 100);
        cs.add_validator("v3", 100);

        cs.run_round(vec!["tx-a".to_string()]).unwrap();
        cs.run_round(vec!["tx-b".to_string()]).unwrap();
        cs.run_round(vec!["tx-c".to_string()]).unwrap();

        assert_eq!(cs.finalized_height, 3);
        assert_eq!(cs.blocks.len(), 3);
    }

    #[test]
    fn test_signature_verification() {
        let mut cs = ConsensusState::new();
        cs.add_validator("v1", 100);

        let block = cs.propose_block(vec!["genesis-tx".to_string()]).unwrap();
        let validator = &cs.validators[0];

        assert!(block.verify(&validator.keypair.public_key));
    }

    #[test]
    fn test_tampered_block_rejected() {
        let mut cs = ConsensusState::new();
        cs.add_validator("v1", 100);
        cs.add_validator("v2", 100);

        let mut block = cs.propose_block(vec!["tx1".to_string()]).unwrap();

        // Tamper with the block
        block.transactions.push("fake-tx".to_string());

        let validator = &cs.validators[0];
        assert!(!block.verify(&validator.keypair.public_key));
    }
}
