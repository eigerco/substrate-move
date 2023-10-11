use crate::mock::StorageMock;
use move_vm_backend::Mvm;

use move_core_types::account_address::AccountAddress;
use move_core_types::identifier::Identifier;
use move_core_types::language_storage::ModuleId;

use move_vm_test_utils::gas_schedule::GasStatus;

pub mod mock;

/// Reads bytes from a file for the given path.
/// Can panic if the file doesn't exist.
fn read_bytes(file_path: &str) -> Vec<u8> {
    std::fs::read(file_path).unwrap_or_else(|e| panic!("Can't read {file_path}: {e}"))
}

/// Reads a precompiled Move module from our assets directory.
fn read_module_bytes_from_project(project: &str, module_name: &str) -> Vec<u8> {
    const MOVE_PROJECTS: &str = "tests/assets/move-projects";

    let path =
        format!("{MOVE_PROJECTS}/{project}/build/{project}/bytecode_modules/{module_name}.mv");

    read_bytes(&path)
}

#[test]
fn load_module_not_found_test() {
    let store = StorageMock::new();
    let vm = Mvm::new(store).unwrap();

    let module_id = ModuleId::new(
        AccountAddress::new([0x1; AccountAddress::LENGTH]),
        Identifier::new("TestModule").unwrap(),
    );

    let result = vm.load_module(&module_id);

    assert!(result.is_err());
}

#[test]
#[ignore = "we need to build the move package before with a script before running the test"]
// This test heavily depends on Move.toml files for thes used Move packages.
fn publish_and_load_module_test() {
    let store = StorageMock::new();
    let vm = Mvm::new(store).unwrap();

    let address = AccountAddress::from_hex_literal("0xCAFE").unwrap();
    let module = read_module_bytes_from_project("empty", "Empty");

    let mut gas_status = GasStatus::new_unmetered();

    let result = vm.publish_module(module.as_slice(), address, &mut gas_status);
    assert!(result.is_ok(), "Failed to publish the module");

    let module_id = ModuleId::new(address, Identifier::new("Empty").unwrap());
    assert!(
        vm.load_module(&module_id).is_ok(),
        "Failed to load the module"
    );
}

#[test]
#[ignore = "we need to build the move package before with a script before running the test"]
// This test heavily depends on Move.toml files for thes used Move packages.
fn publish_module_test() {
    let store = StorageMock::new();
    let vm = Mvm::new(store).unwrap();

    let address = AccountAddress::from_hex_literal("0xCAFE").unwrap();
    let module = read_module_bytes_from_project("empty", "Empty");

    let mut gas_status = GasStatus::new_unmetered();

    let result = vm.publish_module(module.as_slice(), address, &mut gas_status);

    assert!(result.is_ok(), "Failed to publish the module");
}

#[allow(non_snake_case)]
#[test]
#[ignore = "we need to build the move package before with a script before running the test"]
// This test heavily depends on Move.toml files for thes used Move packages.
fn publish_module_dependent_on_stdlib_natives() {
    let store = StorageMock::new();
    let vm = Mvm::new(store).unwrap();
    let mut gas_status = GasStatus::new_unmetered();

    let mod_using_stdlib_natives = read_module_bytes_from_project("using_stdlib_natives", "Vector");
    let addr_StdNativesUser = AccountAddress::from_hex_literal("0x2").unwrap();

    // Natives are part of the MoveVM so no need to publish compiled stdlib bytecode modules.
    let result = vm.publish_module(
        &mod_using_stdlib_natives,
        addr_StdNativesUser,
        &mut gas_status,
    );
    assert!(result.is_ok(), "The first module cannot be published");

    let mod_depends_on_using_stdlib_natives =
        read_module_bytes_from_project("depends_on__using_stdlib_natives", "VectorUser");
    let addr_TestingNatives = AccountAddress::from_hex_literal("0x4").unwrap();

    let result = vm.publish_module(
        &mod_depends_on_using_stdlib_natives,
        addr_TestingNatives,
        &mut gas_status,
    );
    assert!(result.is_ok(), "The second module cannot be published");
}

#[allow(non_snake_case)]
#[test]
#[ignore = "we need to build the move package before with a script before running the test"]
// This test heavily depends on Move.toml files for thes used Move packages.
fn publish_module_using_stdlib_full_fails() {
    let store = StorageMock::new();
    let vm = Mvm::new(store).unwrap();
    let mut gas_status = GasStatus::new_unmetered();

    let mod_using_stdlib_natives =
        read_module_bytes_from_project("using_stdlib_full", "StringAndVector");
    let addr_StdNativesUser = AccountAddress::from_hex_literal("0x3").unwrap();

    // In order to publish a module which is using the full stdlib package, we first must publish
    // the stdlib package itself to the MoveVM.
    let result = vm.publish_module(
        &mod_using_stdlib_natives,
        addr_StdNativesUser,
        &mut gas_status,
    );
    assert!(result.is_err(), "The module shouldn't be published");
}
