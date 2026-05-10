# LUGET Network Protocol Specification
## P2P Layer for the LUGET Blockchain
**Version:** 1.0.0 | **Author:** Kipngetich Clinton | **Date:** 2026-05-10

## 1. Overview
The LUGET network layer uses TCP with Noise IK encryption (Curve25519 + ChaChaPoly + Blake2b). Messages are length-prefixed with CBOR encoding. Validators authenticate using their on-chain Ed25519 keys.

## 2. Transport
| Parameter | Value |
| :--- | :--- |
| Protocol | TCP (IPv4/IPv6) |
| Default port | 9733 |
| Testnet port | 19733 |
| Connection timeout | 10s |
| Idle timeout | 300s |
| Max inbound | 128 |
| Max outbound | 16 |
| Min outbound | 8 |

## 3. Noise IK Handshake
Initiator knows responder's static key from the on-chain validator registry. Both parties generate ephemeral keys. After handshake, all messages are ChaChaPoly-encrypted.

## 4. Message Framing
4-byte big-endian length prefix + CBOR-encoded payload. Max payload: 1MB standard, 4MB blocks, 8MB epoch proofs.

## 5. Message Types
| ID | Name | Purpose |
| :--- | :--- | :--- |
| 0x01 | HELLO | Handshake, capabilities |
| 0x02 | PING | Keep-alive |
| 0x03 | PONG | Latency response |
| 0x10 | GET_PEERS | Request peer list |
| 0x11 | PEERS | Peer addresses |
| 0x20 | TRANSACTION | Gossip pending tx |
| 0x21 | BLOCK_PROPOSAL | New block |
| 0x22 | BLOCK_VOTE | Validator vote |
| 0x23 | BLOCK_COMMIT | Finalized block |
| 0x30 | EPOCH_STATE_ROOT | VM epoch root |
| 0x31 | BLCC_ATTESTATION | Bridge committee vote |

## 6. Peer Discovery
Five hardcoded bootnodes (Africa, Europe, Asia, N.America, S.America). Nodes disconnect from bootnodes after finding 8+ peers. Peer exchange via PEERS messages and on-chain validator registry.

## 7. Peer Scoring
Score range -100 to +100. Invalid signature = -50 (24h ban). Valid block = +5. Valid vote = +3.

## 8. Gossip
Validators forward to all peers. Full nodes forward to 8 random peers. Light clients do not forward.

## 9. NAT Traversal
Validators behind NAT designate a relay with public IP. UPnP attempted on startup.

## 10. Eclipse Attack Resistance
Minimum 8 outbound peers from different /16 IPv4 prefixes. Bootnodes rotate each epoch. Connection rotation every 10 minutes.

## 11. Connection State Machine
Disconnected → Dialing/Listening → NoiseHandshake → Established → Active → Disconnecting/Banned → Disconnected.

## 12. Security
Noise encryption for all traffic. Ed25519 signatures on all messages. Sequence numbers prevent replay. Rate limiting and peer scoring prevent DDoS.

*Reference implementation: github.com/luget-chain/lugsim*
