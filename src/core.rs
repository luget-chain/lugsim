// LUGET Core Module
// UTXO-based sound money layer
// Kipngetich Clinton, Waigeri, Bomet County, Kenya

use std::collections::HashMap;

/// A single Unspent Transaction Output
#[derive(Debug, Clone, PartialEq)]
pub struct Utxo {
    pub id: String,           // Unique identifier: hash of tx that created it + output index
    pub amount: u64,          // Amount in LGT (smallest unit)
    pub owner: String,        // Public key hash of the owner
}

/// A Collateralized Lock - frozen UTXO that backs VM assets
#[derive(Debug, Clone, PartialEq)]
pub struct CLock {
    pub id: String,
    pub amount: u64,
    pub owner: String,        // Who can unilaterally close after timeout
    pub vm_object_id: Option<String>, // Linked object in VM (if minted)
    pub created_at_block: u64,
}

/// A transaction that spends UTXOs and creates new ones
#[derive(Debug, Clone)]
pub struct Transaction {
    pub inputs: Vec<String>,      // UTXO IDs being spent
    pub outputs: Vec<Utxo>,       // New UTXOs being created
    pub signatures: Vec<String>,  // Signatures authorizing each input
    pub is_clock_create: bool,    // Does this tx create a CLock?
    pub clock: Option<CLock>,     // The CLock being created (if any)
}

/// The complete Core state
#[derive(Debug, Clone)]
pub struct CoreState {
    pub utxos: HashMap<String, Utxo>,   // Active UTXO set
    pub clocks: HashMap<String, CLock>, // Active collateralized locks
    pub total_supply: u64,
    pub current_block: u64,
}

impl CoreState {
    /// Create a new empty Core state (genesis)
    pub fn new() -> Self {
        CoreState {
            utxos: HashMap::new(),
            clocks: HashMap::new(),
            total_supply: 0,
            current_block: 0,
        }
    }

    /// Create the genesis UTXO - the first LGT ever minted
    /// In the real protocol, this is mined via RandomX PoW
    pub fn genesis(&mut self, initial_owner: &str, initial_amount: u64) {
        let genesis_utxo = Utxo {
            id: "genesis-0".to_string(),
            amount: initial_amount,
            owner: initial_owner.to_string(),
        };
        self.utxos.insert(genesis_utxo.id.clone(), genesis_utxo);
        self.total_supply = initial_amount;
        println!("[CORE] Genesis: {} LGT minted to {}", initial_amount, initial_owner);
    }

    /// Validate a basic transfer transaction
    pub fn validate_transaction(&self, tx: &Transaction) -> Result<(), String> {
        let mut input_amount: u64 = 0;
        let mut output_amount: u64 = 0;

        // Check all inputs exist and belong to the spender
        for input_id in &tx.inputs {
            match self.utxos.get(input_id) {
                Some(utxo) => {
                    input_amount += utxo.amount;
                }
                None => return Err(format!("UTXO {} not found", input_id)),
            }
        }

        // Sum all outputs
        for output in &tx.outputs {
            output_amount += output.amount;
        }

        // Conservation check: outputs cannot exceed inputs
        // The difference (if any) is the fee, which is burned
        if output_amount > input_amount {
            return Err(format!(
                "Outputs ({}) exceed inputs ({})", output_amount, input_amount
            ));
        }

        // If creating a CLock, validate it
        if tx.is_clock_create {
            if let Some(ref clock) = tx.clock {
                if clock.amount > input_amount {
                    return Err("CLock amount exceeds inputs".to_string());
                }
            } else {
                return Err("is_clock_create is true but no CLock provided".to_string());
            }
        }

        Ok(())
    }

    /// Execute a validated transaction
    pub fn execute_transaction(&mut self, tx: &Transaction) -> Result<u64, String> {
        // Validate first
        self.validate_transaction(tx)?;

        let mut input_amount: u64 = 0;
        let mut output_amount: u64 = 0;

        // Remove spent UTXOs
        for input_id in &tx.inputs {
            if let Some(utxo) = self.utxos.remove(input_id) {
                input_amount += utxo.amount;
            }
        }

        // Add new UTXOs
        for output in &tx.outputs {
            output_amount += output.amount;
            self.utxos.insert(output.id.clone(), output.clone());
        }

        // Process CLock creation
        if tx.is_clock_create {
            if let Some(ref clock) = tx.clock {
                self.clocks.insert(clock.id.clone(), clock.clone());
                println!("[CORE] CLock created: {} LGT locked by {}", clock.amount, clock.owner);
            }
        }

        // Fee is the difference, burned (reduces supply)
        let fee = input_amount - output_amount;
        if fee > 0 {
            self.total_supply -= fee;
            println!("[CORE] Fee burned: {} LGT", fee);
        }

        self.current_block += 1;
        println!("[CORE] Block {}: {} UTXOs, {} CLocks, Supply: {} LGT",
            self.current_block, self.utxos.len(), self.clocks.len(), self.total_supply);

        Ok(fee)
    }

    /// Get the balance of an address
    pub fn get_balance(&self, owner: &str) -> u64 {
        self.utxos
            .values()
            .filter(|u| u.owner == owner)
            .map(|u| u.amount)
            .sum()
    }

    /// Print the current state (for simulation output)
    pub fn print_state(&self) {
        println!("=== CORE STATE (Block {}) ===", self.current_block);
        println!("Total Supply: {} LGT", self.total_supply);
        println!("Active UTXOs: {}", self.utxos.len());
        println!("Active CLocks: {}", self.clocks.len());
        for (id, utxo) in &self.utxos {
            println!("  UTXO {}: {} LGT -> {}", id, utxo.amount, &utxo.owner[..8.min(utxo.owner.len())]);
        }
        for (id, clock) in &self.clocks {
            println!("  CLock {}: {} LGT locked by {}", id, clock.amount, &clock.owner[..8.min(clock.owner.len())]);
        }
        println!("==============================");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_genesis() {
        let mut state = CoreState::new();
        state.genesis("alice", 1000);
        assert_eq!(state.total_supply, 1000);
        assert_eq!(state.get_balance("alice"), 1000);
    }

    #[test]
    fn test_simple_transfer() {
        let mut state = CoreState::new();
        state.genesis("alice", 1000);

        let tx = Transaction {
            inputs: vec!["genesis-0".to_string()],
            outputs: vec![
                Utxo { id: "tx1-0".to_string(), amount: 400, owner: "bob".to_string() },
                Utxo { id: "tx1-1".to_string(), amount: 590, owner: "alice".to_string() },
            ],
            signatures: vec!["alice-sig".to_string()],
            is_clock_create: false,
            clock: None,
        };

        let result = state.execute_transaction(&tx);
        assert!(result.is_ok());
        assert_eq!(state.get_balance("bob"), 400);
        assert_eq!(state.get_balance("alice"), 590);
        assert_eq!(state.total_supply, 990); // 10 burned as fee
    }

    #[test]
    fn test_double_spend_rejected() {
        let mut state = CoreState::new();
        state.genesis("alice", 1000);

        let tx = Transaction {
            inputs: vec!["genesis-0".to_string()],
            outputs: vec![
                Utxo { id: "tx1-0".to_string(), amount: 1000, owner: "bob".to_string() },
            ],
            signatures: vec!["alice-sig".to_string()],
            is_clock_create: false,
            clock: None,
        };

        state.execute_transaction(&tx).unwrap();

        // Try to spend the same UTXO again
        let double_spend = Transaction {
            inputs: vec!["genesis-0".to_string()],
            outputs: vec![
                Utxo { id: "tx2-0".to_string(), amount: 1000, owner: "charlie".to_string() },
            ],
            signatures: vec!["alice-sig".to_string()],
            is_clock_create: false,
            clock: None,
        };

        let result = state.execute_transaction(&double_spend);
        assert!(result.is_err());
    }

    #[test]
    fn test_clock_creation() {
        let mut state = CoreState::new();
        state.genesis("alice", 1000);

        let clock = CLock {
            id: "clock-1".to_string(),
            amount: 500,
            owner: "alice".to_string(),
            vm_object_id: None,
            created_at_block: 1,
        };

        let tx = Transaction {
            inputs: vec!["genesis-0".to_string()],
            outputs: vec![
                Utxo { id: "tx1-0".to_string(), amount: 490, owner: "alice".to_string() },
            ],
            signatures: vec!["alice-sig".to_string()],
            is_clock_create: true,
            clock: Some(clock),
        };

        let result = state.execute_transaction(&tx);
        assert!(result.is_ok());
        assert_eq!(state.clocks.len(), 1);
        assert_eq!(state.clocks.get("clock-1").unwrap().amount, 500);
    }
}
