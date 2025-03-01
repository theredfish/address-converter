use serde_json::Value;
use crate::domain::address::Address;
use crate::domain::address_conversion::AddressConversionError;
use crate::domain::address_conversion::AddressConvertible;
use crate::domain::french_address::FrenchAddress;
use crate::domain::iso20022_address::*;
use crate::domain::repositories::*;

pub struct AddressService {
    repository: Box<dyn AddressRepository>
}

pub enum Either<F, I> {
    French(F),
    Iso20022(I),
}

impl<F, I> Either<F, I> {
    pub fn french(self) -> Option<F> {
        match self {
            Either::French(f) => Some(f),
            Either::Iso20022(_) => None
        }
    }

    pub fn iso20022(self) -> Option<I> {
        match self {
            Either::French(_) => None,
            Either::Iso20022(i) => Some(i)
        }
    }
}

pub enum Format {
    French,
    Iso20022
}

impl AddressService {
    pub fn new(repository: Box<dyn AddressRepository>) -> Self {
        Self { repository }
    }

    pub fn convert(&self, input: &str, format: Format, save: bool) -> Result<Either<FrenchAddress, IsoAddress>, String> {
        let address = self.address_from(input)?;

        let converted_address = match format {
            Format::French => {
                let addr = address.to_french().map_err(|e| e.to_string())?;
                Either::French(addr)
            }
            Format::Iso20022 => {
                let addr = address.to_iso20022().map_err(|e| e.to_string())?;
                Either::Iso20022(addr)
            }
        };

        // Only save if the conversion is a success, which ensure more domain
        // validation.
        if save {
            self.repository.save(address).map_err(|e| e.to_string())?;
        }

        Ok(converted_address)
    }

    fn address_from(&self, input: &str) -> Result<Address, String> {
        let value: Value = serde_json::from_str(input).map_err(|e| e.to_string())?;
        let addr = if value.get("iso_address").is_some() {
            let iso: IsoAddress = serde_json::from_value(value).map_err(|e| e.to_string())?;
            Address::from_iso20022(iso)
        } else if value.get("french_address").is_some() {
            let french: FrenchAddress = serde_json::from_value(value).map_err(|e| e.to_string())?;
            Address::from_french(french)
        } else {
            Err(AddressConversionError::InvalidFormat("Invalid input: either iso_address or french_address".to_string()))
        }.map_err(|e| e.to_string())?;

        Ok(addr)
    }
}