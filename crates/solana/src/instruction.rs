use crate::Pubkey;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccountMeta {
    pub pubkey: Pubkey,
    pub is_signer: bool,
    pub is_writable: bool,
}

impl AccountMeta {
    pub const fn writable(pubkey: Pubkey, is_signer: bool) -> Self {
        Self { pubkey, is_signer, is_writable: true }
    }
    pub const fn readonly(pubkey: Pubkey, is_signer: bool) -> Self {
        Self { pubkey, is_signer, is_writable: false }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Instruction {
    pub program_id: Pubkey,
    pub accounts: Vec<AccountMeta>,
    pub data: Vec<u8>,
}

/// An instruction after compilation into a message: accounts are indices
/// into the message's account-key table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompiledInstruction {
    pub program_id_index: u8,
    pub accounts: Vec<u8>,
    pub data: Vec<u8>,
}
