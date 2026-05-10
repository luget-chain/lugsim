# LUGET Consensus Specification
## BFT Consensus Protocol for Dual-Domain Finality
**Version:** 1.0.0 | **Author:** Kipngetich Clinton | **Date:** 2026-05-10

## 1. Overview
LUGET uses a Tendermint-derived BFT consensus protocol with dual finality. Validators propose blocks in rounds. Each round has proposer, prevote, and precommit phases. Blocks achieve provable finality after 2/3+ precommits. Epochs (64 blocks) finalize VM state roots for bridge withdrawals.

## 2. Validator Set
| Parameter | Value |
| :--- | :--- |
| Minimum validators | 4 |
| Maximum validators | 256 |
| Minimum stake | 32,000 LGT |
| Validator activation | After 1 epoch |
| Validator exit | 2 epoch cooldown |
| Slashing | See economics module |

## 3. Consensus Rounds
### 3.1 Round Timing
| Parameter | Value |
| :--- | :--- |
| Block time (target) | 12 seconds |
| Proposer timeout | 3 seconds |
| Prevote timeout | 6 seconds |
| Precommit timeout | 9 seconds |
| Round timeout | 12 seconds |

### 3.2 Proposer Selection
Proposer = validator_set[ (block_height + round) % len(validator_set) ]
Weighted by stake. The proposer rotates deterministically.

### 3.3 Round States
If timeout at any phase, increment round and restart with new proposer.

## 4. Phase Specifications
### 4.1 Proposer Phase (0-3s)
Proposer creates block with:
- Parent hash of last committed block
- Valid transactions from mempool (max 4000)
- Core UTXO merkle root
- VM state merkle root
- Timestamp > median of last 11 blocks
- Ed25519 signature

Broadcasts BLOCK_PROPOSAL. If no proposal received by T+3s, validators move to prevote with nil.

### 4.2 Prevote Phase (3-6s)
Validators validate proposed block:
- Proposer is correct for this height+round
- Parent hash matches last known commit
- Timestamp is valid
- All transactions pass validation
- Merkle roots are correct

If valid, broadcast BLOCK_VOTE (Prevote, approve=true).
If invalid or no proposal, broadcast BLOCK_VOTE (Prevote, approve=false).

### 4.3 Precommit Phase (6-9s)
If 2/3+ prevotes for a block received, broadcast BLOCK_VOTE (Precommit, approve=true).
If 2/3+ prevotes for nil received, broadcast BLOCK_VOTE (Precommit, approve=false).
Otherwise, broadcast BLOCK_VOTE (Precommit, approve=false).

### 4.4 Commit Phase (9-12s)
If 2/3+ precommits for same block received:
- Block is FINALIZED
- Execute transactions, update state
- Broadcast BLOCK_COMMIT
- Increment block height, reset round to 0
If round expires without commit, increment round, restart with new proposer.

## 5. Dual Finality
### 5.1 Block Finality (Probabilistic)
Each block commits after 2/3+ precommits (~9-12 seconds).
### 5.2 Epoch Finality (Provable)
Every 64 blocks, validators produce VM state root.
State root committed to Core chain.
Required for bridge withdrawals (mandatory 1-epoch delay).

## 6. Fork Choice Rule
Honest validators follow: heaviest chain by stake-weighted attestations.
Never reorganize finalized blocks.

## 7. Byzantine Fault Tolerance
Tolerates up to f = (n-1)/3 Byzantine validators.
With 100 validators: tolerates 33 malicious.
Safety guaranteed if < 1/3 are Byzantine.
Liveness guaranteed if ≥ 2/3 are honest and network is synchronous.

## 8. Evidence Handling
### 8.1 Duplicate Vote Evidence
If validator signs two different blocks at same height+round, submit evidence.
Validator slashed 100%, stake burned, permanently jailed.
### 8.2 Invalid Block Evidence
If validator signs precommit for invalid block, submit evidence.
Validator slashed 100%.

## 9. State Machine
### Transitions:

## 10. Performance
| Metric | Target |
| :--- | :--- |
| Block time | 12 seconds |
| Transactions per block | 4,000 |
| Epoch duration | 12.8 minutes |
| Finality | 1 block (~12s) |
| Throughput | ~333 tx/s (single chain) |

## 11. Test Vectors
See `src/consensus.rs` for implementation.
Key tests:
- `test_finalize_block`: 5 validators, 100% attestation
- `test_timeout_triggers_new_round`: proposer offline
- `test_nil_prevote_on_invalid_block`: invalid proposal

## 12. References
- LUGET Yellowpaper, Section 5: Consensus Mechanism
- Tendermint: https://arxiv.org/abs/1807.04938
- `lugsim` source: `src/consensus.rs`

*Ready for implementation. See github.com/luget-chain/lugsim*
