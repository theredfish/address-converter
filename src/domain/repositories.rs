use thiserror::Error;

use super::address::Address;

#[derive(Error, Debug)]
pub enum AddressRepositoryError {
    #[error("Resource not found: `{0}`")]
    NotFound(String),
    #[error("Resource already exists: `{0}`")]
    AlreadyExists(String),
}

/// Short hand for `Result` type.
pub type Result<T> = std::result::Result<T, AddressRepositoryError>;

pub trait AddressRepository {
    fn save(&self, addr: Address) -> Result<()>;
    fn fetch(&self, id: &str) -> Result<Address>;
    fn update(&self, addr: Address) -> Result<()>;
    fn delete(&self, id: &str) -> Result<()>;
}