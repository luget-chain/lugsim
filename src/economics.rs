// LUGET Validator Economics Module
// Dual-pool revenue, uptime multiplier, slashing, MEV redistribution
// Kipngetich Clinton, Waigeri, Bomet County, Kenya

/// Represents a single validator in the network
#[derive(Debug, Clone)]
pub struct Validator {
    pub id: String,
    pub address: String,
    pub self_bonded_stake: u64,      // Validator's own LGT at risk
    pub delegated_stake: u64,        // LGT from delegators
    pub uptime_score: f64,           // 0.0 to 1.0
    pub jailed: bool,
    pub slash_history: Vec<SlashEvent>,
}

/// A slashing event recorded on-chain
#[derive(Debug, Clone)]
pub struct SlashEvent {
    pub reason: SlashReason,
    pub amount_burned: u64,
    pub epoch: u64,
    pub affected_self_bonded: bool,
    pub affected_delegated: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SlashReason {
    DoubleSequencing,       // 100% slash, all stake burned
    DoubleExecution,        // 100% slash, all stake burned
    InvalidStateRoot,       // 10% of self-bonded at 3x multiplier, delegators protected
    LivenessDeadZone,       // 2% per missed epoch after 5th consecutive
}

/// The complete economics state
#[derive(Debug, Clone)]
pub struct EconomicsState {
    pub validators: Vec<Validator>,
    pub pool_a_balance: u64,        // Core Security Budget
    pub pool_b_balance: u64,        // VM Execution Marketplace
    pub public_goods_pool: u64,     // 25% of MEV/priority tips
    pub total_supply: u64,
    pub current_epoch: u64,
    pub base_inflation_rate: f64,   // Starts at 1.5% annually
    pub mev_fee_floor: f64,         // 0.25 = 25%
    pub min_validator_stake: u64,   // 32,000 LGT
}

impl EconomicsState {
    pub fn new(total_supply: u64) -> Self {
        EconomicsState {
            validators: Vec::new(),
            pool_a_balance: 0,
            pool_b_balance: 0,
            public_goods_pool: 0,
            total_supply,
            current_epoch: 0,
            base_inflation_rate: 0.015, // 1.5% annual
            mev_fee_floor: 0.25,
            min_validator_stake: 32_000,
        }
    }

    /// Register a new validator
    pub fn register_validator(&mut self, address: &str, stake: u64) -> Result<String, String> {
        if stake < self.min_validator_stake {
            return Err(format!(
                "Minimum stake is {} LGT. Provided: {}",
                self.min_validator_stake, stake
            ));
        }

        let id = format!("validator-{}", self.validators.len() + 1);
        let validator = Validator {
            id: id.clone(),
            address: address.to_string(),
            self_bonded_stake: stake,
            delegated_stake: 0,
            uptime_score: 1.0,
            jailed: false,
            slash_history: Vec::new(),
        };

        self.validators.push(validator);
        println!("[ECON] Validator {} registered with {} LGT stake", id, stake);
        Ok(id)
    }

    /// Add delegation to a validator
    pub fn delegate(&mut self, validator_id: &str, amount: u64) -> Result<(), String> {
        let validator = self.validators
            .iter_mut()
            .find(|v| v.id == validator_id)
            .ok_or("Validator not found")?;

        validator.delegated_stake += amount;
        println!("[ECON] {} LGT delegated to {}", amount, validator_id);
        Ok(())
    }

    /// Calculate total stake (self-bonded + delegated)
    pub fn get_total_stake(&self, validator_id: &str) -> Result<u64, String> {
        let v = self.validators
            .iter()
            .find(|v| v.id == validator_id)
            .ok_or("Validator not found")?;
        Ok(v.self_bonded_stake + v.delegated_stake)
    }

    /// Fund Pool A (Core Security Budget) from inflation
    pub fn fund_pool_a(&mut self) {
        // Annual inflation converted to per-epoch
        // Assuming 32,850 epochs per year (365 days / 12.8 minutes per epoch)
        let epochs_per_year = 32_850.0;
        let epoch_inflation_rate = self.base_inflation_rate / epochs_per_year;
        let inflation_amount = (self.total_supply as f64 * epoch_inflation_rate) as u64;

        self.pool_a_balance += inflation_amount;
        self.total_supply += inflation_amount;

        println!("[ECON] Pool A funded: {} LGT (inflation)", inflation_amount);
    }

    /// Fund Pool B (VM Execution Marketplace) from fees
    pub fn fund_pool_b(&mut self, compute_fees: u64, storage_rent: u64, tips: u64) {
        let total = compute_fees + storage_rent + tips;

        // MEV Fee Floor: 25% redirected to Public Goods Pool
        let mev_portion = (tips as f64 * self.mev_fee_floor) as u64;
        let pool_b_portion = total - mev_portion;

        self.pool_b_balance += pool_b_portion;
        self.public_goods_pool += mev_portion;

        println!("[ECON] Pool B funded: {} LGT ({} to public goods)", pool_b_portion, mev_portion);
    }

    /// Distribute rewards to all validators at epoch end
    pub fn distribute_rewards(&mut self) {
        if self.validators.is_empty() {
            println!("[ECON] No validators to distribute rewards");
            return;
        }

        let total_stake: u64 = self.validators
            .iter()
            .filter(|v| !v.jailed)
            .map(|v| v.self_bonded_stake + v.delegated_stake)
            .sum();

        if total_stake == 0 {
            return;
        }

        for validator in &mut self.validators {
            if validator.jailed {
                continue;
            }

            let stake = validator.self_bonded_stake + validator.delegated_stake;
            let stake_share = stake as f64 / total_stake as f64;

            // Pool A: distributed pro-rata by stake weight
            let pool_a_share = (self.pool_a_balance as f64 * stake_share) as u64;

            // Pool B: distributed pro-rata by execution work (simplified to stake here)
            let pool_b_share = (self.pool_b_balance as f64 * stake_share) as u64;

            // Uptime multiplier applies to both pools
            let uptime = validator.uptime_score;
            let reward = ((pool_a_share + pool_b_share) as f64 * uptime) as u64;

            // Add reward to self-bonded stake (compounding)
            validator.self_bonded_stake += reward;

            println!("[ECON] {} earned {} LGT (uptime: {:.0}%)",
                validator.id, reward, uptime * 100.0);
        }

        // Clear pools after distribution
        self.pool_a_balance = 0;
        self.pool_b_balance = 0;
        self.current_epoch += 1;
    }

    /// Slash a validator for a specific offense
    pub fn slash(
        &mut self,
        validator_id: &str,
        reason: SlashReason,
    ) -> Result<u64, String> {
        let validator = self.validators
            .iter_mut()
            .find(|v| v.id == validator_id)
            .ok_or("Validator not found")?;

        if validator.jailed {
            return Err("Validator already jailed".to_string());
        }

        let (slash_percent, affects_delegated) = match reason {
            SlashReason::DoubleSequencing => (100, true),   // 100% slash, all stake
            SlashReason::DoubleExecution => (100, true),    // 100% slash, all stake
            SlashReason::InvalidStateRoot => (30, false),   // 10% at 3x = 30% of self-bonded, delegators safe
            SlashReason::LivenessDeadZone => (2, false),    // 2% of self-bonded
        };

        let self_bonded_slash = (validator.self_bonded_stake * slash_percent) / 100;
        let delegated_slash = if affects_delegated {
            (validator.delegated_stake * slash_percent) / 100
        } else {
            0
        };

        let total_slashed = self_bonded_slash + delegated_slash;

        // Apply slash
        validator.self_bonded_stake -= self_bonded_slash;
        validator.delegated_stake -= delegated_slash;

        // Burn slashed stake (reduces total supply)
        self.total_supply -= total_slashed;

        // Record slash event
        validator.slash_history.push(SlashEvent {
            reason: reason.clone(),
            amount_burned: total_slashed,
            epoch: self.current_epoch,
            affected_self_bonded: self_bonded_slash > 0,
            affected_delegated: delegated_slash > 0,
        });

        // Jail validator for malicious offenses
        if affects_delegated {
            validator.jailed = true;
            println!("[ECON] {} JAILED permanently", validator_id);
        }

        println!("[ECON] {} SLASHED: {} LGT burned ({:?})",
            validator_id, total_slashed, reason);

        Ok(total_slashed)
    }

    /// Set validator uptime score (called each epoch)
    pub fn set_uptime(&mut self, validator_id: &str, score: f64) -> Result<(), String> {
        if score < 0.0 || score > 1.0 {
            return Err("Uptime must be between 0.0 and 1.0".to_string());
        }

        let validator = self.validators
            .iter_mut()
            .find(|v| v.id == validator_id)
            .ok_or("Validator not found")?;

        validator.uptime_score = score;
        Ok(())
    }

    /// Print the economics state
    pub fn print_state(&self) {
        println!("=== VALIDATOR ECONOMICS ===");
        println!("Epoch: {}", self.current_epoch);
        println!("Total Supply: {} LGT", self.total_supply);
        println!("Pool A (Core Security): {} LGT", self.pool_a_balance);
        println!("Pool B (VM Execution):  {} LGT", self.pool_b_balance);
        println!("Public Goods Pool:     {} LGT", self.public_goods_pool);
        println!("Validators: {}", self.validators.len());
        for v in &self.validators {
            let total_stake = v.self_bonded_stake + v.delegated_stake;
            let status = if v.jailed { "JAILED" } else { "active" };
            println!("  {} ({}) - {} LGT total, uptime: {:.0}%, slashes: {}",
                v.id, status, total_stake, v.uptime_score * 100.0, v.slash_history.len());
        }
        println!("==============================");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_validator() {
        let mut econ = EconomicsState::new(1_000_000_000);
        let result = econ.register_validator("alice", 50_000);
        assert!(result.is_ok());
        assert_eq!(econ.validators.len(), 1);
    }

    #[test]
    fn test_register_below_minimum_stake() {
        let mut econ = EconomicsState::new(1_000_000_000);
        let result = econ.register_validator("bob", 100);
        assert!(result.is_err());
    }

    #[test]
    fn test_delegation() {
        let mut econ = EconomicsState::new(1_000_000_000);
        econ.register_validator("alice", 50_000).unwrap();
        econ.delegate("validator-1", 10_000).unwrap();
        let total = econ.get_total_stake("validator-1").unwrap();
        assert_eq!(total, 60_000);
    }

    #[test]
    fn test_fund_and_distribute() {
        let mut econ = EconomicsState::new(1_000_000_000);
        econ.register_validator("alice", 50_000).unwrap();
        econ.register_validator("bob", 50_000).unwrap();

        let pre_stake_alice = econ.validators[0].self_bonded_stake;
        econ.fund_pool_a();
        econ.fund_pool_b(100, 50, 20);
        econ.distribute_rewards();

        // Both validators should have earned rewards
        assert!(econ.validators[0].self_bonded_stake > pre_stake_alice);
        assert_eq!(econ.pool_a_balance, 0);
        assert_eq!(econ.pool_b_balance, 0);
    }

    #[test]
    fn test_uptime_multiplier() {
        let mut econ = EconomicsState::new(1_000_000_000);
        econ.register_validator("alice", 50_000).unwrap();
        econ.register_validator("bob", 50_000).unwrap();

        // Bob has 50% uptime
        econ.set_uptime("validator-2", 0.5).unwrap();

        econ.fund_pool_a();
        econ.distribute_rewards();

        // Alice (100% uptime) should earn more than Bob (50%)
        let alice_stake = econ.validators[0].self_bonded_stake;
        let bob_stake = econ.validators[1].self_bonded_stake;
        assert!(alice_stake > bob_stake);
    }

    #[test]
    fn test_double_sign_slash() {
        let mut econ = EconomicsState::new(1_000_000_000);
        econ.register_validator("alice", 50_000).unwrap();
        econ.delegate("validator-1", 10_000).unwrap();

        let result = econ.slash("validator-1", SlashReason::DoubleSequencing);
        assert!(result.is_ok());

        let v = &econ.validators[0];
        assert!(v.jailed);
        assert_eq!(v.self_bonded_stake, 0);
        assert_eq!(v.delegated_stake, 0);
    }

    #[test]
    fn test_invalid_state_root_slash_protects_delegators() {
        let mut econ = EconomicsState::new(1_000_000_000);
        econ.register_validator("alice", 50_000).unwrap();
        econ.delegate("validator-1", 10_000).unwrap();

        let result = econ.slash("validator-1", SlashReason::InvalidStateRoot);
        assert!(result.is_ok());

        let v = &econ.validators[0];
        assert!(!v.jailed); // Not jailed for operator fault
        assert!(v.self_bonded_stake < 50_000); // Self-bonded slashed
        assert_eq!(v.delegated_stake, 10_000); // Delegators protected
    }

    #[test]
    fn test_mev_redistribution() {
        let mut econ = EconomicsState::new(1_000_000_000);
        econ.fund_pool_b(0, 0, 100); // 100 LGT in tips

        // 25% should go to public goods pool
        assert_eq!(econ.public_goods_pool, 25);
        // 75% should go to pool B
        assert_eq!(econ.pool_b_balance, 75);
    }
}
