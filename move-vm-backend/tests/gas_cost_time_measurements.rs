use crate::mock::{BalanceMock, StorageMock};
use move_core_types::account_address::AccountAddress;
use move_core_types::language_storage::TypeTag;
use move_vm_backend::types::GasStrategy;
use move_vm_backend::Mvm;
use std::io::Write;

pub mod mock;

const FILENAME: &str = "gas-costs-native-function-calls.txt";
const ITERATIONS: usize = 250;
const BOB: &str = "0x21";

/// Reads bytes from a file for the given path.
/// Can panic if the file doesn't exist.
fn read_bytes(file_path: &str) -> Vec<u8> {
    std::fs::read(file_path)
        .unwrap_or_else(|e| panic!("Can't read {file_path}: {e} - make sure you run move-vm-backend/tests/assets/move-projects/smove-build-all.sh"))
}

/// Reads a precompiled Move scripts from our assets directory.
fn read_script_bytes_from_project(project: &str, script_name: &str) -> Vec<u8> {
    const MOVE_PROJECTS: &str = "tests/assets/move-projects";

    let path =
        format!("{MOVE_PROJECTS}/{project}/build/{project}/bytecode_scripts/{script_name}.mv");

    read_bytes(&path)
}

/// Reads a precompiled Move module from our assets directory.
fn read_module_bytes_from_project(project: &str, module_name: &str) -> Vec<u8> {
    const MOVE_PROJECTS: &str = "tests/assets/move-projects";

    let path =
        format!("{MOVE_PROJECTS}/{project}/build/{project}/bytecode_modules/{module_name}.mv");

    read_bytes(&path)
}

#[test]
fn gas_cost_measurement() {
    let acc_bob = AccountAddress::from_hex_literal(BOB).unwrap();
    let acc_std = AccountAddress::from_hex_literal("0x1").unwrap();

    let fresh_vm = || {
        let store = StorageMock::new();
        let mut balance = BalanceMock::new();
        balance.write_cheque(
            acc_bob.clone(),
            1_000_000,
        );
        let vm = Mvm::new(store, balance).unwrap();

        assert!(vm
            .publish_module_bundle(
                &move_stdlib::move_stdlib_bundle(),
                acc_std.clone(),
                GasStrategy::Unmetered
            )
            .error_message
            .is_none());
        assert!(vm
            .publish_module_bundle(
                &move_stdlib::substrate_stdlib_bundle(),
                acc_std.clone(),
                GasStrategy::Unmetered
            )
            .error_message
            .is_none());
        assert!(vm
            .publish_module(
                &read_module_bytes_from_project("gas-costs", "Module"),
                acc_bob.clone(),
                GasStrategy::Unmetered
            )
            .error_message
            .is_none());
        vm
    };

    let bob = bcs::to_bytes(&acc_bob).unwrap();
    let type_args: Vec<TypeTag> = vec![];
    let params: Vec<&[u8]> = vec![&bob];

    let script_names = [
        "magic_gas_costs_balance_transfer",
        "magic_gas_costs_vector",
        "magic_gas_costs_type_name1",
        "magic_gas_costs_signer",
        "magic_gas_costs_balance_cheque_amount",
        "magic_gas_costs_type_name2",
        "magic_gas_costs_balance_total_amount",
    ];

    let data_files = [
        "costs_balance_transfer.txt",
        "costs_balance_cheque_amount.txt",
        "costs_balance_total_amount.txt",
        // "costs_bcs_to_bytes.txt",
        // "costs_debug_print.txt",
        // "costs_debug_print_stack_trace.txt",
        // "costs_event_write_to_event_store.txt",
        // "costs_hash_sha2_256.txt",
        // "costs_hash_sha3_256.txt",
        "costs_signer_borrow_address.txt",
        "costs_string_check_utf8.txt",
        // "costs_string_is_char_boundary.txt",
        // "costs_string_sub_string.txt",
        // "costs_string_index_of.txt",
        "costs_type_name_get.txt",
        // "costs_vector_empty.txt",
        // "costs_vector_length.txt",
        // "costs_vector_push_back.txt",
        // "costs_vector_borrow.txt",
        // "costs_vector_pop_back.txt",
        // "costs_vector_destroy_empty.txt",
        // "costs_vector_swap.txt",
    ];

    let mut file = std::fs::File::create(FILENAME).unwrap();
    file.write(
        format!(
            "Gas Cost Measurements for Move Native Function Calls
----------------------------------------------------
"
        )
        .as_bytes(),
    )
    .unwrap();

    for n in 0..script_names.len() {
        let script = read_script_bytes_from_project("gas-costs", script_names[n]);
        println!("running {}", script_names[n]);

        for _ in 0..ITERATIONS {
            let vm = fresh_vm();
            std::thread::sleep(std::time::Duration::from_millis(10));
            let type_args1 = type_args.clone();
            let params1 = params.clone();
            std::thread::sleep(std::time::Duration::from_millis(20));
            let result = vm.execute_script(&script, type_args1, params1, GasStrategy::Unmetered);
            if let Some(msg) = result.error_message {
                panic!("MVM ececution error: {msg:?}");
            }
            std::thread::sleep(std::time::Duration::from_millis(20));
        }

        for fname in data_files.iter() {
            let path = std::path::PathBuf::from(fname);
            if path.exists() {
                let data_string = std::fs::read_to_string(&path).unwrap();
                let mut data: Vec<u128> = data_string
                    .split("\n")
                    .map(str::parse::<u128>)
                    .filter(Result::is_ok)
                    .map(Result::unwrap)
                    .collect();
                if data.len() > ITERATIONS {
                    let _ = data.split_off(ITERATIONS);
                }
                let avg = data.iter().sum::<u128>() / (ITERATIONS as u128);
                file.write(format!("{} :: {path:?} -> {avg}\n", script_names[n]).as_bytes())
                    .unwrap();
                force_remove::force_remove_file(path).unwrap();
            }
        }
    }
}
