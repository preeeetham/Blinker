//! A small, dependency-light implementation of the parts of the Solana wire
//! format Blinker needs.
//!
//! The official `solana-sdk` pulls in hundreds of crates. Blinker only has to
//! build a handful of instruction types, serialize unsigned transactions for a
//! wallet to sign, and decode transactions fetched from RPC to verify
//! payments. Doing that by hand keeps compile times and binary size down, and
//! it makes the decoder a good fuzzing target: every length field in a
//! transaction is attacker-controlled.
//!
//! Layout reference: <https://solana.com/docs/core/transactions>

pub mod error;
pub mod hash;
pub mod instruction;
#[cfg(feature = "signer")]
pub mod keypair;
pub mod message;
pub mod parse;
pub mod programs;
pub mod pubkey;
pub mod shortvec;
pub mod transaction;

pub use error::DecodeError;
pub use hash::Hash;
pub use instruction::{AccountMeta, Instruction};
#[cfg(feature = "signer")]
pub use keypair::Keypair;
pub use message::{Message, VersionedMessage};
pub use pubkey::Pubkey;
pub use transaction::{Transaction, VersionedTransaction};

/// Lamports per SOL.
pub const LAMPORTS_PER_SOL: u64 = 1_000_000_000;

/// Hard ceiling on a serialized transaction (one IPv6 MTU minus headers).
pub const PACKET_DATA_SIZE: usize = 1232;
