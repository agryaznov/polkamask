#![cfg_attr(not(feature = "std"), no_std)]

mod account;
mod signing;

#[cfg(any(feature = "std", test))]
mod input;

#[cfg(any(feature = "std", test))]
pub use input::{compose_and_sign_tx, ContractInput, EthTxInput};

pub use account::AccountId20;
pub use signing::{EthereumSignature, EthereumSigner};

pub use ethereum::{
    AccessListItem, BlockV2 as Block, EnvelopedDecodable, EnvelopedEncodable, LegacyTransaction,
    LegacyTransactionMessage, Log, ReceiptV3 as Receipt, TransactionAction, TransactionSignature,
    TransactionV2 as EthTransaction,
};

pub use ethereum_types::{H160, H256, H64, U256, U64};
