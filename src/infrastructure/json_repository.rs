use crate::domain::repositories::{AddressRepository, AddressRepositoryError, RepositoryResult};
use crate::domain::Address;
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io;
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
        self.dir.join(format!("{id}.json"))
    }
}

impl AddressRepository for JsonAddressRepository {
    fn save(&self, addr: Address) -> RepositoryResult<Uuid> {
        let id = addr.id();

        // In case of UUID collision. While the probabilities of
        // collisions are minimal, we remain defensive about this possibility.
        // This will also cover human errors.
        if self.fetch(&id.to_string()).is_ok() {
            return Err(AddressRepositoryError::AlreadyExists(id.to_string()));
        }

        // Prevent address duplication
        let all_addresses = self.fetch_all()?;
        let duplication_check = all_addresses.iter().find(|existing| {
            existing.street == addr.street
                && existing.postal_details.postcode == addr.postal_details.postcode
                && existing.country == addr.country
        });

        if let Some(duplicated_addr) = duplication_check {
            return Err(AddressRepositoryError::AlreadyExists(
                duplicated_addr.id().to_string(),
            ));
        }

        let file = File::create(self.file_path(&id))?;
        serde_json::to_writer(file, &StoredAddress { id, address: addr })?;

        Ok(id)
    }

    fn fetch(&self, id: &str) -> RepositoryResult<Address> {
        let id = Uuid::parse_str(id)?;
        let result = File::open(self.file_path(&id));

        let file = match result {
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                return Err(AddressRepositoryError::NotFound(id.to_string()))
            }
            Err(e) => return Err(AddressRepositoryError::IOFailure(e)),
            Ok(file) => file,
        };

        let stored: StoredAddress = serde_json::from_reader(file)?;

        Ok(stored.address)
    }

    fn fetch_all(&self) -> RepositoryResult<Vec<Address>> {
        let mut addresses = Vec::new();

        for dir_entry in fs::read_dir(&self.dir)? {
            let path = dir_entry?.path();

            if path.extension().is_some_and(|ext| ext == "json") {
                let file = File::open(&path)?;
                let stored: StoredAddress = serde_json::from_reader(file)?;
                addresses.push(stored.address);
            }
        }
        Ok(addresses)
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
        let result = fs::remove_file(self.file_path(&id));

        match result {
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                Err(AddressRepositoryError::NotFound(id.to_string()))
            }
            Err(e) => Err(AddressRepositoryError::IOFailure(e)),
            Ok(_) => Ok(()),
        }
    }
}
