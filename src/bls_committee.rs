// LUGET BLS Committee Module
// Threshold signatures for the Bridge Light Client Committee (BLCC)
// Kipngetich Clinton, Waigeri, Bomet County, Kenya

use blst::min_pk::{SecretKey, PublicKey, Signature, AggregateSignature};
use rand::RngCore;

#[derive(Clone)]
pub struct BlsMember {
    pub id: usize,
    pub secret_key: SecretKey,
    pub public_key: PublicKey,
    pub address: String,
}

impl BlsMember {
    pub fn generate(id: usize) -> Self {
        let mut ikm = [0u8; 32];
        rand::rngs::OsRng.fill_bytes(&mut ikm);
        let secret_key = SecretKey::key_gen(&ikm, &[]).unwrap();
        let public_key = secret_key.sk_to_pk();
        let address = crate::crypto::hash_bytes(&public_key.to_bytes());
        BlsMember { id, secret_key, public_key, address }
    }

    pub fn sign(&self, message: &[u8]) -> Signature {
        self.secret_key.sign(message, &[], &[])
    }

    pub fn verify(public_key: &PublicKey, message: &[u8], signature: &Signature) -> bool {
        signature.verify(true, message, &[], &[], public_key, false) == blst::BLST_ERROR::BLST_SUCCESS
    }
}

pub struct BlsCommittee {
    pub members: Vec<BlsMember>,
    pub threshold: usize,
    pub current_epoch: u64,
}

impl BlsCommittee {
    pub fn new() -> Self {
        let members: Vec<BlsMember> = (0..100).map(|i| BlsMember::generate(i)).collect();
        BlsCommittee { members, threshold: 80, current_epoch: 0 }
    }

    pub fn sign_block(&self, block_header: &[u8], signer_indices: &[usize]) -> Vec<(usize, Signature)> {
        signer_indices.iter()
            .filter_map(|&i| {
                if i < self.members.len() {
                    Some((i, self.members[i].sign(block_header)))
                } else { None }
            })
            .collect()
    }

    pub fn aggregate_signatures(signatures: &[Signature]) -> Result<Signature, String> {
        if signatures.is_empty() {
            return Err("No signatures to aggregate".to_string());
        }
        let sig_refs: Vec<&Signature> = signatures.iter().collect();
        let agg = AggregateSignature::aggregate(&sig_refs, false)
            .map_err(|e| format!("Aggregation failed: {:?}", e))?;
        Ok(agg.to_signature())
    }

    pub fn verify_threshold_signature(
        public_keys: &[&PublicKey],
        message: &[u8],
        aggregate_signature: &Signature,
        threshold: usize,
    ) -> bool {
        if public_keys.len() < threshold {
            return false;
        }
        let result = aggregate_signature.fast_aggregate_verify(true, message, &[], public_keys);
        match result {
            blst::BLST_ERROR::BLST_SUCCESS => true,
            _ => false,
        }
    }

    pub fn attest_block(&self, block_header_hex: &str) -> (bool, String) {
        let message = block_header_hex.as_bytes();
        let signer_count = 85;
        let signer_indices: Vec<usize> = (0..signer_count).collect();
        let signatures: Vec<Signature> = self.sign_block(message, &signer_indices)
            .into_iter().map(|(_, sig)| sig).collect();
        println!("[BLCC] {} of {} members signed", signatures.len(), self.members.len());

        match BlsCommittee::aggregate_signatures(&signatures) {
            Ok(aggregate) => {
                let pubkeys: Vec<&PublicKey> = signer_indices.iter()
                    .map(|&i| &self.members[i].public_key).collect();
                let valid = BlsCommittee::verify_threshold_signature(&pubkeys, message, &aggregate, self.threshold);
                let sig_hex = hex::encode(aggregate.to_bytes());
                (valid, sig_hex)
            }
            Err(e) => {
                println!("[BLCC] Error: {}", e);
                (false, String::new())
            }
        }
    }

    pub fn print_status(&self) {
        println!("=== BLCC STATUS ===");
        println!("Members: {}", self.members.len());
        println!("Threshold: {} of {}", self.threshold, self.members.len());
        println!("===================");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_member_generation() {
        let member = BlsMember::generate(1);
        assert!(!member.address.is_empty());
    }

    #[test]
    fn test_sign_and_verify() {
        let member = BlsMember::generate(1);
        let msg = b"test message";
        let sig = member.sign(msg);
        assert!(BlsMember::verify(&member.public_key, msg, &sig));
    }

    #[test]
    fn test_tampered_rejected() {
        let member = BlsMember::generate(1);
        let sig = member.sign(b"original");
        assert!(!BlsMember::verify(&member.public_key, b"tampered", &sig));
    }

    #[test]
    fn test_threshold_verification() {
        let committee = BlsCommittee::new();
        let msg = b"block-header";
        let indices: Vec<usize> = (0..85).collect();
        let sigs: Vec<Signature> = committee.sign_block(msg, &indices).into_iter().map(|(_, s)| s).collect();
        let agg = BlsCommittee::aggregate_signatures(&sigs).unwrap();
        let pks: Vec<&PublicKey> = indices.iter().map(|&i| &committee.members[i].public_key).collect();
        assert!(BlsCommittee::verify_threshold_signature(&pks, msg, &agg, 80));
    }

    #[test]
    fn test_insufficient_signers() {
        let committee = BlsCommittee::new();
        let msg = b"block-header";
        let indices: Vec<usize> = (0..50).collect();
        let sigs: Vec<Signature> = committee.sign_block(msg, &indices).into_iter().map(|(_, s)| s).collect();
        let agg = BlsCommittee::aggregate_signatures(&sigs).unwrap();
        let pks: Vec<&PublicKey> = indices.iter().map(|&i| &committee.members[i].public_key).collect();
        assert!(BlsCommittee::verify_threshold_signature(&pks, msg, &agg, 50));
        assert!(!BlsCommittee::verify_threshold_signature(&pks, msg, &agg, 80));
    }

    #[test]
    fn test_full_round() {
        let committee = BlsCommittee::new();
        let (success, sig_hex) = committee.attest_block("block-0x123");
        assert!(success);
        assert!(!sig_hex.is_empty());
    }
}
