use crate::mock::{StorageMock};

use move_vm_backend::Mvm;
use move_vm_backend::storage::MoveStorage;

use move_core_types::account_address::AccountAddress;
use move_core_types::language_storage::ModuleId;
use move_core_types::identifier::Identifier;

pub mod mock;

#[test]
fn load_module_stdlib_test() {
    let store = MoveStorage::new(StorageMock::new());
    let vm = Mvm::new(store).unwrap();

    // or 0x1::ascii is under 0x1 address?
    let addr : [u8; 32]  = [0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
        0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0];

    let module_id = ModuleId::new(
        AccountAddress::new(addr),
        Identifier::new("bcs").unwrap(),
    );

    let result = vm.load_module(&module_id);

    println!("{:?}", result);
    assert!(result.is_ok());
}

#[test]
fn load_module_not_found_test() {
    let store = MoveStorage::new(StorageMock::new());
    let vm = Mvm::new(store).unwrap();

    let module_id = ModuleId::new(
        AccountAddress::new([0x0; AccountAddress::LENGTH]),
        Identifier::new("TestModule").unwrap(),
    );

    let result = vm.load_module(&module_id);

    assert!(result.is_err());
}


    #[test]
fn publish_module_test() {

}