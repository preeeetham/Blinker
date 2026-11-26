//! Extracts the facts Blinker verifies (memos, SOL and token transfers) from a
//! decoded transaction. Works on resolved account keys so it handles both
//! legacy and v0 transactions.

use crate::message::{Message, VersionedMessage};
use crate::programs::{MEMO_PROGRAM_ID, MEMO_V1_PROGRAM_ID, SYSTEM_PROGRAM_ID, TOKEN_2022_PROGRAM_ID, TOKEN_PROGRAM_ID};
use crate::Pubkey;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action<'a> {
    Memo(&'a str),
    SolTransfer { from: Pubkey, to: Pubkey, lamports: u64 },
    /// `to` is the destination token account, not the wallet.
    TokenTransfer { mint: Option<Pubkey>, to: Pubkey, owner: Pubkey, amount: u64 },
}

/// Walks top-level instructions and yields the ones Blinker understands.
///
/// `keys` must be the full account list: static keys followed by any
/// loaded (lookup-table) addresses, as returned by `getTransaction`'s
/// `meta.loadedAddresses`.
pub fn actions<'a>(message: &'a Message, keys: &'a [Pubkey]) -> impl Iterator<Item = Action<'a>> + 'a {
    message.instructions.iter().filter_map(move |ix| {
        let program = keys.get(ix.program_id_index as usize)?;
        let acc = |i: usize| ix.accounts.get(i).and_then(|&k| keys.get(k as usize)).copied();
        let d = &ix.data;
        if *program == MEMO_PROGRAM_ID || *program == MEMO_V1_PROGRAM_ID {
            return std::str::from_utf8(d).ok().map(Action::Memo);
        }
        if *program == SYSTEM_PROGRAM_ID {
            if d.len() == 12 && d[..4] == 2u32.to_le_bytes() {
                let lamports = u64::from_le_bytes(d[4..12].try_into().ok()?);
                return Some(Action::SolTransfer { from: acc(0)?, to: acc(1)?, lamports });
            }
            return None;
        }
        if *program == TOKEN_PROGRAM_ID || *program == TOKEN_2022_PROGRAM_ID {
            return match d.first()? {
                // Transfer { amount }
                3 if d.len() >= 9 => Some(Action::TokenTransfer {
                    mint: None,
                    to: acc(1)?,
                    owner: acc(2)?,
                    amount: u64::from_le_bytes(d[1..9].try_into().ok()?),
                }),
                // TransferChecked { amount, decimals }
                12 if d.len() >= 10 => Some(Action::TokenTransfer {
                    mint: Some(acc(1)?),
                    to: acc(2)?,
                    owner: acc(3)?,
                    amount: u64::from_le_bytes(d[1..9].try_into().ok()?),
                }),
                _ => None,
            };
        }
        None
    })
}

/// Full key list for a decoded message given the RPC-provided loaded addresses.
pub fn resolve_keys(msg: &VersionedMessage, loaded_writable: &[Pubkey], loaded_readonly: &[Pubkey]) -> Vec<Pubkey> {
    let m = msg.inner();
    let mut keys = Vec::with_capacity(m.account_keys.len() + loaded_writable.len() + loaded_readonly.len());
    keys.extend_from_slice(&m.account_keys);
    if matches!(msg, VersionedMessage::V0 { .. }) {
        keys.extend_from_slice(loaded_writable);
        keys.extend_from_slice(loaded_readonly);
    }
    keys
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{programs, Hash};

    #[test]
    fn finds_memo_and_transfer() {
        let payer = Pubkey([3; 32]);
        let to = Pubkey([4; 32]);
        let ixs = [programs::memo("blinker:fee:xyz"), programs::system_transfer(&payer, &to, 1234)];
        let m = Message::new(&ixs, &payer, Hash::default());
        let got: Vec<_> = actions(&m, &m.account_keys).collect();
        assert_eq!(
            got,
            vec![Action::Memo("blinker:fee:xyz"), Action::SolTransfer { from: payer, to, lamports: 1234 }]
        );
    }

    #[test]
    fn out_of_range_indices_are_skipped() {
        let mut m = Message::new(&[programs::memo("x")], &Pubkey([1; 32]), Hash::default());
        m.instructions[0].program_id_index = 200;
        assert_eq!(actions(&m, &m.account_keys).count(), 0);
    }
}
