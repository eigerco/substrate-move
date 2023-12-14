use alloc::string::String;
use alloc::vec::Vec;
use move_core_types::account_address::AccountAddress;
use move_core_types::identifier::Identifier;
use move_core_types::language_storage::TypeTag;
use move_core_types::vm_status::StatusCode;
use move_vm_backend_common::gas_schedule::INSTRUCTION_COST_TABLE;
use move_vm_test_utils::gas_schedule::GasStatus;

/// Call type used to determine if we are calling script or function inside some module.
#[derive(Debug)]
pub enum Call {
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
pub struct Transaction {
    /// Call type.
    pub call: Call,
    /// Type arguments.
    pub type_args: Vec<TypeTag>,
    /// Arguments of the call.
    pub args: Vec<Vec<u8>>,
}

/// Result of the execution.
#[derive(Debug)]
pub struct VmResult {
    /// Execution status code read from the MoveVM
    pub status_code: StatusCode,
    /// Optional error message.
    pub error_message: Option<String>,
    /// Gas used.
    pub gas_used: u64,
}

impl VmResult {
    /// Create a new VmResult.
    pub fn new(status_code: StatusCode, error_message: Option<String>, gas_used: u64) -> Self {
        Self {
            status_code,
            error_message,
            gas_used,
        }
    }

    /// Check if the execution was successful.
    #[inline]
    pub fn is_ok(&self) -> bool {
        self.status_code == StatusCode::EXECUTED
    }

    /// Check if the execution failed.
    #[inline]
    pub fn is_err(&self) -> bool {
        !self.is_ok()
    }
}

/// Gas is a resource-fuel for executing Move scripts.
#[derive(Debug, Clone, Copy)]
pub enum GasStrategy {
    /// A metered gas with a provided limit.
    ///
    /// If the provided gas is not enough to execute the script or publish the script, then the
    /// MoveVM will return the out-of-gas error message.
    ///
    /// This should be the standard option for the MoveVM.
    Metered(u64),
    /// It allows to run Move operations with an infinite amount of gas.
    ///
    /// This option should be used to estimate the required gas for the given MoveVM operation.
    DryRun,
    /// It allows to run the Move operations with the gas handling disabled.
    ///
    /// This option should be used only for testing and debugging purposes.
    Unmetered,
}

/// Internal gas handler.
pub(crate) struct GasHandler<'a> {
    /// Gas status is an input for the MoveVM which tracks spent gas.
    pub(crate) status: GasStatus<'a>,
    /// Dry run shouldn't make any changes to the MoveVM storage.
    pub(crate) dry_run: bool,
}

impl GasHandler<'_> {
    /// Constructs a new [`GasHandler`].
    pub(crate) fn new(strategy: GasStrategy) -> Self {
        let dry_run = matches!(strategy, GasStrategy::DryRun);

        let status = match strategy {
            GasStrategy::Metered(amount) => GasStatus::new(&INSTRUCTION_COST_TABLE, amount.into()),
            GasStrategy::DryRun => GasStatus::new(&INSTRUCTION_COST_TABLE, u64::MAX.into()),
            GasStrategy::Unmetered => GasStatus::new_unmetered(),
        };

        Self { dry_run, status }
    }
}
