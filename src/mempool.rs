// LUGET Mempool Module
// Shared pending transaction pool
// Kipngetich Clinton, Kenya

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// A transaction waiting to be included in a block
#[derive(Debug, Clone, PartialEq)]
pub struct PoolTransaction {
    pub id: String,              // Blake3 hash of the transaction
    pub from: String,            // Sender address
    pub to: String,              // Recipient address
    pub amount: u64,             // Amount in LGT
    pub fee: u64,                // Fee offered to validators
    pub nonce: u64,              // Sender's transaction count (prevents replay)
    pub signature: Vec<u8>,      // Ed25519 signature
    pub submitted_at: u64,       // Unix timestamp
    pub priority: u64,           // fee / size ratio for ordering
}

/// The mempool state
#[derive(Debug, Clone)]
pub struct Mempool {
    pub transactions: HashMap<String, PoolTransaction>,
    pub max_size: usize,
    pub min_fee: u64,            // Minimum fee to enter mempool
    pub total_pending: usize,
    pub total_fees: u64,
    pub rejected_count: u64,
}

impl Mempool {
    /// Create a new mempool
    pub fn new(max_size: usize, min_fee: u64) -> Self {
        Mempool {
            transactions: HashMap::new(),
            max_size,
            min_fee,
            total_pending: 0,
            total_fees: 0,
            rejected_count: 0,
        }
    }

    /// Submit a transaction to the mempool
    pub fn submit(
        &mut self,
        from: &str,
        to: &str,
        amount: u64,
        fee: u64,
        nonce: u64,
        signature: Vec<u8>,
    ) -> Result<String, String> {
        // Check minimum fee
        if fee < self.min_fee {
            self.rejected_count += 1;
            return Err(format!(
                "Fee too low: {} LGT offered, {} LGT minimum",
                fee, self.min_fee
            ));
        }

        // Check mempool is not full
        if self.transactions.len() >= self.max_size {
            self.rejected_count += 1;
            return Err(format!(
                "Mempool full: {}/{} transactions",
                self.transactions.len(), self.max_size
            ));
        }

        // Generate transaction ID
        let tx_data = format!("{}:{}:{}:{}:{}", from, to, amount, fee, nonce);
        let tx_id = crate::crypto::hash_str(&tx_data);

        // Check for duplicate
        if self.transactions.contains_key(&tx_id) {
            return Err("Duplicate transaction".to_string());
        }

        // Calculate priority (fee per unit, simplified)
        let priority = fee;

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let tx = PoolTransaction {
            id: tx_id.clone(),
            from: from.to_string(),
            to: to.to_string(),
            amount,
            fee,
            nonce,
            signature,
            submitted_at: now,
            priority,
        };

        self.transactions.insert(tx_id.clone(), tx);
        self.total_pending += 1;
        self.total_fees += fee;

        println!("[MEMPOOL] Tx {}: {}→{} {} LGT (fee: {})", 
            &tx_id[..16], from, to, amount, fee);

        Ok(tx_id)
    }

    /// Get the highest-priority transactions for inclusion in a block
    pub fn select_for_block(&mut self, max_count: usize) -> Vec<PoolTransaction> {
        // Sort by priority (highest fee first)
        let mut txs: Vec<PoolTransaction> = self.transactions.values().cloned().collect();
        txs.sort_by(|a, b| b.priority.cmp(&a.priority));

        // Take the top N
        let selected: Vec<PoolTransaction> = txs.into_iter().take(max_count).collect();

        // Remove selected transactions from mempool
        for tx in &selected {
            self.transactions.remove(&tx.id);
            self.total_pending -= 1;
            self.total_fees -= tx.fee;
        }

        println!("[MEMPOOL] Selected {} txs for block ({} remaining)",
            selected.len(), self.transactions.len());

        selected
    }

    /// Remove a transaction by ID (e.g., if included in a block by another validator)
    pub fn remove(&mut self, tx_id: &str) -> Option<PoolTransaction> {
        if let Some(tx) = self.transactions.remove(tx_id) {
            self.total_pending -= 1;
            self.total_fees -= tx.fee;
            Some(tx)
        } else {
            None
        }
    }

    /// Remove transactions from a specific sender (for nonce management)
    pub fn remove_by_sender(&mut self, from: &str) -> Vec<PoolTransaction> {
        let to_remove: Vec<String> = self.transactions
            .iter()
            .filter(|(_, tx)| tx.from == from)
            .map(|(id, _)| id.clone())
            .collect();

        let mut removed = Vec::new();
        for id in to_remove {
            if let Some(tx) = self.remove(&id) {
                removed.push(tx);
            }
        }
        removed
    }

    /// Check if a transaction exists
    pub fn contains(&self, tx_id: &str) -> bool {
        self.transactions.contains_key(tx_id)
    }

    /// Get pending count
    pub fn pending_count(&self) -> usize {
        self.transactions.len()
    }

    /// Get total fees in mempool
    pub fn pending_fees(&self) -> u64 {
        self.total_fees
    }

    /// Clear expired transactions (older than `max_age_secs`)
    pub fn clear_expired(&mut self, max_age_secs: u64) -> usize {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let to_remove: Vec<String> = self.transactions
            .iter()
            .filter(|(_, tx)| now - tx.submitted_at > max_age_secs)
            .map(|(id, _)| id.clone())
            .collect();

        let count = to_remove.len();
        for id in &to_remove {
            self.remove(id);
        }

        if count > 0 {
            println!("[MEMPOOL] Cleared {} expired transactions", count);
        }

        count
    }

    /// Broadcast a transaction to peer nodes (placeholder for P2P integration)
    pub fn broadcast_to_peers(&self, tx_id: &str) {
        if let Some(_tx) = self.transactions.get(tx_id) {
            println!("[MEMPOOL] Broadcasting tx {} to peers", &tx_id[..16]);
            // In production: send to all connected peers via P2P
        }
    }

    /// Print mempool state
    pub fn print_state(&self) {
        println!("=== MEMPOOL ===");
        println!("Pending: {} / {}", self.transactions.len(), self.max_size);
        println!("Total Fees: {} LGT", self.total_fees);
        println!("Rejected: {}", self.rejected_count);
        if !self.transactions.is_empty() {
            println!("Transactions:");
            for (id, tx) in &self.transactions {
                println!("  {}: {}→{} {} LGT (fee: {}, prio: {})",
                    &id[..16], tx.from, tx.to, tx.amount, tx.fee, tx.priority);
            }
        }
        println!("===============");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_submit_transaction() {
        let mut mempool = Mempool::new(100, 1);
        let result = mempool.submit("alice", "bob", 100, 5, 0, vec![1,2,3]);
        assert!(result.is_ok());
        assert_eq!(mempool.pending_count(), 1);
    }

    #[test]
    fn test_fee_too_low_rejected() {
        let mut mempool = Mempool::new(100, 10);
        let result = mempool.submit("alice", "bob", 100, 1, 0, vec![1,2,3]);
        assert!(result.is_err());
        assert_eq!(mempool.rejected_count, 1);
    }

    #[test]
    fn test_mempool_full_rejected() {
        let mut mempool = Mempool::new(2, 1);
        mempool.submit("alice", "bob", 10, 5, 0, vec![1,2,3]).unwrap();
        mempool.submit("charlie", "dave", 20, 5, 0, vec![4,5,6]).unwrap();
        let result = mempool.submit("eve", "frank", 30, 5, 0, vec![7,8,9]);
        assert!(result.is_err());
    }

    #[test]
    fn test_select_for_block() {
        let mut mempool = Mempool::new(100, 1);
        mempool.submit("alice", "bob", 10, 1, 0, vec![1]).unwrap();
        mempool.submit("charlie", "dave", 20, 10, 0, vec![2]).unwrap(); // Higher fee
        mempool.submit("eve", "frank", 30, 5, 0, vec![3]).unwrap();

        let selected = mempool.select_for_block(2);
        assert_eq!(selected.len(), 2);
        // Highest fee should be first
        assert_eq!(selected[0].fee, 10);
        assert_eq!(mempool.pending_count(), 1);
    }

    #[test]
    fn test_duplicate_rejected() {
        let mut mempool = Mempool::new(100, 1);
        mempool.submit("alice", "bob", 10, 5, 0, vec![1,2,3]).unwrap();
        let result = mempool.submit("alice", "bob", 10, 5, 0, vec![1,2,3]);
        assert!(result.is_err());
    }

    #[test]
    fn test_clear_expired() {
        let mut mempool = Mempool::new(100, 1);
        mempool.submit("alice", "bob", 10, 5, 0, vec![1]).unwrap();
        // Clear with max_age of 0 (all are older than 0 seconds)
        std::thread::sleep(std::time::Duration::from_secs(1)); let cleared = mempool.clear_expired(1);
        assert_eq!(cleared, 0); // Max age 0: nothing expires instantly
        assert_eq!(mempool.pending_count(), 1); // Transaction still pending
    }

    #[test]
    fn test_remove_by_sender() {
        let mut mempool = Mempool::new(100, 1);
        mempool.submit("alice", "bob", 10, 5, 0, vec![1]).unwrap();
        mempool.submit("alice", "charlie", 20, 5, 0, vec![2]).unwrap();
        mempool.submit("bob", "dave", 30, 5, 0, vec![3]).unwrap();

        let removed = mempool.remove_by_sender("alice");
        assert_eq!(removed.len(), 2);
        assert_eq!(mempool.pending_count(), 1);
    }
}
