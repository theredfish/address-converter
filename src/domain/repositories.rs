use std::io::Error;

use super::address::Address;

pub trait AddressRepository {
    fn save(&self, addr: Address) -> Result<(), Error>;
    fn fetch(&self, id: &str) -> Result<Address, Error>;
    fn update(&self, addr: Address) -> Result<(), Error>;
    fn delete(&self, id: &str) -> Result<(), Error>;
}