#[cfg(not(feature = "std"))]
use alloc::{borrow::ToOwned, vec, vec::Vec};
use lazy_static::lazy_static;
/// Re-export for pallet to consume as well
pub use move_core_types::language_storage::CORE_CODE_ADDRESS;
use move_core_types::{account_address::AccountAddress, ident_str, language_storage::StructTag};
use serde::{Deserialize, Serialize};

/// Mirroring structure of Move::deposit::Deposit
/// Designed for bridging move transfer and native tokens of substrate
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Deposit {
    destination: AccountAddress,
    amount: u128,
}

impl Deposit {
    pub fn new(destination: AccountAddress, amount: u128) -> Self {
        Deposit {
            destination,
            amount,
        }
    }
}

impl From<Deposit> for (AccountAddress, u128) {
    fn from(val: Deposit) -> Self {
        (val.destination, val.amount)
    }
}

lazy_static! {
    /// Parsing template for Move VM type -> Rust type conversion and matching
    pub static ref DEPOSIT_TEMPLATE: StructTag = StructTag {
        address: CORE_CODE_ADDRESS,
        module: ident_str!("deposit").to_owned(),
        name: ident_str!("Deposit").to_owned(),
        type_params: vec![],
    };
    /// Actual bytes of DepositModule module for use with pallet/Mvm
    pub static ref MOVE_DEPOSIT_MODULE_BYTES: Vec<u8> =
        include_bytes!("../../contracts/deposit.mv").to_vec();
    /// signer `std` module bytes
    pub static ref SIGNER_MODULE_BYTES: Vec<u8> = include_bytes!("../../contracts/signer.mv").to_vec();
    /// deposit transfer script for executing deposit
    pub static ref DEPOSIT_SCRIPT_BYTES: Vec<u8> = include_bytes!("../../contracts/transfer.mv").to_vec();
    /// balance checking script to get native balance fo account
    pub static ref CHECK_BALANCE_OF_SCRIPT_BYTES: Vec<u8> = include_bytes!("../../contracts/check_balance_of.mv").to_vec();
    /// test only script to check if parameter parsing works as expected
    pub static ref ALL_YOUR_MONEY_BELONG_TO_ME: Vec<u8> = include_bytes!("../../contracts/all_of_your_money_belong_to_me.mv").to_vec();
    /// test only script to check if struct wropper around signer is captured alright
    pub static ref BOGUS_SIGNER_WRAPPER_SCRIPT: Vec<u8> = include_bytes!("../../contracts/bogus_struct.mv").to_vec();
    ///  test only bogus Test module wrapper struct around signer
    pub static ref BOGUS_SIGNER_WRAPPER_MODULE: Vec<u8> = include_bytes!("../../contracts/test_struct.mv").to_vec();
}
