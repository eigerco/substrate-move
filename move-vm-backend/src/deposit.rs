#[cfg(not(feature = "std"))]
use alloc::{borrow::ToOwned, vec, vec::Vec};
use lazy_static::lazy_static;
use move_core_types::{
    account_address::AccountAddress,
    ident_str,
    language_storage::{StructTag, TypeTag},
};
use serde::{Deserialize, Serialize};

/// Mirroring structure of Move::deposit::Deposit
/// Designed for bridging move transfer and native tokens of substrate
#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct Deposit {
    destination: AccountAddress,
    amount: u128,
}

impl Into<(AccountAddress, u128)> for Deposit {
    fn into(self) -> (AccountAddress, u128) {
        (self.destination, self.amount)
    }
}

lazy_static! {
    pub(crate) static ref DEPOSIT_TEMPLATE: StructTag = StructTag {
        address: ROOT_ADDRESS.clone(),
        module: ident_str!("DepositModule").to_owned(),
        name: ident_str!("Deposit").to_owned(),
        type_params: vec![TypeTag::U8],
    };
    /// Publisher address of DepositModule and other `std`
    pub static ref ROOT_ADDRESS: AccountAddress = AccountAddress::from_hex_literal("0x01").unwrap();
    /// Actual bytes of DepositModule module for use with pallet/Mvm
    pub static ref MOVE_DEPOSIT_MODULE_BYTES: Vec<u8> =
        include_bytes!("../../contracts/DepositModule.mv").to_vec();
    /// signer `std` module bytes
    pub static ref SIGNER_MODULE_BYTES: Vec<u8> = include_bytes!("../../contracts/signer.mv").to_vec();
}
