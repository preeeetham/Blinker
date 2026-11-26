//! Builders for the three transactions Blinker hands to wallets, and the
//! memo tags that let the verifier attribute them.
//!
//! * fee:      creator → treasury, memo `blinker:fee:<id>`
//! * SOL pay:  payer → recipient, memo `blinker:pay:<id>`
//! * SPL pay:  create recipient ATA (idempotent) + TransferChecked, same memo
//!
//! Payment transfers carry the treasury as an extra read-only "reference"
//! account (the Solana Pay pattern). Programs ignore trailing accounts, but
//! the transaction now *mentions* the treasury, so a single
//! `logsSubscribe { mentions: [treasury] }` sees fees and payments alike.

use blinker_analytics::BlinkId;
use std::collections::HashMap;
use std::sync::RwLock;

use blinker_solana::{programs, AccountMeta, Hash, Message, Pubkey, Transaction};

pub const FEE_PREFIX: &str = "blinker:fee:";
pub const PAY_PREFIX: &str = "blinker:pay:";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tag {
    Fee(BlinkId),
    Pay(BlinkId),
}

pub fn parse_memo(memo: &str) -> Option<Tag> {
    // Wallets/explorers sometimes surface memos with surrounding whitespace.
    let m = memo.trim();
    if let Some(id) = m.strip_prefix(FEE_PREFIX) {
        return BlinkId::parse(id).map(Tag::Fee);
    }
    m.strip_prefix(PAY_PREFIX).and_then(BlinkId::parse).map(Tag::Pay)
}

pub fn fee_tx(creator: &Pubkey, treasury: &Pubkey, lamports: u64, id: &BlinkId, bh: Hash) -> Transaction {
    let ixs = [
        programs::memo(&format!("{FEE_PREFIX}{id}")),
        programs::system_transfer(creator, treasury, lamports),
    ];
    Transaction::new_unsigned(Message::new(&ixs, creator, bh))
}

pub fn sol_payment_tx(payer: &Pubkey, recipient: &Pubkey, lamports: u64, reference: &Pubkey, id: &BlinkId, bh: Hash) -> Transaction {
    let mut transfer = programs::system_transfer(payer, recipient, lamports);
    transfer.accounts.push(AccountMeta::readonly(*reference, false));
    let ixs = [transfer, programs::memo(&format!("{PAY_PREFIX}{id}"))];
    Transaction::new_unsigned(Message::new(&ixs, payer, bh))
}

pub struct SplPayment<'a> {
    pub payer: &'a Pubkey,
    pub recipient: &'a Pubkey,
    /// The recipient's ATA. Fixed per blink, so callers pass a cached value
    /// (see `AtaCache`) instead of re-deriving the PDA on every request.
    pub recipient_ata: &'a Pubkey,
    pub mint: &'a Pubkey,
    pub token_program: &'a Pubkey,
    pub amount: u64,
    pub decimals: u8,
    pub reference: &'a Pubkey,
}

pub fn spl_payment_tx(p: &SplPayment<'_>, id: &BlinkId, bh: Hash) -> Transaction {
    let src = programs::associated_token_address(p.payer, p.mint, p.token_program);
    let mut transfer =
        programs::token_transfer_checked(p.token_program, &src, p.mint, p.recipient_ata, p.payer, p.amount, p.decimals);
    transfer.accounts.push(AccountMeta::readonly(*p.reference, false));
    let ixs = [
        programs::create_ata_idempotent_with(p.payer, p.recipient, p.recipient_ata, p.mint, p.token_program),
        transfer,
        programs::memo(&format!("{PAY_PREFIX}{id}")),
    ];
    Transaction::new_unsigned(Message::new(&ixs, p.payer, bh))
}

/// Memoises ATA derivations. Deriving a PDA costs ~8µs (SHA-256 plus an
/// Edwards-point decompression per bump tried) and the recipient side of a
/// blink never changes, so SPL action POSTs and payment verification hit
/// this map instead. Bounded by a simple clear-on-full policy.
#[derive(Default)]
pub struct AtaCache {
    map: RwLock<HashMap<(Pubkey, Pubkey, Pubkey), Pubkey>>,
}

const ATA_CACHE_MAX: usize = 65_536;

impl AtaCache {
    pub fn get(&self, owner: &Pubkey, mint: &Pubkey, token_program: &Pubkey) -> Pubkey {
        let key = (*owner, *mint, *token_program);
        if let Some(a) = self.map.read().unwrap().get(&key) {
            return *a;
        }
        let ata = programs::associated_token_address(owner, mint, token_program);
        let mut m = self.map.write().unwrap();
        if m.len() >= ATA_CACHE_MAX {
            m.clear();
        }
        m.insert(key, ata);
        ata
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memo_tags() {
        let id = BlinkId::parse("AbCdEf123456").unwrap();
        assert_eq!(parse_memo("blinker:fee:AbCdEf123456"), Some(Tag::Fee(id)));
        assert_eq!(parse_memo(" blinker:pay:AbCdEf123456 "), Some(Tag::Pay(id)));
        assert_eq!(parse_memo("blinker:pay:short"), None);
        assert_eq!(parse_memo("hello"), None);
    }

    #[test]
    fn spl_tx_matches_uncached_derivation() {
        let id = BlinkId::parse("AbCdEf123456").unwrap();
        let (payer, to, mint, r) = (Pubkey([1; 32]), Pubkey([2; 32]), Pubkey([5; 32]), Pubkey([3; 32]));
        let tp = programs::TOKEN_PROGRAM_ID;
        let cache = AtaCache::default();
        let ata = cache.get(&to, &mint, &tp);
        assert_eq!(ata, programs::associated_token_address(&to, &mint, &tp));
        let p = SplPayment { payer: &payer, recipient: &to, recipient_ata: &ata, mint: &mint, token_program: &tp, amount: 7, decimals: 6, reference: &r };
        let tx = spl_payment_tx(&p, &id, Hash::default());
        // Identical to building the create-ATA instruction the uncached way.
        let reference = programs::create_ata_idempotent(&payer, &to, &mint, &tp);
        let keys = &tx.message.account_keys;
        let ix = &tx.message.instructions[0];
        let resolved: Vec<Pubkey> = ix.accounts.iter().map(|&i| keys[i as usize]).collect();
        assert_eq!(resolved, reference.accounts.iter().map(|m| m.pubkey).collect::<Vec<_>>());
    }

    #[test]
    fn payment_mentions_reference() {
        let id = BlinkId::parse("AbCdEf123456").unwrap();
        let (payer, to, treasury) = (Pubkey([1; 32]), Pubkey([2; 32]), Pubkey([3; 32]));
        let tx = sol_payment_tx(&payer, &to, 10, &treasury, &id, Hash::default());
        assert!(tx.message.account_keys.contains(&treasury));
        // Reference must not be writable or a signer.
        let i = tx.message.account_keys.iter().position(|k| *k == treasury).unwrap();
        assert!(!tx.message.is_writable(i) && !tx.message.is_signer(i));
    }
}
