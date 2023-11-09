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

pub(crate) const CORE_CODE_ADDRESS: AccountAddress = AccountAddress::new([
    0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
    0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 1u8,
]);

lazy_static! {
    pub(crate) static ref DEPOSIT_TEMPLATE: StructTag = StructTag {
        address: CORE_CODE_ADDRESS,
        module: ident_str!("DepositModule").to_owned(),
        name: ident_str!("Deposit").to_owned(),
        type_params: vec![TypeTag::U8],
    };
}
