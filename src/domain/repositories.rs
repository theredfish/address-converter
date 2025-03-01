use thiserror::Error;
use uuid::Uuid;

use super::address::Address;

#[derive(Error, Debug)]
pub enum AddressRepositoryError {
    #[error("Resource not found: `{0}`")]
    NotFound(String),
    #[error("Resource already exists: `{0}`")]
    AlreadyExists(String),
}

/// Short hand for `Result` type.
pub type RepositoryResult<T> = std::result::Result<T, AddressRepositoryError>;

pub trait AddressRepository {
    fn save(&self, addr: Address) -> RepositoryResult<Uuid>;
    fn fetch(&self, id: &str) -> RepositoryResult<Address>;
    fn update(&self, addr: Address) -> RepositoryResult<()>;
    fn delete(&self, id: &str) -> RepositoryResult<()>;
}