#![cfg_attr(not(feature = "std"), no_std)]

#[macro_use]
extern crate alloc;

pub mod storage;

use alloc::sync::Arc;

use anyhow::{anyhow, Error};

use move_binary_format::{
    errors::VMResult,
    CompiledModule,
};

use move_core_types::language_storage::{CORE_CODE_ADDRESS, ModuleId};
use move_core_types::resolver::{ModuleResolver, ResourceResolver};
use move_vm_runtime::move_vm::MoveVM;

use move_stdlib::natives::{all_natives, GasParameters};

use crate::storage::Storage;

pub struct Mvm<S>
    where
        S: Storage + ModuleResolver<Error = Error> + ResourceResolver<Error = Error>,
{
    vm: MoveVM,
    storage: S,
}

impl<S> Mvm<S>
    where
        S: Storage + ModuleResolver<Error = Error> + ResourceResolver<Error = Error>,
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
            storage,
        })
    }

    pub fn load_module(&self, module: &ModuleId) -> Result<Arc<CompiledModule>, Error> {
        let module = self.vm.load_module(module, &self.storage)?;
        Ok(module)
    }
}