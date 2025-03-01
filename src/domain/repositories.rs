use thiserror::Error;
use uuid::Uuid;

use super::address::Address;

#[derive(Error, Debug)]
pub enum AddressRepositoryError {
    #[error("Resource not found: `{0}`")]
    NotFound(String),
    #[error("Resource already exists: `{0}`")]
    AlreadyExists(String),
    #[error("Invalid uuid")]
    InvalidUuid(#[from] uuid::Error),
    #[error("Underlying I/O operation failed")]
    IOFailure(#[from] std::io::Error),
    #[error("Underlying serialization or deserialization operation failed")]
    SerializationFailure(#[from] serde_json::Error)
}

/// Short hand for `Result` type.
pub type RepositoryResult<T> = std::result::Result<T, AddressRepositoryError>;

pub trait AddressRepository {
    fn save(&self, addr: Address) -> RepositoryResult<Uuid>;
    fn fetch(&self, id: &str) -> RepositoryResult<Address>;
    fn fetch_all(&self) -> RepositoryResult<Vec<Address>>;
    fn update(&self, addr: Address) -> RepositoryResult<()>;
    fn delete(&self, id: &str) -> RepositoryResult<()>;
}