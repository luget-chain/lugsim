// LUGET Cryptography Module
// Ed25519 signatures, Blake3 hashing, key generation
// Kipngetich Clinton, Waigeri, Bomet County, Kenya

use ed25519_dalek::{SigningKey, VerifyingKey, Signature, Signer, Verifier};
use rand::rngs::OsRng;
use blake3::Hasher;

/// A keypair for a user or validator
#[derive(Debug, Clone)]
pub struct KeyPair {
    pub public_key: VerifyingKey,
    pub secret_key: SigningKey,
    pub address: String,  // Blake3 hash of public key, hex-encoded
}

impl KeyPair {
    /// Generate a new random keypair
    pub fn generate() -> Self {
        let secret_key = SigningKey::generate(&mut OsRng);
        let public_key = secret_key.verifying_key();
        let address = hash_bytes(&public_key.to_bytes());
        KeyPair { public_key, secret_key, address }
    }

    /// Sign a message (any bytes)
    pub fn sign(&self, message: &[u8]) -> Vec<u8> {
        self.secret_key.sign(message).to_vec()
    }

    /// Verify a signature
    pub fn verify(public_key: &VerifyingKey, message: &[u8], signature: &[u8]) -> bool {
        if let Ok(sig) = Signature::from_slice(signature) {
            public_key.verify(message, &sig).is_ok()
        } else {
            false
        }
    }

    /// Get the wallet address (Blake3 hash of public key)
    pub fn address(&self) -> String {
        self.address.clone()
    }
}

/// Hash arbitrary bytes with Blake3, return hex string
pub fn hash_bytes(data: &[u8]) -> String {
    let mut hasher = Hasher::new();
    hasher.update(data);
    hex::encode(hasher.finalize().as_bytes())
}

/// Hash a string with Blake3, return hex string
pub fn hash_str(s: &str) -> String {
    hash_bytes(s.as_bytes())
}

/// Create a UTXO ID from transaction hash and output index
pub fn utxo_id(tx_hash: &str, output_index: usize) -> String {
    let input = format!("{}:{}", tx_hash, output_index);
    hash_str(&input)
}

/// Create a CLock ID
pub fn clock_id(tx_hash: &str) -> String {
    format!("clock-{}", &tx_hash[..12.min(tx_hash.len())])
}

/// Create an Object ID
pub fn object_id(creator_tx: &str, index: u64) -> String {
    let input = format!("{}:{}", creator_tx, index);
    hash_str(&input)
}

/// Create an epoch state root from withdrawal data
pub fn epoch_state_root(epoch: u64, withdrawals: &[String]) -> String {
    let mut hasher = Hasher::new();
    hasher.update(&epoch.to_le_bytes());
    for w in withdrawals {
        hasher.update(w.as_bytes());
    }
    hex::encode(hasher.finalize().as_bytes())
}

/// Sign transaction data for verification
pub fn sign_transaction(keypair: &KeyPair, tx_data: &str) -> Vec<u8> {
    keypair.sign(tx_data.as_bytes())
}

/// Verify a transaction signature
pub fn verify_transaction(public_key: &VerifyingKey, tx_data: &str, signature: &[u8]) -> bool {
    KeyPair::verify(public_key, tx_data.as_bytes(), signature)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_generation() {
        let kp = KeyPair::generate();
        assert!(!kp.address().is_empty());
        assert_eq!(kp.address().len(), 64); // Blake3 hex is 64 chars
    }

    #[test]
    fn test_sign_and_verify() {
        let kp = KeyPair::generate();
        let message = b"Alice sends 100 LGT to Bob";
        let signature = kp.sign(message);
        assert!(KeyPair::verify(&kp.public_key, message, &signature));
    }

    #[test]
    fn test_signature_rejects_tampered_message() {
        let kp = KeyPair::generate();
        let message = b"Alice sends 100 LGT to Bob";
        let signature = kp.sign(message);
        let tampered = b"Alice sends 1000 LGT to Bob";
        assert!(!KeyPair::verify(&kp.public_key, tampered, &signature));
    }

    #[test]
    fn test_signature_rejects_wrong_key() {
        let alice = KeyPair::generate();
        let bob = KeyPair::generate();
        let message = b"Alice sends 100 LGT to Bob";
        let signature = alice.sign(message);
        assert!(!KeyPair::verify(&bob.public_key, message, &signature));
    }

    #[test]
    fn test_utxo_id_deterministic() {
        let tx_hash = hash_str("genesis");
        let id1 = utxo_id(&tx_hash, 0);
        let id2 = utxo_id(&tx_hash, 0);
        assert_eq!(id1, id2);
        assert_eq!(id1.len(), 64);
    }

    #[test]
    fn test_epoch_state_root_deterministic() {
        let withdrawals = vec!["clock-1".to_string(), "clock-2".to_string()];
        let root1 = epoch_state_root(1, &withdrawals);
        let root2 = epoch_state_root(1, &withdrawals);
        assert_eq!(root1, root2);
    }

    #[test]
    fn test_epoch_state_root_changes_with_epoch() {
        let withdrawals = vec!["clock-1".to_string()];
        let root1 = epoch_state_root(1, &withdrawals);
        let root2 = epoch_state_root(2, &withdrawals);
        assert_ne!(root1, root2);
    }
}
