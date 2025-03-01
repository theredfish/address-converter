use std::str::FromStr;
use thiserror::Error;

use super::address::*;
use super::french_address::*;
use super::iso20022_address::*;

#[derive(Debug, Error)]
pub enum AddressConversionError {
    #[error("Missing required field `{0}`")]
    MissingField(String),
    #[error("Invalid format: `{0}`")]
    InvalidFormat(String),
}

/// A trait representing the conversion rules for any convertible address.
pub trait AddressConvertible {
    /// Converts a NF Z10-011 french address into a new Address entity.
    fn from_french(address: FrenchAddress) -> Result<Self, AddressConversionError> where Self: Sized;
    /// Converts an ISO 20022 address into a new Address entity.
    fn from_iso20022(address: IsoAddress) -> Result<Self, AddressConversionError> where Self: Sized;
    /// Converts the address into the french standard NF Z10-011.
    fn to_french(&self) -> Result<FrenchAddress, AddressConversionError>;
    /// Converts the address into the ISO 20022 standard.
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

                    match (postbox, town_location) {
                        (None, None) => None,
                        (None, Some(town_location)) => Some(town_location),
                        (Some(postbox), None) => Some(postbox),
                        (Some(postbox), Some(town_location)) => Some(format!("{postbox} {town_location}"))
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

                let street = self.street.as_ref()
                    .map(|street| {
                        match (street.number.clone(), street.name.clone()) {
                            (Some(number), name) => format!("{number} {name}"),
                            (None, name) => name
                        }                    
                    });

                let distribution_info = distribution_info();
                let postal = postal_info();
                
                Ok(FrenchAddress::Individual(IndividualFrenchAddress {
                    name,
                    internal_delivery,
                    external_delivery,
                    street,
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

                // For the moment it has been decided that businesses should have
                // a street line information.
                let street = self.street.as_ref()
                    .map(|street| {
                        match (street.number.clone(), street.name.clone()) {
                            (Some(number), name) => format!("{number} {name}"),
                            (None, name) => name
                        }                    
                })
                .ok_or(AddressConversionError::MissingField("Street information is required for french business addresses".to_string()))?;

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
        let mut iso_address = IsoPostalAddress {
            street_name: self.street.as_ref().map(|street| street.name.clone()),
            building_number: self.street.as_ref().and_then(|street| street.number.clone()),
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
                Ok(IsoAddress::IndividualIsoAddress { name, postal_address: iso_address })
            }
            AddressKind::Business => {
                let org_id = match &self.recipient {
                    Recipient::Business { company_name, .. } if !company_name.is_empty() => company_name.clone(),
                    _ => return Err(AddressConversionError::MissingField("company_name".to_string())),
                };
                iso_address.department = self.recipient.denomination();

                Ok(IsoAddress::BusinessIsoAddress { company_name: org_id, postal_address: iso_address })
            }
        }
    }
    
    fn from_french(address: FrenchAddress) -> Result<Self, AddressConversionError> where Self: Sized {
        match address {
            FrenchAddress::Individual(individual) => {
                let street = match individual.street {
                    Some(street) => Some(FrenchAddressParser::parse_street(&street)?),
                    None => None
                };

                let postal = FrenchAddressParser::parse_postal(&individual.postal)?;

                let individual_delivery = (
                    individual.external_delivery,
                    individual.internal_delivery,
                    individual.distribution_info
                );
                let delivery_point = match individual_delivery {
                    (None, None, None) => None,
                    _ => Some(DeliveryPoint {
                        external: individual_delivery.0,
                        internal: individual_delivery.1,
                        postbox: individual_delivery.2
                    })
                };
                let country = Country::from_str(&individual.country)
                    .map_err(|err| AddressConversionError::InvalidFormat(err.to_string()))?;

                let individual_address = Address::new(
                    AddressKind::Individual,
                    Recipient::Individual { name: individual.name },
                    delivery_point,
                    street,
                    postal,
                    country
                );

                Ok(individual_address)

            },
            FrenchAddress::Business(business) => {
                let street = Some(FrenchAddressParser::parse_street(&business.street)?);
                let mut postal = FrenchAddressParser::parse_postal(&business.postal)?;
                
                let postbox = business.distribution_info.as_ref()
                    .map(|info| FrenchAddressParser::parse_postbox(info))
                    .transpose()?
                    .flatten();
                let town_location = business.distribution_info.as_ref()
                    .map(|info| FrenchAddressParser::parse_town_location(info))
                    .transpose()?
                    .flatten();

                postal.town_location = town_location;

                let address = Address::new(
                    AddressKind::Business,
                    Recipient::Business { 
                        company_name: business.business_name, 
                        contact: business.recipient 
                    },
                    Some(DeliveryPoint {
                        external: business.external_delivery,
                        internal: None,
                        postbox,
                    }),
                    street,
                    postal,
                    Country::France,
                );

                Ok(address)
            }
        }
    }
    
    fn from_iso20022(address: IsoAddress) -> Result<Self, AddressConversionError> where Self: Sized {
        match address {
            IsoAddress::IndividualIsoAddress { name, postal_address: iso_address } => {
                let street_name = match iso_address.street_name {
                    Some(name) if !name.is_empty() => name,
                    _ => return Err(AddressConversionError::MissingField("street_name".to_string()))
                };
                let country = Country::from_str(&iso_address.country)
                    .map_err(|err| AddressConversionError::InvalidFormat(err.to_string()))?;

                let address = Address::new(
                    AddressKind::Individual,
                    Recipient::Individual { name },
                    Some(DeliveryPoint {
                        external: iso_address.floor,
                        internal: iso_address.room,
                        postbox: iso_address.postbox,
                    }),
                    Some(Street {
                        number: iso_address.building_number,
                        name: street_name,
                    }),
                    PostalDetails {
                        postcode: iso_address.postcode,
                        town: iso_address.town_name,
                        town_location: iso_address.town_location_name,
                    },
                    country
                );

                Ok(address)
            }
            IsoAddress::BusinessIsoAddress { company_name, postal_address: iso_address } => {
                let country = Country::from_str(&iso_address.country)
                    .map_err(|err| AddressConversionError::InvalidFormat(err.to_string()))?;

                let address = Address::new(
                    AddressKind::Business,
                    Recipient::Business { 
                        company_name,
                        contact: iso_address.department,
                    },
                    Some(DeliveryPoint {
                        external: iso_address.floor,
                        internal: None,
                        postbox: iso_address.postbox,
                    }),
                    Some(Street {
                        number: iso_address.building_number,
                        name: iso_address.street_name.unwrap_or_default(),
                    }),
                    PostalDetails {
                        postcode: iso_address.postcode,
                        town: iso_address.town_name,
                        town_location: iso_address.town_location_name,
                    },
                    country
                );

                Ok(address)
            }
        }
    }
}
