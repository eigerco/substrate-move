use std::cell::RefCell;
use std::collections::HashMap;
use move_core_types::account_address::AccountAddress;
use move_vm_backend::storage::Storage;
use move_vm_backend::{SubstrateAPI, TransferError};

// Mock storage implementation for testing
#[derive(Clone, Debug)]
pub struct StorageMock {
    pub data: RefCell<HashMap<Vec<u8>, Vec<u8>>>,
}

impl StorageMock {
    pub fn new() -> StorageMock {
        StorageMock {
            data: RefCell::new(Default::default()),
        }
    }
}

impl Default for StorageMock {
    fn default() -> Self {
        StorageMock::new()
    }
}

impl Storage for StorageMock {
    fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        let data = self.data.borrow();
        data.get(key).map(|blob| blob.to_owned())
    }

    fn set(&self, key: &[u8], value: &[u8]) {
        let mut data = self.data.borrow_mut();
        data.insert(key.to_owned(), value.to_owned());
    }

    fn remove(&self, key: &[u8]) {
        let mut data = self.data.borrow_mut();
        data.remove(key);
    }
}

pub struct SubstrateApiMock {
    pub storage: StorageMock
}

impl SubstrateAPI for SubstrateApiMock {
    fn transfer(&self, _from: AccountAddress, to: AccountAddress, amount: u128) -> Result<(), TransferError> {
        Ok(self.storage.set(to.as_slice(), amount.to_be_bytes().as_slice()))
    }

    fn get_balance(&self, of: AccountAddress) -> u128 {
        if let Some(b_data) = self.storage.get(of.as_slice()) {
            u128::from_be_bytes(b_data.try_into().unwrap())
        } else { 0 }
    }
}
