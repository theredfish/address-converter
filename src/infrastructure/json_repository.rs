use crate::domain::Address;
use crate::domain::repositories::{AddressRepository, RepositoryResult};
use serde::{Serialize, Deserialize};
use std::fs::{self, File};
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
struct StoredAddress {
    id: Uuid,
    address: Address,
}

pub struct JsonAddressRepository {
    dir: PathBuf,
}

impl JsonAddressRepository {
    pub fn new(dir: impl Into<PathBuf>) -> Self {
        let dir = dir.into();
        fs::create_dir_all(&dir).expect("Failed to create JSON storage directory");
        Self { dir }
    }

    fn file_path(&self, id: &Uuid) -> PathBuf {
        self.dir.join(format!("{}.json", id))
    }
}

impl AddressRepository for JsonAddressRepository {
    fn save(&self, addr: Address) -> RepositoryResult<Uuid> {
        let id = addr.id(); // Use Address's existing id
        let stored = StoredAddress { id, address: addr };
        let file = File::create(self.file_path(&id))?;
        serde_json::to_writer(file, &stored)?;
        
        Ok(id)
    }

    fn fetch(&self, id: &str) -> RepositoryResult<Address> {
        let id = Uuid::parse_str(id)?;
        let file = File::open(self.file_path(&id))?;
        let stored: StoredAddress = serde_json::from_reader(file)?;
        
        Ok(stored.address)
    }

    fn update(&self, addr: Address) -> RepositoryResult<()> {
        let id = addr.id();
        let stored = StoredAddress { id, address: addr };
        let file = File::create(self.file_path(&id))?;
        serde_json::to_writer(file, &stored)?;
        
        Ok(())
    }

    fn delete(&self, id: &str) -> RepositoryResult<()> {
        let id = Uuid::parse_str(id)?;
        fs::remove_file(self.file_path(&id))?;
        
        Ok(())
    }
}