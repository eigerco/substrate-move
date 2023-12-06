#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

mod abi;
pub mod storage;
mod types;
mod warehouse;

use abi::ModuleAbi;
use alloc::vec::Vec;
use anyhow::{anyhow, Error};

use move_binary_format::errors::VMResult;
use move_binary_format::CompiledModule;
use move_core_types::account_address::AccountAddress;
use move_core_types::identifier::Identifier;

use move_core_types::effects::{ChangeSet, Event};
use move_core_types::vm_status::StatusCode;
use move_core_types::{
    language_storage::{ModuleId, TypeTag, CORE_CODE_ADDRESS},
    resolver::{ModuleResolver, ResourceResolver},
};
use move_vm_runtime::move_vm::MoveVM;

use move_stdlib::natives::{all_natives, GasParameters};
use move_vm_backend_common::types::ModuleBundle;
use move_vm_types::gas::GasMeter;

use crate::storage::Storage;
use crate::types::VmResult;
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

/// Call type used to determine if we are calling script or function inside some module.
#[derive(Debug)]
enum Call {
    /// Script
    Script {
        /// Script bytecode.
        code: Vec<u8>,
    },
    /// Function in module with script viability.
    ScriptFunction {
        /// Module address.
        mod_address: AccountAddress,
        /// Module name.
        mod_name: Identifier,
        /// Function name - must be public and marked as `entry` in the module.
        func_name: Identifier,
    },
}

/// Transaction struct used in execute_script call.
#[derive(Debug)]
struct Transaction {
    /// Call type.
    pub call: Call,
    /// Type arguments.
    pub type_args: Vec<TypeTag>,
    /// Arguments of the call.
    pub args: Vec<Vec<u8>>,
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

    /// Get module binary using the address and the name.
    pub fn get_module(
        &self,
        address: AccountAddress,
        name: &str,
    ) -> Result<Option<Vec<u8>>, Error> {
        let ident = Identifier::new(name)?;
        let module_id = ModuleId::new(address, ident);
        self.warehouse.get_module(&module_id)
    }

    /// Get module binary ABI using the address and the name.
    pub fn get_module_abi(
        &self,
        address: AccountAddress,
        name: &str,
    ) -> Result<Option<Vec<u8>>, Error> {
        if let Some(bytecode) = self.get_module(address, name)? {
            return Ok(Some(
                bcs::to_bytes(&ModuleAbi::from(
                    CompiledModule::deserialize(&bytecode).map_err(Error::msg)?,
                ))
                .map_err(Error::msg)?,
            ));
        }

        Ok(None)
    }

    /// Get resource using an address and a tag.
    // TODO: could we use Identifier and AccountAddress here instead as arguments?
    pub fn get_resource(
        &self,
        address: &AccountAddress,
        tag: &[u8],
    ) -> Result<Option<Vec<u8>>, Error> {
        let tag = bcs::from_bytes(tag).map_err(Error::msg)?;
        self.warehouse.get_resource(address, &tag)
    }

    /// Publish module into the storage. Module is published under the given address.
    pub fn publish_module(
        &self,
        module: &[u8],
        address: AccountAddress,
        gas: &mut impl GasMeter,
    ) -> VmResult {
        let mut sess = self.vm.new_session(&self.warehouse);

        let result = sess.publish_module(module.to_vec(), address, gas);

        self.handle_result(
            result.and_then(|_| sess.finish()),
            gas,
        )
    }

    /// Publish a package of modules into the storage under the given address.
    pub fn publish_module_package(
        &self,
        package: &[u8],
        address: AccountAddress,
        gas: &mut impl GasMeter,
    ) -> VmResult {
        let modules = ModuleBundle::try_from(package).map_err(|e| {
            VmResult::new(
                StatusCode::UNKNOWN_MODULE,
                Some(e.to_string()),
                gas.balance_internal().into(),
            )
        });

        let modules = match modules {
            Ok(modules) => modules.into_inner(),
            Err(e) => return e,
        };

        let mut sess = self.vm.new_session(&self.warehouse);

        let result = sess.publish_module_bundle(modules, address, gas);

        self.handle_result(
            result.and_then(|_| sess.finish()),
            gas,
        )
    }

    /// Execute script using the given arguments (args).
    pub fn execute_script(
        &self,
        script: &[u8],
        type_args: Vec<TypeTag>,
        args: Vec<&[u8]>,
        gas: &mut impl GasMeter,
    ) -> VmResult {
        self.execute_script_worker(
            Transaction {
                call: Call::Script {
                    code: script.to_vec(),
                },
                type_args,
                args: args.iter().map(|x| x.to_vec()).collect(),
            },
            gas,
        )
    }

    /// Execute function from module using the given arguments (args).
    pub fn execute_function(
        &self,
        mod_address: AccountAddress,
        mod_name: Identifier,
        func_name: Identifier,
        type_args: Vec<TypeTag>,
        args: Vec<&[u8]>,
        gas: &mut impl GasMeter,
    ) -> VmResult {
        self.execute_script_worker(
            Transaction {
                call: Call::ScriptFunction {
                    mod_address,
                    mod_name,
                    func_name,
                },
                type_args,
                args: args.iter().map(|x| x.to_vec()).collect(),
            },
            gas,
        )
    }

    /// Execute script using the given arguments (args).
    fn execute_script_worker(&self, transaction: Transaction, gas: &mut impl GasMeter) -> VmResult {
        let mut sess = self.vm.new_session(&self.warehouse);
        let result;

        match transaction.call {
            Call::Script { code } => {
                result = sess.execute_script(code, transaction.type_args, transaction.args, gas);
            }
            Call::ScriptFunction {
                mod_address,
                mod_name,
                func_name,
            } => {
                result = sess.execute_entry_function(
                    &ModuleId::new(mod_address, mod_name),
                    &func_name,
                    transaction.type_args,
                    transaction.args,
                    gas,
                );
            }
        }

        self.handle_result(
            result.and_then(|_| sess.finish()),
            gas,
        )
    }

    fn handle_result(
        &self,
        result: VMResult<(ChangeSet, Vec<Event>)>,
        gas: &mut impl GasMeter,
    ) -> VmResult {
        match result {
            Ok((changeset, _)) => {
                let mut result =
                    VmResult::new(StatusCode::EXECUTED, None, gas.balance_internal().into());

                if let Err(e) = self.warehouse.apply_changes(changeset) {
                    result.status_code = StatusCode::STORAGE_ERROR;
                    result.error_message = Some(format!("Storage error: {}", e));
                }

                result
            }
            Err(err) => {
                let (status_code, _, msg, _, _, _, _) = err.all_data();
                VmResult::new(status_code, msg.clone(), gas.balance_internal().into())
            }
        }
    }
}
