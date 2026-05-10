# Contributing to LUGET
**The battlefield is open. You are needed.**
LUGET is a dual-domain Layer-1 blockchain: sound money on Core, safe applications on the VM, connected by a firewalled bridge. No premine. No VCs. No insider tokens. Phase 0 complete. 66 tests passing.

## Quick Start
git clone https://github.com/luget-chain/lugsim.git
cd lugsim
cargo build --all
cargo test --all

## Wallet
cargo run --bin lugwallet -- generate
cargo run --bin lugwallet -- address
cargo run --bin lugwallet -- send ADDRESS 100 5

## Project Structure
src/core.rs — UTXO state machine
src/vm.rs — Object model with type abilities
src/bridge.rs — Firewalled bridge
src/economics.rs — Validator rewards + slashing
src/governance.rs — 4-institution governance
src/crypto.rs — Ed25519 + Blake3
src/bls_committee.rs — BLS threshold signatures
src/network.rs — P2P multi-node simulation
src/consensus.rs — BFT consensus engine
specs/NETWORK_SPEC.md — P2P protocol spec
specs/CONSENSUS_SPEC.md — BFT consensus spec
VERIFICATION.md — Formal proofs of bridge invariants

## Good First Issues
Add test vectors for UTXO validation (core.rs)
Implement Owner::Object in simulation (vm.rs)
Add parallel execution benchmark (benchmarks.rs)

## Major Initiatives (Seeking Engineers)
Production Rust node client (Tokio, P2P, consensus)
Real BFT consensus implementation
VM bytecode compiler (Move language)

## How to Get Paid
Zero VC funding. Builder's Fund: 25M LGT over 10 years. Contributors earn grant eligibility.

## Communication
Discord: https://discord.gg/M2f34jdP5M
GitHub Issues for bugs and features

## Code of Conduct
Verify, don't trust. No insider advantage. Build in the open. Zero tolerance for scams.

*The battlefield is open. Let's build.*
