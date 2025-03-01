use uuid::Uuid;

use crate::domain::Address;
use crate::domain::repositories::{AddressRepository, AddressRepositoryError, RepositoryResult};
use std::cell::RefCell;
use std::collections::HashMap;

pub struct InMemoryAddressRepository {
    addresses: RefCell<HashMap<String, Address>>,
}

impl InMemoryAddressRepository {
    pub fn new() -> Self {
        Self {
            addresses: RefCell::new(HashMap::new()),
        }
    }
}

impl Default for InMemoryAddressRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl AddressRepository for InMemoryAddressRepository {
    fn save(&self, addr: Address) -> RepositoryResult<Uuid> {
        let id = addr.id();
        let mut addresses = self.addresses.borrow_mut();
        
        if addresses.get(&id.to_string()).is_some() {
            return Err(AddressRepositoryError::AlreadyExists(id.to_string()));
        }

        addresses.insert(id.to_string(), addr);

        Ok(id)
    }

    fn fetch(&self, id: &str) -> RepositoryResult<Address> {
        let address = self.addresses.borrow().get(id).cloned();

        match address {
            Some(address) => Ok(address),
            None => Err(AddressRepositoryError::NotFound(id.to_string()))
        }
    }

    fn update(&self, addr: Address) -> RepositoryResult<()> {
        let mut addresses = self.addresses.borrow_mut();
        let id = addr.id().to_string();
        
        if addresses.get(&id).is_none() {
            return Err(AddressRepositoryError::NotFound(id));
        }

        addresses.insert(id, addr);

        Ok(())
    }

    fn delete(&self, id: &str) -> RepositoryResult<()> {
        let mut addresses = self.addresses.borrow_mut();
        let id = id.to_string();
        
        if addresses.get(&id).is_none() {
            return Err(AddressRepositoryError::NotFound(id));
        }

        addresses.remove(&id);

        Ok(())
    }
}