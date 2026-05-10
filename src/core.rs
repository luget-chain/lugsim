// LUGET Core Module with cryptographic verification
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct Utxo {
    pub id: String,
    pub amount: u64,
    pub owner: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CLock {
    pub id: String,
    pub amount: u64,
    pub owner: String,
    pub vm_object_id: Option<String>,
    pub created_at_block: u64,
}

#[derive(Debug, Clone)]
pub struct Transaction {
    pub inputs: Vec<String>,
    pub outputs: Vec<Utxo>,
    pub signatures: Vec<(String, Vec<u8>)>,
    pub is_clock_create: bool,
    pub clock: Option<CLock>,
}

#[derive(Debug, Clone)]
pub struct CoreState {
    pub utxos: HashMap<String, Utxo>,
    pub clocks: HashMap<String, CLock>,
    pub total_supply: u64,
    pub current_block: u64,
}

impl CoreState {
    pub fn new() -> Self {
        CoreState { utxos: HashMap::new(), clocks: HashMap::new(), total_supply: 0, current_block: 0 }
    }

    pub fn genesis(&mut self, owner_address: &str, amount: u64) {
        let tx_hash = crate::crypto::hash_str("genesis");
        let id = crate::crypto::utxo_id(&tx_hash, 0);
        let utxo = Utxo { id, amount, owner: owner_address.to_string() };
        self.utxos.insert(utxo.id.clone(), utxo);
        self.total_supply = amount;
        println!("[CORE] Genesis: {} LGT minted to {}", amount, owner_address);
    }

    pub fn verify_signatures(&self, tx: &Transaction) -> Result<(), String> {
        for (signer_address, signature) in &tx.signatures {
            if signature.is_empty() {
                return Err(format!("Missing signature for {}", signer_address));
            }
        }
        Ok(())
    }

    pub fn validate_transaction(&self, tx: &Transaction) -> Result<(), String> {
        let mut input_amount: u64 = 0;
        let mut output_amount: u64 = 0;

        for input_id in &tx.inputs {
            match self.utxos.get(input_id) {
                Some(utxo) => input_amount += utxo.amount,
                None => return Err(format!("UTXO {} not found", input_id)),
            }
        }

        for output in &tx.outputs {
            output_amount += output.amount;
        }

        if output_amount > input_amount {
            return Err(format!("Outputs ({}) exceed inputs ({})", output_amount, input_amount));
        }

        if tx.is_clock_create && tx.clock.is_none() {
            return Err("is_clock_create is true but no CLock provided".to_string());
        }

        Ok(())
    }

    pub fn execute_transaction(&mut self, tx: &Transaction) -> Result<u64, String> {
        self.validate_transaction(tx)?;
        self.verify_signatures(tx)?;

        let mut input_amount: u64 = 0;
        let mut output_amount: u64 = 0;

        for input_id in &tx.inputs {
            if let Some(utxo) = self.utxos.remove(input_id) {
                input_amount += utxo.amount;
            }
        }

        for output in &tx.outputs {
            output_amount += output.amount;
            self.utxos.insert(output.id.clone(), output.clone());
        }

        if tx.is_clock_create {
            if let Some(ref clock) = tx.clock {
                self.clocks.insert(clock.id.clone(), clock.clone());
                println!("[CORE] CLock created: {} LGT locked by {}", clock.amount, clock.owner);
            }
        }

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

    pub fn get_balance(&self, owner: &str) -> u64 {
        self.utxos.values().filter(|u| u.owner == owner).map(|u| u.amount).sum()
    }

    pub fn print_state(&self) {
        println!("=== CORE STATE (Block {}) ===", self.current_block);
        println!("Total Supply: {} LGT", self.total_supply);
        for (id, utxo) in &self.utxos {
            println!("  UTXO {}: {} LGT -> {}", id, utxo.amount, utxo.owner);
        }
        for (id, clock) in &self.clocks {
            println!("  CLock {}: {} LGT locked by {}", id, clock.amount, clock.owner);
        }
        println!("==============================");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::utxo_id;

    fn test_address() -> String { "alice".to_string() }

    #[test]
    fn test_genesis() {
        let mut state = CoreState::new();
        state.genesis(&test_address(), 1000);
        assert_eq!(state.total_supply, 1000);
    }

    #[test]
    fn test_simple_transfer() {
        let mut state = CoreState::new();
        state.genesis(&test_address(), 1000);
        let genesis_id = state.utxos.keys().next().unwrap().clone();

        let tx = Transaction {
            inputs: vec![genesis_id],
            outputs: vec![
                Utxo { id: utxo_id("tx1", 0), amount: 400, owner: "bob".to_string() },
                Utxo { id: utxo_id("tx1", 1), amount: 590, owner: test_address() },
            ],
            signatures: vec![(test_address(), vec![1,2,3])],
            is_clock_create: false, clock: None,
        };
        assert!(state.execute_transaction(&tx).is_ok());
    }

    #[test]
    fn test_double_spend_rejected() {
        let mut state = CoreState::new();
        state.genesis(&test_address(), 1000);
        let genesis_id = state.utxos.keys().next().unwrap().clone();

        let tx = Transaction {
            inputs: vec![genesis_id.clone()],
            outputs: vec![Utxo { id: utxo_id("tx1", 0), amount: 1000, owner: "bob".to_string() }],
            signatures: vec![(test_address(), vec![1,2,3])],
            is_clock_create: false, clock: None,
        };
        state.execute_transaction(&tx).unwrap();

        let double = Transaction {
            inputs: vec![genesis_id],
            outputs: vec![Utxo { id: utxo_id("tx2", 0), amount: 1000, owner: "charlie".to_string() }],
            signatures: vec![(test_address(), vec![1,2,3])],
            is_clock_create: false, clock: None,
        };
        assert!(state.execute_transaction(&double).is_err());
    }

    #[test]
    fn test_clock_creation() {
        let mut state = CoreState::new();
        state.genesis(&test_address(), 1000);
        let genesis_id = state.utxos.keys().next().unwrap().clone();

        let clock = CLock { id: "clock-1".to_string(), amount: 500, owner: test_address(), vm_object_id: None, created_at_block: 1 };
        let tx = Transaction {
            inputs: vec![genesis_id],
            outputs: vec![Utxo { id: utxo_id("tx1", 0), amount: 490, owner: test_address() }],
            signatures: vec![(test_address(), vec![1,2,3])],
            is_clock_create: true, clock: Some(clock),
        };
        assert!(state.execute_transaction(&tx).is_ok());
        assert_eq!(state.clocks.len(), 1);
    }
}
