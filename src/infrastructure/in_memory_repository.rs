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

        // In case of UUID collision. While the probabilities of
        // collisions are minimal, we remain defensive about this possibility.
        // This will also cover human errors.
        if self.fetch(&id.to_string()).is_ok() {
            return Err(AddressRepositoryError::AlreadyExists(id.to_string()));
        }
        
        // Check for address duplicates
        let all_addresses = self.fetch_all()?;
        let duplication_check = all_addresses.iter().find(|existing| {
            existing.street == addr.street &&
            existing.postal_details.postcode == addr.postal_details.postcode &&
            existing.country == addr.country
        });

        if let Some(duplicated_addr) = duplication_check {
            return Err(AddressRepositoryError::AlreadyExists(duplicated_addr.id().to_string()));
        }

        self.addresses.borrow_mut().insert(id.to_string(), addr);

        Ok(id)
    }

    fn fetch(&self, id: &str) -> RepositoryResult<Address> {
        let address = self.addresses.borrow().get(id).cloned();

        match address {
            Some(address) => Ok(address),
            None => Err(AddressRepositoryError::NotFound(id.to_string()))
        }
    }

    fn fetch_all(&self) -> RepositoryResult<Vec<Address>> {
        let addresses = self.addresses.borrow();
        Ok(addresses.values().cloned().collect())
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