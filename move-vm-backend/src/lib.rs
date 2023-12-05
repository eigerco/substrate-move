#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

mod abi;
pub mod deposit;
pub mod storage;
mod warehouse;
use crate::storage::Storage;
use crate::warehouse::Warehouse;
use abi::ModuleAbi;
use alloc::{format, vec::Vec};
use anyhow::{anyhow, Error};
use core::fmt::Display;
use move_binary_format::file_format::StructHandleIndex;
/// re-export for param verification
pub use move_binary_format::file_format::{CompiledScript, SignatureToken};
use move_binary_format::CompiledModule;
use move_core_types::account_address::AccountAddress;
use move_core_types::identifier::Identifier;
use move_core_types::{
    language_storage::{ModuleId, TypeTag, CORE_CODE_ADDRESS},
    resolver::{ModuleResolver, ResourceResolver},
};
use move_stdlib::natives::{all_natives, GasParameters};
use move_vm_backend_common::types::ModuleBundle;
use move_vm_runtime::move_vm::MoveVM;
use move_vm_types::gas::GasMeter;
use move_vm_types::loaded_data::runtime_types::{CachedStructIndex, Type};

/// Represents failures that might occure during native token transaction
#[derive(Debug)]
pub enum TransferError {
    InsuficientBalance,
    NoSessionTokenPresent,
}

impl Display for TransferError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(format!("{:?}", self).as_str())
    }
}

pub trait SubstrateAPI {
    /// Callback signature of method to do the transfer between accounts
    /// # Params
    /// * `from` - 'AccountAddress' is of a sender;
    /// * `to` - 'AccountAddress' is of recepient;
    /// * `amount` - 'u128' is amount to transfer;
    /// # Returns
    /// * Result of success or 'TransferError' variant on error.
    fn transfer(
        &self,
        from: AccountAddress,
        to: AccountAddress,
        amount: u128,
    ) -> Result<(), TransferError>;
    /// Callback to fetch account's balance in Substrate native currency
    /// # Params
    /// `of` - 'AccountAddress' of the account in question;
    /// # Returns 'u128' value of account's balance.
    fn get_balance(&self, of: AccountAddress) -> u128;
}

/// Main MoveVM structure, which is used to represent the virutal machine itself.
pub struct Mvm<S, Api>
where
    S: Storage,
    Api: SubstrateAPI,
{
    // MoveVM instance - from move_vm_runtime crate
    vm: MoveVM,
    // Storage instance
    warehouse: Warehouse<S, Api>,
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

impl<S, Api> Mvm<S, Api>
where
    S: Storage,
    Api: SubstrateAPI,
{
    /// Create a new Move VM with the given storage.
    pub fn new(storage: S, substrate_api: Api) -> Result<Mvm<S, Api>, Error> {
        Self::new_with_config(storage, substrate_api)
    }

    /// Create a new Move VM with the given storage and configuration.
    pub(crate) fn new_with_config(
        storage: S,
        substrate_api: Api,
        // config: VMConfig,
    ) -> Result<Mvm<S, Api>, Error> {
        Ok(Mvm {
            vm: MoveVM::new(all_natives(CORE_CODE_ADDRESS, GasParameters::zeros())).map_err(
                |err| {
                    let (code, _, msg, _, _, _, _) = err.all_data();
                    anyhow!("Error code:{:?}: msg: '{}'", code, msg.unwrap_or_default())
                },
            )?,
            warehouse: Warehouse::new(storage, substrate_api),
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

    /// Publish a package of modules into the storage under the given address.
    pub fn publish_module_package(
        &self,
        package: &[u8],
        address: AccountAddress,
        gas: &mut impl GasMeter,
    ) -> Result<(), Error> {
        let modules = ModuleBundle::try_from(package)?.into_inner();
        let mut sess = self.vm.new_session(&self.warehouse);

        sess.publish_module_bundle(modules, address, gas)
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

    /// Execute script using the given arguments (args).
    pub fn execute_script(
        &self,
        script: &[u8],
        type_args: Vec<TypeTag>,
        args: Vec<&[u8]>,
        gas: &mut impl GasMeter,
    ) -> Result<(), Error> {
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
    ) -> Result<(), Error> {
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
    fn execute_script_worker(
        &self,
        transaction: Transaction,
        gas: &mut impl GasMeter,
    ) -> Result<(), Error> {
        let mut sess = self.vm.new_session(&self.warehouse);

        match transaction.call {
            Call::Script { code } => {
                sess.execute_script(code, transaction.type_args, transaction.args, gas)
                    .map_err(|err| {
                        let (code, _, msg, _, _, _, _) = err.all_data();
                        anyhow!("Error code:{:?}: msg: '{}'", code, msg.unwrap_or_default())
                    })?;
            }
            Call::ScriptFunction {
                mod_address,
                mod_name,
                func_name,
            } => {
                sess.execute_entry_function(
                    &ModuleId::new(mod_address, mod_name),
                    &func_name,
                    transaction.type_args,
                    transaction.args,
                    gas,
                )
                .map_err(|err| {
                    let (code, _, msg, _, _, _, _) = err.all_data();
                    anyhow!("Error code:{:?}: msg: '{}'", code, msg.unwrap_or_default())
                })?;
            }
        }

        let (changeset, _) = sess.finish().map_err(|err| {
            let (code, _, msg, _, _, _, _) = err.all_data();
            anyhow!("Error code:{:?}: msg: '{}'", code, msg.unwrap_or_default())
        })?;

        self.warehouse.apply_changes(changeset)
    }

    pub fn get_struct_members(&self, idx: StructHandleIndex) -> Vec<SignatureToken> {
        let sess = self.vm.new_session(&self.warehouse);
        let Some(s) = sess.get_struct_type(CachedStructIndex(idx.0.into())) else {
            return vec![]; // no struct loaded
        };
        s.fields
            .clone()
            .into_iter()
            .map(Self::type_to_token)
            .collect()
    }

    // WARN: non-reverse for matching purposes only!
    fn type_to_token(type_s: Type) -> SignatureToken {
        match type_s {
            Type::Signer => SignatureToken::Signer,
            Type::Address => SignatureToken::Address,
            Type::Bool => SignatureToken::Bool,
            Type::U8 => SignatureToken::U8,
            Type::U16 => SignatureToken::U16,
            Type::U32 => SignatureToken::U32,
            Type::U64 => SignatureToken::U64,
            Type::U128 => SignatureToken::U128,
            Type::U256 => SignatureToken::U256,
            Type::Struct(csi) => {
                SignatureToken::Struct(StructHandleIndex(csi.0.try_into().unwrap_or_default()))
            }
            Type::StructInstantiation(csi, types) => SignatureToken::StructInstantiation(
                StructHandleIndex(csi.0.try_into().unwrap_or_default()),
                types.into_iter().map(Self::type_to_token).collect(),
            ),
            Type::Vector(v) => Self::type_to_token(*v),
            Type::Reference(r) => Self::type_to_token(*r),
            Type::MutableReference(m) => Self::type_to_token(*m),
            Type::TyParam(p) => SignatureToken::TypeParameter(p),
        }
    }
}
