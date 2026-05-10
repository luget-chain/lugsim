// LUGET Governance Module
// Quadripartite governance: Core Council, VM Council, Monetary Commission, Holder Veto
// Kipngetich Clinton, Waigeri, Bomet County, Kenya

#[derive(Debug, Clone)]
pub struct Proposal {
    pub id: String,
    pub title: String,
    pub description: String,
    pub proposal_type: ProposalType,
    pub status: ProposalStatus,
    pub submitted_epoch: u64,
    pub votes_for: u64,
    pub votes_against: u64,
    pub timelock_blocks: u64,
    pub required_participation: f64,  // 0.0 to 1.0
    pub required_approval: f64,       // 0.0 to 1.0
}

#[derive(Debug, Clone, PartialEq)]
pub enum ProposalType {
    CoreParameter,         // Core Council: block size, CLock fees
    VMUpgrade,             // VM Council: bytecodes, gas schedule
    MonetaryEmergency,     // Monetary Commission: freeze inflation
    ConstitutionalAmendment, // Holder veto: fundamental rules
}

#[derive(Debug, Clone, PartialEq)]
pub enum ProposalStatus {
    Proposed,
    ReviewPeriod,
    Voting,
    Passed,
    Rejected,
    Vetoed,
    Expired,
}

/// The Core Council (7 members, 5-of-7 multi-sig)
#[derive(Debug, Clone)]
pub struct CoreCouncil {
    pub members: Vec<String>,  // Member addresses
    pub threshold: usize,       // 5 of 7
}

/// The VM Council (15-25 technical members)
#[derive(Debug, Clone)]
pub struct VmCouncil {
    pub members: Vec<String>,
    pub threshold: usize,       // 66% of members
}

/// The Monetary Commission (3 oversight members)
#[derive(Debug, Clone)]
pub struct MonetaryCommission {
    pub members: Vec<String>,
    pub emergency_declared: bool,
    pub emergency_epoch: u64,
}

/// The complete governance state
#[derive(Debug, Clone)]
pub struct GovernanceState {
    pub core_council: CoreCouncil,
    pub vm_council: VmCouncil,
    pub monetary_commission: MonetaryCommission,
    pub proposals: Vec<Proposal>,
    pub total_active_stake: u64,     // LGT moved/staked/voted in last 4 years
    pub total_dormant_stake: u64,    // LGT unmoved for >4 years
    pub current_epoch: u64,
    pub next_proposal_id: u64,
}

impl GovernanceState {
    pub fn new() -> Self {
        GovernanceState {
            core_council: CoreCouncil {
                members: vec![
                    "councilor-1".to_string(), "councilor-2".to_string(),
                    "councilor-3".to_string(), "councilor-4".to_string(),
                    "councilor-5".to_string(), "councilor-6".to_string(),
                    "councilor-7".to_string(),
                ],
                threshold: 5,
            },
            vm_council: VmCouncil {
                members: (1..=20).map(|i| format!("vm-councilor-{}", i)).collect(),
                threshold: 14, // 66% of 20
            },
            monetary_commission: MonetaryCommission {
                members: vec!["commissioner-1".to_string(), "commissioner-2".to_string(), "commissioner-3".to_string()],
                emergency_declared: false,
                emergency_epoch: 0,
            },
            proposals: Vec::new(),
            total_active_stake: 0,
            total_dormant_stake: 0,
            current_epoch: 0,
            next_proposal_id: 1,
        }
    }

    /// Submit a new proposal
    pub fn submit_proposal(
        &mut self,
        title: &str,
        description: &str,
        proposal_type: ProposalType,
    ) -> String {
        let id = format!("prop-{}", self.next_proposal_id);
        self.next_proposal_id += 1;

        let (timelock, participation, approval) = match proposal_type {
            ProposalType::CoreParameter => (14 * 24 * 300, 0.20, 0.50),  // 14 days, 20% participation, 50% approval
            ProposalType::VMUpgrade => (30 * 24 * 300, 0.25, 0.50),       // 30 days
            ProposalType::MonetaryEmergency => (14 * 24 * 300, 0.66, 0.66), // 14 days, 66% both
            ProposalType::ConstitutionalAmendment => (180 * 24 * 300, 0.75, 0.80), // 6 months, 75% part, 80% approval
        };

        let proposal = Proposal {
            id: id.clone(),
            title: title.to_string(),
            description: description.to_string(),
            proposal_type,
            status: ProposalStatus::ReviewPeriod,
            submitted_epoch: self.current_epoch,
            votes_for: 0,
            votes_against: 0,
            timelock_blocks: timelock,
            required_participation: participation,
            required_approval: approval,
        };

        println!("[GOV] Proposal {}: {} ({:?})", id, title, proposal.proposal_type);
        self.proposals.push(proposal);
        id
    }

    /// Cast votes on a proposal
    pub fn vote(&mut self, proposal_id: &str, stake: u64, approve: bool) -> Result<(), String> {
        let proposal = self.proposals
            .iter_mut()
            .find(|p| p.id == proposal_id)
            .ok_or("Proposal not found")?;

        if proposal.status != ProposalStatus::Voting {
            return Err("Proposal is not in voting phase".to_string());
        }

        if approve {
            proposal.votes_for += stake;
        } else {
            proposal.votes_against += stake;
        }

        Ok(())
    }

    /// Tally votes and determine outcome
    pub fn tally(&mut self, proposal_id: &str) -> Result<ProposalStatus, String> {
        let proposal = self.proposals
            .iter_mut()
            .find(|p| p.id == proposal_id)
            .ok_or("Proposal not found")?;

        if proposal.status != ProposalStatus::Voting {
            return Err("Proposal is not in voting phase".to_string());
        }

        let total_votes = proposal.votes_for + proposal.votes_against;
        let total_stake = self.total_active_stake;

        // Check participation threshold
        let participation_rate = if total_stake > 0 {
            total_votes as f64 / total_stake as f64
        } else {
            0.0
        };

        if participation_rate < proposal.required_participation {
            proposal.status = ProposalStatus::Rejected;
            println!("[GOV] {} REJECTED: participation {:.1}% < required {:.1}%",
                proposal_id, participation_rate * 100.0, proposal.required_participation * 100.0);
            return Ok(ProposalStatus::Rejected);
        }

        // Check approval threshold
        let approval_rate = if total_votes > 0 {
            proposal.votes_for as f64 / total_votes as f64
        } else {
            0.0
        };

        if approval_rate >= proposal.required_approval {
            proposal.status = ProposalStatus::Passed;
            println!("[GOV] {} PASSED: approval {:.1}% > required {:.1}%",
                proposal_id, approval_rate * 100.0, proposal.required_approval * 100.0);
            Ok(ProposalStatus::Passed)
        } else {
            proposal.status = ProposalStatus::Rejected;
            println!("[GOV] {} REJECTED: approval {:.1}% < required {:.1}%",
                proposal_id, approval_rate * 100.0, proposal.required_approval * 100.0);
            Ok(ProposalStatus::Rejected)
        }
    }

    /// Holder veto: 33% of stake can block any proposal
    pub fn holder_veto(&mut self, proposal_id: &str, veto_stake: u64) -> Result<bool, String> {
        let proposal = self.proposals
            .iter_mut()
            .find(|p| p.id == proposal_id)
            .ok_or("Proposal not found")?;

        if self.total_active_stake == 0 {
            return Err("No active stake to measure against".to_string());
        }

        let veto_threshold = 0.33; // 33% of active stake
        let veto_ratio = veto_stake as f64 / self.total_active_stake as f64;

        if veto_ratio >= veto_threshold {
            proposal.status = ProposalStatus::Vetoed;
            println!("[GOV] {} VETOED: {:.1}% of active stake voted NO",
                proposal_id, veto_ratio * 100.0);
            Ok(true)
        } else {
            println!("[GOV] Veto attempt failed: {:.1}% < 33% required",
                veto_ratio * 100.0);
            Ok(false)
        }
    }

    /// Monetary Commission declares emergency (requires unanimous 3/3)
    pub fn declare_monetary_emergency(&mut self, votes: [bool; 3]) -> Result<bool, String> {
        let all_yes = votes.iter().all(|&v| v);

        if !all_yes {
            println!("[GOV] Monetary emergency DECLINED: not unanimous");
            return Ok(false);
        }

        self.monetary_commission.emergency_declared = true;
        self.monetary_commission.emergency_epoch = self.current_epoch;
        println!("[GOV] MONETARY EMERGENCY DECLARED (unanimous)");
        println!("[GOV] Inflation frozen for max 14 days. Holder ratification required.");
        Ok(true)
    }

    /// Rotate Core Council (simulated election)
    pub fn elect_core_council(&mut self, new_members: Vec<String>) -> Result<(), String> {
        if new_members.len() != 7 {
            return Err("Core Council requires exactly 7 members".to_string());
        }
        self.core_council.members = new_members;
        println!("[GOV] Core Council elected: {} members", self.core_council.members.len());
        Ok(())
    }

    /// Apply dormancy fee to inactive stake
    pub fn apply_dormancy_fee(&mut self) -> u64 {
        let fee = (self.total_dormant_stake as f64 * 0.02) as u64 / 32_850; // 2% annual, per epoch
        self.total_dormant_stake = self.total_dormant_stake.saturating_sub(fee);
        println!("[GOV] Dormancy fee applied: {} LGT burned", fee);
        fee
    }

    /// Advance epoch
    pub fn advance_epoch(&mut self) {
        self.current_epoch += 1;

        // Move proposals from review to voting after timelock
        for proposal in &mut self.proposals {
            if proposal.status == ProposalStatus::ReviewPeriod {
                let elapsed = self.current_epoch - proposal.submitted_epoch;
                if elapsed * 13 * 300 >= proposal.timelock_blocks {
                    proposal.status = ProposalStatus::Voting;
                    println!("[GOV] {} moved to voting phase", proposal.id);
                }
            }
        }

        // Expire emergency if over 14 days
        if self.monetary_commission.emergency_declared {
            let emergency_duration = self.current_epoch - self.monetary_commission.emergency_epoch;
            if emergency_duration > 14 * 24 * 300 {
                self.monetary_commission.emergency_declared = false;
                println!("[GOV] Monetary emergency EXPIRED");
            }
        }
    }

    pub fn print_state(&self) {
        println!("=== GOVERNANCE STATE ===");
        println!("Epoch: {}", self.current_epoch);
        println!("Core Council: {} members (threshold: {})",
            self.core_council.members.len(), self.core_council.threshold);
        println!("VM Council: {} members (threshold: {})",
            self.vm_council.members.len(), self.vm_council.threshold);
        println!("Monetary Commission: {} members, emergency: {}",
            self.monetary_commission.members.len(), self.monetary_commission.emergency_declared);
        println!("Active Stake: {} LGT", self.total_active_stake);
        println!("Dormant Stake: {} LGT", self.total_dormant_stake);
        println!("Proposals: {}", self.proposals.len());
        for p in &self.proposals {
            println!("  {} ({:?}): {:?} — For: {}, Against: {}",
                p.id, p.proposal_type, p.status, p.votes_for, p.votes_against);
        }
        println!("==============================");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_submit_proposal() {
        let mut gov = GovernanceState::new();
        let id = gov.submit_proposal("Test", "A test proposal", ProposalType::CoreParameter);
        assert_eq!(id, "prop-1");
        assert_eq!(gov.proposals.len(), 1);
    }

    #[test]
    fn test_vote_and_tally_passes() {
        let mut gov = GovernanceState::new();
        gov.total_active_stake = 1000;

        let id = gov.submit_proposal("Fee change", "Adjust CLock fee", ProposalType::CoreParameter);

        // Move to voting
        gov.proposals[0].status = ProposalStatus::Voting;

        // Vote: 250 for, 50 against out of 1000 total active
        gov.vote(&id, 250, true).unwrap();
        gov.vote(&id, 50, false).unwrap();

        let result = gov.tally(&id).unwrap();
        assert_eq!(result, ProposalStatus::Passed);
    }

    #[test]
    fn test_low_participation_fails() {
        let mut gov = GovernanceState::new();
        gov.total_active_stake = 1000;

        let id = gov.submit_proposal("Upgrade", "VM upgrade", ProposalType::VMUpgrade);

        gov.proposals[0].status = ProposalStatus::Voting;
        gov.vote(&id, 100, true).unwrap(); // Only 10% participation

        let result = gov.tally(&id).unwrap();
        assert_eq!(result, ProposalStatus::Rejected);
    }

    #[test]
    fn test_holder_veto() {
        let mut gov = GovernanceState::new();
        gov.total_active_stake = 1000;

        let id = gov.submit_proposal("Change", "Something controversial", ProposalType::VMUpgrade);

        // 33% veto threshold: 330 LGT
        let result = gov.holder_veto(&id, 350);
        assert!(result.is_ok());
        assert!(result.unwrap()); // Veto succeeds

        let proposal = gov.proposals.iter().find(|p| p.id == id).unwrap();
        assert_eq!(proposal.status, ProposalStatus::Vetoed);
    }

    #[test]
    fn test_monetary_emergency_requires_unanimous() {
        let mut gov = GovernanceState::new();

        // 2 of 3 = fails
        let result = gov.declare_monetary_emergency([true, true, false]);
        assert!(result.is_ok());
        assert!(!result.unwrap());
        assert!(!gov.monetary_commission.emergency_declared);

        // 3 of 3 = succeeds
        let result = gov.declare_monetary_emergency([true, true, true]);
        assert!(result.is_ok());
        assert!(result.unwrap());
        assert!(gov.monetary_commission.emergency_declared);
    }

    #[test]
    fn test_constitutional_amendment_needs_supermajority() {
        let mut gov = GovernanceState::new();
        gov.total_active_stake = 1000;

        let id = gov.submit_proposal(
            "Change supply cap",
            "Amend the asymptotic hard cap",
            ProposalType::ConstitutionalAmendment,
        );

        // Constitutional: 75% participation, 80% approval
        gov.proposals[0].status = ProposalStatus::Voting;

        // 800 stake votes, 700 for (87.5% approval, 80% participation)
        gov.vote(&id, 700, true).unwrap();
        gov.vote(&id, 100, false).unwrap();

        let result = gov.tally(&id).unwrap();
        assert_eq!(result, ProposalStatus::Passed);
    }

    #[test]
    fn test_constitutional_amendment_fails_without_approval() {
        let mut gov = GovernanceState::new();
        gov.total_active_stake = 1000;

        let id = gov.submit_proposal(
            "Radical change",
            "Something controversial",
            ProposalType::ConstitutionalAmendment,
        );

        gov.proposals[0].status = ProposalStatus::Voting;

        // 800 participation but only 60% approval
        gov.vote(&id, 480, true).unwrap();
        gov.vote(&id, 320, false).unwrap();

        let result = gov.tally(&id).unwrap();
        assert_eq!(result, ProposalStatus::Rejected);
    }

    #[test]
    fn test_dormancy_fee() {
        let mut gov = GovernanceState::new();
        gov.total_dormant_stake = 100_000_000;

        let fee = gov.apply_dormancy_fee();
        assert!(fee > 0);
        assert!(gov.total_dormant_stake < 100_000_000);
    }
}
