use base64::Engine;

use crate::error::{DecodeError, Reader};
use crate::message::{Message, VersionedMessage};
use crate::{shortvec, PACKET_DATA_SIZE};

pub type Signature = [u8; 64];

/// A legacy transaction as Blinker builds it for wallets.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Transaction {
    pub signatures: Vec<Signature>,
    pub message: Message,
}

impl Transaction {
    /// An unsigned transaction: one zeroed signature slot per required signer.
    /// This is the shape the Solana Actions spec expects in a POST response.
    pub fn new_unsigned(message: Message) -> Self {
        let n = message.header.num_required_signatures as usize;
        Self { signatures: vec![[0u8; 64]; n], message }
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(
            shortvec::encoded_len(self.signatures.len() as u16) + 64 * self.signatures.len() + self.message.serialized_len(),
        );
        shortvec::encode(self.signatures.len() as u16, &mut out);
        for s in &self.signatures {
            out.extend_from_slice(s);
        }
        self.message.serialize_into(&mut out);
        out
    }

    pub fn to_base64(&self) -> String {
        base64::engine::general_purpose::STANDARD.encode(self.serialize())
    }

    #[cfg(feature = "signer")]
    pub fn sign(&mut self, signers: &[&crate::Keypair]) {
        let msg = self.message.serialize();
        for kp in signers {
            if let Some(i) = self.message.account_keys.iter().position(|k| *k == kp.pubkey())
                && i < self.signatures.len()
            {
                self.signatures[i] = kp.sign(&msg);
            }
        }
    }

    /// The transaction id is its first signature.
    pub fn signature_b58(&self) -> Option<String> {
        self.signatures.first().map(|s| bs58::encode(s).into_string())
    }
}

/// A transaction of either version, as decoded from the network.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VersionedTransaction {
    pub signatures: Vec<Signature>,
    pub message: VersionedMessage,
}

impl VersionedTransaction {
    /// Strict decoder: bounded size, minimal compact-u16s, no trailing bytes,
    /// and signature count must equal the header's signer count.
    pub fn decode(bytes: &[u8]) -> Result<Self, DecodeError> {
        if bytes.len() > PACKET_DATA_SIZE {
            return Err(DecodeError::TooLarge { max: PACKET_DATA_SIZE });
        }
        let mut r = Reader::new(bytes);
        let n_sigs = r.short_len_checked(64)?;
        let mut signatures = Vec::with_capacity(n_sigs);
        for _ in 0..n_sigs {
            signatures.push(r.array::<64>()?);
        }
        let message = VersionedMessage::decode_from(&mut r)?;
        if r.remaining() != 0 {
            return Err(DecodeError::Trailing(r.remaining()));
        }
        if message.header().num_required_signatures as usize != n_sigs {
            return Err(DecodeError::Header("signature count != required signers"));
        }
        Ok(Self { signatures, message })
    }

    pub fn decode_base64(s: &str) -> Result<Self, DecodeError> {
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(s.trim())
            .map_err(|_| DecodeError::Base64)?;
        Self::decode(&bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{programs, Hash, Pubkey};

    fn sample() -> Transaction {
        let payer: Pubkey = "9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM".parse().unwrap();
        let to: Pubkey = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".parse().unwrap();
        let ixs = [
            programs::memo("blinker:abc123"),
            programs::system_transfer(&payer, &to, 1_000_000),
        ];
        Transaction::new_unsigned(Message::new(&ixs, &payer, Hash([7; 32])))
    }

    #[test]
    fn account_ordering_and_header() {
        let tx = sample();
        let m = &tx.message;
        // payer(sw), to(w), memo(r), system(r)
        assert_eq!(m.header.num_required_signatures, 1);
        assert_eq!(m.header.num_readonly_signed_accounts, 0);
        assert_eq!(m.header.num_readonly_unsigned_accounts, 2);
        assert_eq!(m.account_keys[0].to_string(), "9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM");
        assert!(m.is_writable(0) && m.is_writable(1));
        assert!(!m.is_writable(2) && !m.is_writable(3));
        assert_eq!(m.serialized_len(), m.serialize().len());
    }

    #[test]
    fn roundtrip() {
        let tx = sample();
        let bytes = tx.serialize();
        let decoded = VersionedTransaction::decode(&bytes).unwrap();
        assert_eq!(decoded.signatures, tx.signatures);
        assert_eq!(decoded.message, VersionedMessage::Legacy(tx.message.clone()));
    }

    #[test]
    fn rejects_trailing_and_truncated() {
        let mut bytes = sample().serialize();
        bytes.push(0);
        assert!(matches!(VersionedTransaction::decode(&bytes), Err(DecodeError::Trailing(1))));
        bytes.truncate(bytes.len() - 5);
        assert!(VersionedTransaction::decode(&bytes).is_err());
    }

    #[test]
    fn huge_length_prefix_does_not_allocate() {
        // 0xffff signatures claimed, only 3 bytes present.
        assert!(VersionedTransaction::decode(&[0xff, 0xff, 0x03]).is_err());
    }

    #[cfg(feature = "signer")]
    #[test]
    fn signature_verifies() {
        let kp = crate::Keypair::generate();
        let to = Pubkey([9; 32]);
        let ixs = [programs::system_transfer(&kp.pubkey(), &to, 5)];
        let mut tx = Transaction::new_unsigned(Message::new(&ixs, &kp.pubkey(), Hash([1; 32])));
        tx.sign(&[&kp]);
        assert!(kp.verify(&tx.message.serialize(), &tx.signatures[0]));
    }
}
