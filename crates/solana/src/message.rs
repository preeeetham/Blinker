//! Message compilation (legacy) and decoding (legacy + v0).

use crate::error::{DecodeError, Reader};
use crate::instruction::{CompiledInstruction, Instruction};
use crate::{shortvec, Hash, Pubkey};

/// Bit set on the first byte of a versioned message.
const VERSION_PREFIX: u8 = 0x80;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MessageHeader {
    pub num_required_signatures: u8,
    pub num_readonly_signed_accounts: u8,
    pub num_readonly_unsigned_accounts: u8,
}

/// A legacy message: everything Blinker builds is legacy because every
/// wallet supports it and the account count is tiny.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Message {
    pub header: MessageHeader,
    pub account_keys: Vec<Pubkey>,
    pub recent_blockhash: Hash,
    pub instructions: Vec<CompiledInstruction>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AddressTableLookup {
    pub account_key: Pubkey,
    pub writable_indexes: Vec<u8>,
    pub readonly_indexes: Vec<u8>,
}

/// Either message format, as found on-chain.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VersionedMessage {
    Legacy(Message),
    V0 { message: Message, lookups: Vec<AddressTableLookup> },
}

impl VersionedMessage {
    pub fn inner(&self) -> &Message {
        match self {
            VersionedMessage::Legacy(m) => m,
            VersionedMessage::V0 { message, .. } => message,
        }
    }

    pub fn header(&self) -> MessageHeader {
        self.inner().header
    }
}

#[derive(Clone, Copy)]
struct KeyFlags {
    signer: bool,
    writable: bool,
}

impl Message {
    /// Compiles instructions into a legacy message.
    ///
    /// Account order is what the runtime requires: the fee payer first, then
    /// writable signers, readonly signers, writable non-signers and readonly
    /// non-signers. Within each group, keys keep first-seen order, which
    /// matches `solana-sdk` and keeps output deterministic.
    pub fn new(instructions: &[Instruction], payer: &Pubkey, recent_blockhash: Hash) -> Self {
        // Typical Blinker messages have < 12 keys; a Vec with linear search
        // beats a HashMap at this size and avoids hashing allocations.
        let mut keys: Vec<(Pubkey, KeyFlags)> = Vec::with_capacity(8);
        let mut upsert = |k: Pubkey, signer: bool, writable: bool| {
            if let Some((_, f)) = keys.iter_mut().find(|(p, _)| *p == k) {
                f.signer |= signer;
                f.writable |= writable;
            } else {
                keys.push((k, KeyFlags { signer, writable }));
            }
        };
        upsert(*payer, true, true);
        for ix in instructions {
            for m in &ix.accounts {
                upsert(m.pubkey, m.is_signer, m.is_writable);
            }
            upsert(ix.program_id, false, false);
        }

        let group = |f: &KeyFlags| match (f.signer, f.writable) {
            (true, true) => 0,
            (true, false) => 1,
            (false, true) => 2,
            (false, false) => 3,
        };
        // Stable sort keeps first-seen order within a group, payer stays first.
        keys.sort_by_key(|(_, f)| group(f));

        let header = MessageHeader {
            num_required_signatures: keys.iter().filter(|(_, f)| f.signer).count() as u8,
            num_readonly_signed_accounts: keys.iter().filter(|(_, f)| f.signer && !f.writable).count() as u8,
            num_readonly_unsigned_accounts: keys.iter().filter(|(_, f)| !f.signer && !f.writable).count() as u8,
        };
        let account_keys: Vec<Pubkey> = keys.into_iter().map(|(k, _)| k).collect();
        let index_of = |k: &Pubkey| account_keys.iter().position(|a| a == k).unwrap() as u8;

        let instructions = instructions
            .iter()
            .map(|ix| CompiledInstruction {
                program_id_index: index_of(&ix.program_id),
                accounts: ix.accounts.iter().map(|m| index_of(&m.pubkey)).collect(),
                data: ix.data.clone(),
            })
            .collect();

        Message { header, account_keys, recent_blockhash, instructions }
    }

    pub fn fee_payer(&self) -> Option<&Pubkey> {
        self.account_keys.first()
    }

    pub fn is_signer(&self, i: usize) -> bool {
        i < self.header.num_required_signatures as usize
    }

    pub fn is_writable(&self, i: usize) -> bool {
        let h = &self.header;
        let n = self.account_keys.len();
        let signers = h.num_required_signatures as usize;
        if i < signers {
            i < signers - h.num_readonly_signed_accounts as usize
        } else {
            i < n.saturating_sub(h.num_readonly_unsigned_accounts as usize)
        }
    }

    /// Exact serialized size, used to pre-size the output buffer.
    pub fn serialized_len(&self) -> usize {
        3 + shortvec::encoded_len(self.account_keys.len() as u16)
            + 32 * self.account_keys.len()
            + 32
            + shortvec::encoded_len(self.instructions.len() as u16)
            + self
                .instructions
                .iter()
                .map(|ix| {
                    1 + shortvec::encoded_len(ix.accounts.len() as u16)
                        + ix.accounts.len()
                        + shortvec::encoded_len(ix.data.len() as u16)
                        + ix.data.len()
                })
                .sum::<usize>()
    }

    /// Serializes into `out` (the bytes that get signed).
    pub fn serialize_into(&self, out: &mut Vec<u8>) {
        out.reserve(self.serialized_len());
        out.push(self.header.num_required_signatures);
        out.push(self.header.num_readonly_signed_accounts);
        out.push(self.header.num_readonly_unsigned_accounts);
        shortvec::encode(self.account_keys.len() as u16, out);
        for k in &self.account_keys {
            out.extend_from_slice(&k.0);
        }
        out.extend_from_slice(&self.recent_blockhash.0);
        shortvec::encode(self.instructions.len() as u16, out);
        for ix in &self.instructions {
            out.push(ix.program_id_index);
            shortvec::encode(ix.accounts.len() as u16, out);
            out.extend_from_slice(&ix.accounts);
            shortvec::encode(ix.data.len() as u16, out);
            out.extend_from_slice(&ix.data);
        }
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut out = Vec::new();
        self.serialize_into(&mut out);
        out
    }
}

/// Decodes the body shared by legacy and v0 messages (after any version byte).
fn decode_body(r: &mut Reader<'_>) -> Result<Message, DecodeError> {
    let header = MessageHeader {
        num_required_signatures: r.u8()?,
        num_readonly_signed_accounts: r.u8()?,
        num_readonly_unsigned_accounts: r.u8()?,
    };
    let n_keys = r.short_len_checked(32)?;
    let mut account_keys = Vec::with_capacity(n_keys);
    for _ in 0..n_keys {
        account_keys.push(Pubkey(r.array::<32>()?));
    }
    let recent_blockhash = Hash(r.array::<32>()?);

    // Smallest possible instruction is 3 bytes.
    let n_ix = r.short_len_checked(3)?;
    let mut instructions = Vec::with_capacity(n_ix);
    for _ in 0..n_ix {
        let program_id_index = r.u8()?;
        let n_acc = r.short_len_checked(1)?;
        let accounts = r.bytes(n_acc)?.to_vec();
        let n_data = r.short_len_checked(1)?;
        let data = r.bytes(n_data)?.to_vec();
        instructions.push(CompiledInstruction { program_id_index, accounts, data });
    }

    let h = &header;
    if h.num_required_signatures == 0 {
        return Err(DecodeError::Header("no signers"));
    }
    if h.num_readonly_signed_accounts >= h.num_required_signatures {
        return Err(DecodeError::Header("fee payer must be writable"));
    }
    if h.num_required_signatures as usize + h.num_readonly_unsigned_accounts as usize > account_keys.len() {
        return Err(DecodeError::Header("more header accounts than keys"));
    }
    Ok(Message { header, account_keys, recent_blockhash, instructions })
}

impl VersionedMessage {
    pub(crate) fn decode_from(r: &mut Reader<'_>) -> Result<Self, DecodeError> {
        let first = r.peek().ok_or(DecodeError::Eof(r.pos()))?;
        if first & VERSION_PREFIX == 0 {
            return Ok(VersionedMessage::Legacy(decode_body(r)?));
        }
        let version = r.u8()? & !VERSION_PREFIX;
        if version != 0 {
            return Err(DecodeError::UnsupportedVersion(version));
        }
        let message = decode_body(r)?;
        // Each lookup is at least 32 + 1 + 1 bytes.
        let n = r.short_len_checked(34)?;
        let mut lookups = Vec::with_capacity(n);
        for _ in 0..n {
            let account_key = Pubkey(r.array::<32>()?);
            let w = r.short_len_checked(1)?;
            let writable_indexes = r.bytes(w)?.to_vec();
            let ro = r.short_len_checked(1)?;
            let readonly_indexes = r.bytes(ro)?.to_vec();
            lookups.push(AddressTableLookup { account_key, writable_indexes, readonly_indexes });
        }
        Ok(VersionedMessage::V0 { message, lookups })
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, DecodeError> {
        let mut r = Reader::new(bytes);
        let m = Self::decode_from(&mut r)?;
        if r.remaining() != 0 {
            return Err(DecodeError::Trailing(r.remaining()));
        }
        Ok(m)
    }
}
