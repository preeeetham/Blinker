//! Instruction builders for the few programs Blinker talks to.

use crate::{AccountMeta, Instruction, Pubkey};

pub const SYSTEM_PROGRAM_ID: Pubkey = Pubkey::from_str_const("11111111111111111111111111111111");
pub const MEMO_PROGRAM_ID: Pubkey = Pubkey::from_str_const("MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr");
pub const MEMO_V1_PROGRAM_ID: Pubkey = Pubkey::from_str_const("Memo1UhkJRfHyvLMcVucJwxXeuD728EqVDDwQDxFMNo");
pub const TOKEN_PROGRAM_ID: Pubkey = Pubkey::from_str_const("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");
pub const TOKEN_2022_PROGRAM_ID: Pubkey = Pubkey::from_str_const("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb");
pub const ASSOCIATED_TOKEN_PROGRAM_ID: Pubkey =
    Pubkey::from_str_const("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL");

/// System program `Transfer` (instruction index 2).
pub fn system_transfer(from: &Pubkey, to: &Pubkey, lamports: u64) -> Instruction {
    let mut data = Vec::with_capacity(12);
    data.extend_from_slice(&2u32.to_le_bytes());
    data.extend_from_slice(&lamports.to_le_bytes());
    Instruction {
        program_id: SYSTEM_PROGRAM_ID,
        accounts: vec![AccountMeta::writable(*from, true), AccountMeta::writable(*to, false)],
        data,
    }
}

/// SPL Memo with no signer accounts (the memo is just recorded in the log).
pub fn memo(text: &str) -> Instruction {
    Instruction { program_id: MEMO_PROGRAM_ID, accounts: vec![], data: text.as_bytes().to_vec() }
}

/// Associated token account address for (`owner`, `mint`) under `token_program`.
pub fn associated_token_address(owner: &Pubkey, mint: &Pubkey, token_program: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[owner.as_bytes(), token_program.as_bytes(), mint.as_bytes()], &ASSOCIATED_TOKEN_PROGRAM_ID)
        .expect("an ATA bump always exists")
        .0
}

/// `CreateIdempotent` (index 1): creates the ATA if missing, no-op otherwise.
pub fn create_ata_idempotent(payer: &Pubkey, owner: &Pubkey, mint: &Pubkey, token_program: &Pubkey) -> Instruction {
    let ata = associated_token_address(owner, mint, token_program);
    create_ata_idempotent_with(payer, owner, &ata, mint, token_program)
}

/// `create_ata_idempotent` with a precomputed ATA address — PDA derivation
/// is the expensive part (~8µs), so hot paths pass a cached one.
pub fn create_ata_idempotent_with(payer: &Pubkey, owner: &Pubkey, ata: &Pubkey, mint: &Pubkey, token_program: &Pubkey) -> Instruction {
    Instruction {
        program_id: ASSOCIATED_TOKEN_PROGRAM_ID,
        accounts: vec![
            AccountMeta::writable(*payer, true),
            AccountMeta::writable(*ata, false),
            AccountMeta::readonly(*owner, false),
            AccountMeta::readonly(*mint, false),
            AccountMeta::readonly(SYSTEM_PROGRAM_ID, false),
            AccountMeta::readonly(*token_program, false),
        ],
        data: vec![1],
    }
}

/// SPL Token `TransferChecked` (index 12): checks mint and decimals on-chain,
/// so a wallet can render the amount correctly.
pub fn token_transfer_checked(
    token_program: &Pubkey,
    source: &Pubkey,
    mint: &Pubkey,
    destination: &Pubkey,
    owner: &Pubkey,
    amount: u64,
    decimals: u8,
) -> Instruction {
    let mut data = Vec::with_capacity(10);
    data.push(12);
    data.extend_from_slice(&amount.to_le_bytes());
    data.push(decimals);
    Instruction {
        program_id: *token_program,
        accounts: vec![
            AccountMeta::writable(*source, false),
            AccountMeta::readonly(*mint, false),
            AccountMeta::writable(*destination, false),
            AccountMeta::readonly(*owner, true),
        ],
        data,
    }
}

/// Reads `decimals` from a mint account's data (offset 44 in both the
/// Token and Token-2022 base layout).
pub fn mint_decimals(account_data: &[u8]) -> Option<u8> {
    if account_data.len() < 82 {
        return None;
    }
    // is_initialized flag at 45
    (account_data[45] == 1).then_some(account_data[44])
}

/// Converts a UI amount (e.g. 1.5 tokens) to base units, rejecting values
/// that are negative, non-finite or overflow u64.
pub fn ui_to_base_units(ui: f64, decimals: u8) -> Option<u64> {
    if !ui.is_finite() || ui <= 0.0 || decimals > 19 {
        return None;
    }
    let scaled = (ui * 10f64.powi(decimals as i32)).round();
    (scaled >= 1.0 && scaled < u64::MAX as f64).then_some(scaled as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transfer_layout() {
        let ix = system_transfer(&Pubkey([1; 32]), &Pubkey([2; 32]), 42);
        assert_eq!(ix.data, [2, 0, 0, 0, 42, 0, 0, 0, 0, 0, 0, 0]);
    }

    #[test]
    fn ui_amounts() {
        assert_eq!(ui_to_base_units(1.5, 6), Some(1_500_000));
        assert_eq!(ui_to_base_units(0.001, 9), Some(1_000_000));
        assert_eq!(ui_to_base_units(0.0, 9), None);
        assert_eq!(ui_to_base_units(-1.0, 9), None);
        assert_eq!(ui_to_base_units(f64::NAN, 9), None);
        assert_eq!(ui_to_base_units(1e30, 9), None);
    }
}
