#[cfg(not(feature = "std"))]
use alloc::{borrow::ToOwned, vec, vec::Vec};
use lazy_static::lazy_static;
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

impl Into<(AccountAddress, u128)> for Deposit {
    fn into(self) -> (AccountAddress, u128) {
        (self.destination, self.amount)
    }
}

lazy_static! {
    /// Parsing template for Move VM type -> Rust type conversion and matching
    pub static ref DEPOSIT_TEMPLATE: StructTag = StructTag {
        address: ROOT_ADDRESS.clone(),
        module: ident_str!("deposit").to_owned(),
        name: ident_str!("Deposit").to_owned(),
        type_params: vec![],
    };
    /// Publisher address of DepositModule and other `std`
    pub static ref ROOT_ADDRESS: AccountAddress = AccountAddress::from_hex_literal("0x01").unwrap();
    /// Actual bytes of DepositModule module for use with pallet/Mvm
    pub static ref MOVE_DEPOSIT_MODULE_BYTES: Vec<u8> =
        include_bytes!("../../contracts/deposit.mv").to_vec();
    /// signer `std` module bytes
    pub static ref SIGNER_MODULE_BYTES: Vec<u8> = include_bytes!("../../contracts/signer.mv").to_vec();
    /// deposit transfer script for executing deposit
    pub static ref DEPOSIT_SCRIPT_BYTES: Vec<u8> = include_bytes!("../../contracts/transfer.mv").to_vec();
    /// balance checking script to get native balance fo account
    pub static ref CHECK_BALANCE_OF_SCRIPT_BYTES: Vec<u8> = include_bytes!("../../contracts/check_balance_of.mv").to_vec();
}
