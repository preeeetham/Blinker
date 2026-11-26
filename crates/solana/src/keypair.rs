use ed25519_dalek::{Signer, SigningKey, Verifier, VerifyingKey};

use crate::error::DecodeError;
use crate::transaction::Signature;
use crate::Pubkey;

/// An ed25519 keypair, compatible with `solana-keygen` JSON files
/// (a 64-number array: secret seed || public key).
pub struct Keypair {
    signing: SigningKey,
}

impl Keypair {
    pub fn generate() -> Self {
        let seed: [u8; 32] = rand::random();
        Self { signing: SigningKey::from_bytes(&seed) }
    }

    pub fn from_seed(seed: &[u8; 32]) -> Self {
        Self { signing: SigningKey::from_bytes(seed) }
    }

    pub fn pubkey(&self) -> Pubkey {
        Pubkey(self.signing.verifying_key().to_bytes())
    }

    pub fn sign(&self, msg: &[u8]) -> Signature {
        self.signing.sign(msg).to_bytes()
    }

    pub fn verify(&self, msg: &[u8], sig: &Signature) -> bool {
        verify(&self.pubkey(), msg, sig)
    }

    /// 64 bytes: seed || pubkey.
    pub fn to_bytes(&self) -> [u8; 64] {
        let mut out = [0u8; 64];
        out[..32].copy_from_slice(&self.signing.to_bytes());
        out[32..].copy_from_slice(&self.pubkey().0);
        out
    }

    pub fn from_bytes(b: &[u8]) -> Result<Self, DecodeError> {
        if b.len() != 64 {
            return Err(DecodeError::Length { expected: 64, got: b.len() });
        }
        let kp = Self::from_seed(b[..32].try_into().unwrap());
        if kp.pubkey().0 != b[32..] {
            return Err(DecodeError::Header("public key does not match secret"));
        }
        Ok(kp)
    }

    pub fn to_base58(&self) -> String {
        bs58::encode(self.to_bytes()).into_string()
    }

    pub fn from_base58(s: &str) -> Result<Self, DecodeError> {
        let b = bs58::decode(s.trim()).into_vec().map_err(|_| DecodeError::Base58)?;
        Self::from_bytes(&b)
    }

    /// `solana-keygen` JSON format.
    pub fn to_json(&self) -> String {
        let nums: Vec<String> = self.to_bytes().iter().map(|b| b.to_string()).collect();
        format!("[{}]", nums.join(","))
    }

    pub fn from_json(s: &str) -> Result<Self, DecodeError> {
        let bytes: Result<Vec<u8>, _> = s
            .trim()
            .trim_start_matches('[')
            .trim_end_matches(']')
            .split(',')
            .map(|n| n.trim().parse::<u8>())
            .collect();
        Self::from_bytes(&bytes.map_err(|_| DecodeError::Header("bad keypair json"))?)
    }
}

pub fn verify(pubkey: &Pubkey, msg: &[u8], sig: &Signature) -> bool {
    let Ok(vk) = VerifyingKey::from_bytes(&pubkey.0) else { return false };
    vk.verify(msg, &ed25519_dalek::Signature::from_bytes(sig)).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_and_base58_roundtrip() {
        let kp = Keypair::generate();
        assert_eq!(Keypair::from_json(&kp.to_json()).unwrap().pubkey(), kp.pubkey());
        assert_eq!(Keypair::from_base58(&kp.to_base58()).unwrap().pubkey(), kp.pubkey());
    }
}
