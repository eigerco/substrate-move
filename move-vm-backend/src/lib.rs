#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub mod storage;
pub mod warehouse;

use alloc::sync::Arc;

use anyhow::{anyhow, Error};

use move_binary_format::CompiledModule;

use move_core_types::account_address::AccountAddress;

use move_core_types::language_storage::{ModuleId, CORE_CODE_ADDRESS};
use move_vm_runtime::move_vm::MoveVM;

use move_stdlib::natives::{all_natives, GasParameters};
use move_vm_types::gas::GasMeter;

use crate::storage::Storage;
use crate::warehouse::Warehouse;

/// Main MoveVM structure, which is used to represent the virutal machine itself.
pub struct Mvm<S>
where
    S: Storage,
{
    // MoveVM instance - from move_vm_runtime crate
    vm: MoveVM,
    // Storage instance
    warehouse: Warehouse<S>,
}

impl<S> Mvm<S>
where
    S: Storage,
{
    /// Create a new Move VM with the given storage.
    pub fn new(storage: S) -> Result<Mvm<S>, Error> {
        Self::new_with_config(storage)
    }

    /// Create a new Move VM with the given storage and configuration.
    pub(crate) fn new_with_config(
        storage: S,
        // config: VMConfig,
    ) -> Result<Mvm<S>, Error> {
        Ok(Mvm {
            vm: MoveVM::new(all_natives(CORE_CODE_ADDRESS, GasParameters::zeros())).map_err(
                |err| {
                    let (code, _, msg, _, _, _, _) = err.all_data();
                    anyhow!("Error code:{:?}: msg: '{}'", code, msg.unwrap_or_default())
                },
            )?,
            warehouse: Warehouse::new(storage),
        })
    }

    /// Load module into cache.
    /// Module must be previously published.
    pub fn load_module(&self, module: &ModuleId) -> Result<Arc<CompiledModule>, Error> {
        let module = self
            .vm
            .load_module(module, &self.warehouse)
            .map_err(|err| {
                let (code, _, msg, _, _, _, _) = err.all_data();
                anyhow!("Error code:{:?}: msg: '{}'", code, msg.unwrap_or_default())
            })?;

        Ok(module)
    }

    /// Publish module into the storage. Module is published under the given address.
    pub fn publish_module(
        &self,
        module: &[u8],
        address: AccountAddress,
        gas: &mut impl GasMeter,
    ) -> Result<(), Error> {
        let mut sess = self.vm.new_session(&self.warehouse);

        sess.publish_module(module.to_vec(), address, gas)
            .map_err(|err| {
                let (code, _, msg, _, _, _, _) = err.all_data();
                anyhow!("Error code:{:?}: msg: '{}'", code, msg.unwrap_or_default())
            })?;

        let (changeset, _) = sess.finish().map_err(|err| {
            let (code, _, msg, _, _, _, _) = err.all_data();
            anyhow!("Error code:{:?}: msg: '{}'", code, msg.unwrap_or_default())
        })?;

        self.warehouse.apply_changes(changeset)
    }
}
