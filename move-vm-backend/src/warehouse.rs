use crate::{
    deposit::{Deposit, DEPOSIT_TEMPLATE},
    storage::Storage,
    SubstrateAPI,
};
use alloc::{
    collections::{
        btree_map::Entry::{Occupied, Vacant},
        BTreeMap,
    },
    string::ToString,
    vec,
    vec::Vec,
};
use anyhow::{bail, Error, Result};
use core::ops::Deref;
use move_core_types::account_address::AccountAddress;
use move_core_types::effects::{
    ChangeSet,
    Op::{self, Delete, Modify, New},
};
use move_core_types::identifier::Identifier;
use move_core_types::language_storage::{ModuleId, StructTag};
use move_core_types::resolver::{ModuleResolver, ResourceResolver};
use move_core_types::value::{MoveStructLayout, MoveTypeLayout};
use move_vm_types::values::{Struct, Value};
use serde::{Deserialize, Serialize};
use MoveTypeLayout::Address;

/// Structure holding account data which is held under one Move address
/// in Substrate storage).
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
struct AccountData {
    /// Hashmap of the modules kept under this account.
    modules: BTreeMap<Identifier, Vec<u8>>,
    /// Hashmap of the resources kept under this account.
    resources: BTreeMap<StructTag, Vec<u8>>,
}

impl AccountData {
    fn apply_changes<K, V>(
        map: &mut BTreeMap<K, V>,
        changes: impl IntoIterator<Item = (K, Op<V>)>,
    ) -> Result<()>
    where
        K: Ord + core::fmt::Debug,
    {
        for (k, op) in changes.into_iter() {
            match (map.entry(k), op) {
                (Occupied(entry), New(_)) => {
                    bail!(
                        "Failed to apply changes -- key {:?} already exists",
                        entry.key()
                    )
                }
                (Occupied(entry), Delete) => {
                    entry.remove();
                }
                (Occupied(entry), Modify(val)) => {
                    *entry.into_mut() = val;
                }
                (Vacant(entry), New(val)) => {
                    entry.insert(val);
                }
                (Vacant(entry), Delete | Modify(_)) => bail!(
                    "Failed to apply changes -- key {:?} does not exist",
                    entry.key()
                ),
            }
        }
        Ok(())
    }
}

/// Move VM storage implementation for Substrate storage.
pub(crate) struct Warehouse<S: Storage, Api: SubstrateAPI> {
    /// Substrate storage implementing the Storage trait
    storage: S,
    substrate_api: Api,
}

impl<S: Storage, Api: SubstrateAPI> Warehouse<S, Api> {
    pub(crate) fn new(storage: S, substrate_api: Api) -> Warehouse<S, Api> {
        Self {
            storage,
            substrate_api,
        }
    }

    pub(crate) fn apply_changes(&self, changeset: ChangeSet) -> Result<()> {
        for (account, changeset) in changeset.into_inner() {
            let key = account.as_slice();
            let mut store_account = match self.storage.get(key) {
                Some(value) => bcs::from_bytes(&value).map_err(Error::msg)?,
                _ => AccountData::default(),
            };

            let (modules, resources) = changeset.into_inner();
            let mut unprocessed_resources = vec![];
            AccountData::apply_changes(&mut store_account.modules, modules)?;
            // process Deposit
            for (tag, res) in resources {
                match res {
                    New(ref data) | Modify(ref data) => {
                        if let Ok(deposit) = bcs::from_bytes::<Deposit>(data) {
                            let (destination, amount) = deposit.into();
                            // make actual transaction using SubstrateApi
                            self.substrate_api
                                .transfer(account, destination, amount)
                                .map_err(|e| Error::msg(e.to_string()))?;
                        } else {
                            unprocessed_resources.push((tag, res));
                        }
                    }
                    _ => {
                        unprocessed_resources.push((tag, res));
                    } // we ignore Delete
                }
            }
            AccountData::apply_changes(&mut store_account.resources, unprocessed_resources)?;

            let account_bytes = bcs::to_bytes(&store_account).map_err(Error::msg)?;
            self.storage.set(key, &account_bytes);
        }

        Ok(())
    }
}

impl<S: Storage, Api: SubstrateAPI> Deref for Warehouse<S, Api> {
    type Target = S;

    fn deref(&self) -> &Self::Target {
        &self.storage
    }
}

impl<S: Storage, Api: SubstrateAPI> ModuleResolver for Warehouse<S, Api> {
    type Error = Error;

    fn get_module(&self, module_id: &ModuleId) -> Result<Option<Vec<u8>>, Self::Error> {
        let raw_account = self.storage.get(module_id.address().as_slice());

        if let Some(raw_account) = raw_account {
            let mut account: AccountData = bcs::from_bytes(&raw_account).map_err(Error::msg)?;

            // Using remove to get the value since the account is already a copy of data from the storage.
            return Ok(account.modules.remove(module_id.name()));
        }

        // Even if the account is not found, we still return Ok(None) - it's not an error for MoveVM.
        Ok(None)
    }
}

impl<S: Storage, Api: SubstrateAPI> ResourceResolver for Warehouse<S, Api> {
    type Error = Error;

    fn get_resource(
        &self,
        address: &AccountAddress,
        tag: &StructTag,
    ) -> Result<Option<Vec<u8>>, Self::Error> {
        if tag == &*DEPOSIT_TEMPLATE {
            let serialized = Value::struct_(Struct::pack([
                Value::address(address.to_owned()),
                Value::u128(self.substrate_api.get_balance(*address)),
            ]))
            .simple_serialize(&MoveTypeLayout::Struct(MoveStructLayout::Runtime(vec![
                Address,
                MoveTypeLayout::U128,
            ])));
            return Ok(serialized);
        }
        let raw_account = self.storage.get(address.as_slice());

        if let Some(raw_account) = raw_account {
            let mut account: AccountData = bcs::from_bytes(&raw_account).map_err(Error::msg)?;

            // Using remove to get the value since the account is already a copy of data from the storage.
            return Ok(account.resources.remove(tag));
        }

        // Even if the account is not found, we still return Ok(None) - it's not an error for MoveVM.
        Ok(None)
    }
}
