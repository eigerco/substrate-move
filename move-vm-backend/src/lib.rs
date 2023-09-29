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
use move_core_types::identifier::Identifier;
use move_core_types::account_address::AccountAddress;

use move_core_types::language_storage::{CORE_CODE_ADDRESS, ModuleId};
use move_core_types::resolver::{ModuleResolver, ResourceResolver};
use move_core_types::effects::Op::{New, Modify, Delete};
use move_vm_runtime::move_vm::MoveVM;

use move_stdlib::natives::{all_natives, GasParameters};
use move_vm_types::gas::GasMeter;

use crate::storage::Storage;

pub struct Mvm<S>
    where
        S: Storage + ModuleResolver<Error = Error> + ResourceResolver<Error = Error>,
{
    vm: MoveVM,
    storage: S,
}

use move_vm_test_utils::InMemoryStorage;

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


    pub fn publish_module(&self, module: &[u8], address: AccountAddress, gas: &mut impl GasMeter) -> Result<(), Error> {
        // Testing code
        let addr : [u8; 32]  = [0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
            0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0xCA, 0xFE];

        // Uncomment to check that storage is working
        //self.storage.set(&addr, &module.to_vec());

        let mut sess = self.vm.new_session(&self.storage);

        println!("addr content: {:?}", &self.storage.get(&addr));   // Testing code

        sess.publish_module(module.to_vec(), address, gas)?;
        let (changeset, _) = sess.finish()?;

        for module in changeset.modules().into_iter() {
            match module.2 {
                New(data) | Modify(data) => {
                    self.storage.set(module.0.as_slice(), data);
                    println!("New module: {:?}", data);
                }
                Delete => {
                    self.storage.remove(module.0.as_slice());
                    println!("Delete module");
                }
            }
        }

        println!("addr content: {:?}", &self.storage.get(&addr));   // Testing code

        Ok(())
    }

    pub fn publish_module_inmemstorage(&self, module: &[u8], address: AccountAddress, gas: &mut impl GasMeter) -> Result<(), Error> {
        // Testing code
        let addr : [u8; 32]  = [0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
            0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0xCA, 0xFE];

        let moduleId = ModuleId::new(
            AccountAddress::new(addr),
            Identifier::new("Empty").unwrap(),
        );

        let mut test_storage = InMemoryStorage::new();
        let mut sess = self.vm.new_session(&test_storage);

        sess.publish_module(module.to_vec(), address, gas)?;
        let (changeset, _) = sess.finish()?;

        test_storage.apply(changeset).unwrap();

        println!("module content: {:?}", &test_storage.get_module(&moduleId));   // Testing code

        Ok(())
    }
}