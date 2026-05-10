# LUGET Node API Specification
## REST API for the LUGET Node Client
**Version:** 1.0.0 | **Author:** Kipngetich Clinton | **Date:** 2026-05-10

## 1. Overview
The LUGET node client exposes a REST API over HTTP on port 9734. All responses are JSON. Authentication is not required for read endpoints. Transaction submission requires an Ed25519 signature.

## 2. Base URL

## 3. Endpoints

### 3.1 Node Status
**Response:**
```json
{
  "node_id": "0x...",
  "version": "1.0.0",
  "network": "testnet",
  "block_height": 12345,
  "epoch": 192,
  "peers": 47,
  "uptime_seconds": 86400
}
GET /v1/block/{height}
GET /v1/block/{hash}
{
  "height": 12345,
  "hash": "0x...",
  "parent_hash": "0x...",
  "proposer": "0x...",
  "timestamp": 1715200000,
  "transactions": ["0x...", "0x..."],
  "state_root_core": "0x...",
  "state_root_vm": "0x..."
}
GET /v1/tx/{hash}
{
  "hash": "0x...",
  "from": "0x...",
  "to": "0x...",
  "amount": 100,
  "fee": 5,
  "nonce": 42,
  "block_height": 12345,
  "signature": "0x...",
  "status": "confirmed"
}
POST /v1/tx/submit
{
  "from": "0x...",
  "to": "0x...",
  "amount": 100,
  "fee": 5,
  "nonce": 42,
  "signature": "0x..."
}
{
  "tx_hash": "0x...",
  "status": "pending"
}
{
  "error": "Invalid signature"
}
GET /v1/balance/{address}
{
  "address": "0x...",
  "balance": 5000,
  "nonce": 42
}
GET /v1/validators
{
  "epoch": 192,
  "validators": [
    {"id": "validator-1", "stake": 100000, "uptime": 99.8, "status": "active"},
    {"id": "validator-2", "stake": 85000, "uptime": 95.2, "status": "active"}
  ],
  "total_stake": 185000
}
GET /v1/governance/proposals
GET /v1/governance/proposal/{id}
{
  "id": "prop-1",
  "title": "Adjust CLock fee",
  "type": "CoreParameter",
  "status": "voting",
  "votes_for": 2500000,
  "votes_against": 500000,
  "epoch_submitted": 180,
  "timelock_blocks": 100800
}
GET /v1/bridge/status
{
  "locked_ltg": 1500000,
  "minted_coins": 1500000,
  "pending_withdrawals": 12,
  "finalized_epochs": 191,
  "bls_committee_size": 100,
  "managed_pause_active": false
}
GET /v1/network/peers
{
  "peers": [
    {"node_id": "0x...", "address": "13.53.126.22:9733", "score": 12, "connected_duration": 3600},
    {"node_id": "0x...", "address": "18.196.101.23:9733", "score": 8, "connected_duration": 7200}
  ]
}
GET /v1/mempool
{
  "pending_count": 247,
  "transactions": ["0x...", "0x..."]
}
4. Error Codes

Code Meaning
200 Success
400 Invalid request
404 Block/tx/address not found
429 Rate limited
500 Internal error

5. Rate Limiting

· 100 requests per second per IP
· 1000 requests per minute per IP
· Exceeded limits return 429 with Retry-After header

6. Implementation Notes

· Built with axum (Rust web framework)
· JSON serialization via serde_json
· All hashes and addresses are 64-character hex strings (Blake3)
· Signatures are 128-character hex strings (Ed25519)
· Port 9734 does not conflict with P2P port 9733

Reference implementation: github.com/luget-chain/lugsim
