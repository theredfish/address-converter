use std::fmt;

use super::address::*;
use super::french_address::*;
use super::iso20022_address::*;

#[derive(Debug)]
pub enum AddressConversionError {
    MissingField(String),
    InvalidFormat(String),
}

impl fmt::Display for AddressConversionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::MissingField(field) => write!(f, "Missing required field: {}", field),
            Self::InvalidFormat(msg) => write!(f, "Invalid format: {}", msg),
        }
    }
}

impl std::error::Error for AddressConversionError {}


/// A trait representing the conversion rules for any convertible address.
pub trait AddressConvertible {
    /// Convert the address into the french standard NF Z10-011.
    fn to_french(&self) -> Result<FrenchAddress, AddressConversionError>;
    /// Convert the address into the ISO 20022 standard.
    fn to_iso20022(&self) -> Result<IsoAddress, AddressConversionError>;
}

// TODO if time: each value object should be validated based
// on the spec information. Required fields and max length
// should be covered. For now we juste have some examples to demonstrate
// the ability to validate the domain.
impl AddressConvertible for Address {
    fn to_french(&self) -> Result<FrenchAddress, AddressConversionError> {
        let distribution_info = || { 
            self.delivery_point.as_ref()
                .map_or_else(|| None, |delivery_point| {
                    let (town_location, postbox) = (
                        self.postal_details.town_location.clone(),
                        delivery_point.postbox.clone()
                    );

                    match (town_location, postbox) {
                        (None, None) => None,
                        (Some(town_location), None) => Some(town_location),
                        (None, Some(postbox)) => Some(postbox),
                        (Some(town_location), Some(postbox)) => Some(format!("{town_location} {postbox}"))
                    }
                })
        };

        let postal_info = || {
            format!("{} {}", self.postal_details.postcode, self.postal_details.town)
        };

        match &self.kind {
            AddressKind::Individual => {
                let name = match self.recipient.denomination() {
                    Some(name) if !name.is_empty() => name,
                    _ => return Err(AddressConversionError::MissingField("name".to_string()))
                };

                let internal_delivery = self.delivery_point.as_ref()
                    .map_or_else(|| None, |delivery_point| delivery_point.internal.clone());

                let external_delivery = self.delivery_point.as_ref()
                    .map_or_else(|| None, |delivery_point| delivery_point.external.clone());

                let mut street: String = self.street.name.clone();
                if let Some(street_number) = &self.street.number {
                    street = format!("{street_number} {street}");
                }

                let distribution_info = distribution_info();
                let postal = postal_info();
                
                Ok(FrenchAddress::Individual(IndividualFrenchAddress {
                    name,
                    internal_delivery,
                    external_delivery,
                    street: Some(street),
                    distribution_info,
                    postal,
                    country: self.country.to_string()
                }))
            }
            AddressKind::Business => {
                let business_name: String = match &self.recipient {
                    Recipient::Business { company_name, .. } if !company_name.is_empty() => company_name.to_string(),
                    _ => return Err(AddressConversionError::MissingField("company_name".to_string())),
                };

                let recipient = self.recipient.denomination()
                    .map_or_else(|| None, Some);

                let external_delivery = self.delivery_point.as_ref()
                    .map_or_else(|| None, |delivery_point| delivery_point.external.clone());

                let mut street: String = self.street.name.clone();
                if let Some(street_number) = &self.street.number {
                    street = format!("{street_number} {street}");
                };

                let distribution_info = distribution_info();
                let postal = postal_info();

                Ok(FrenchAddress::Business(BusinessFrenchAddress {
                    business_name,
                    recipient,
                    external_delivery,
                    street,
                    distribution_info,
                    postal,
                    country: self.country.to_string()
                }))

            }
        }
    }

    fn to_iso20022(&self) -> Result<IsoAddress, AddressConversionError> {
        let mut iso_postal_address = IsoPostalAddress {
            street_name: Some(self.street.name.clone()),
            building_number: self.street.number.clone(),
            floor: self.delivery_point.as_ref().and_then(|delivery_point| delivery_point.external.clone()),
            room: self.delivery_point.as_ref().and_then(|delivery_point| delivery_point.internal.clone()),
            postbox: self.delivery_point.as_ref().and_then(|delivery_point| delivery_point.postbox.clone()),
            department: None,
            postcode: self.postal_details.postcode.clone(),
            town_name: self.postal_details.town.clone(),
            town_location_name: self.postal_details.town_location.clone(),
            country: self.country.iso_code().to_string(),
        };

        match &self.kind {
            AddressKind::Individual => {
                let name = match &self.recipient {
                    Recipient::Individual { name } if !name.is_empty() => name.clone(),
                    _ => return Err(AddressConversionError::MissingField("name".to_string())),
                };
                Ok(IsoAddress::IndividualIsoAddress { name, iso_postal_address })
            }
            AddressKind::Business => {
                let org_id = match &self.recipient {
                    Recipient::Business { company_name, .. } if !company_name.is_empty() => company_name.clone(),
                    _ => return Err(AddressConversionError::MissingField("company_name".to_string())),
                };
                iso_postal_address.department = self.recipient.denomination();

                Ok(IsoAddress::BusinessIsoAddress { org_id, iso_postal_address })
            }
        }
    }
}
