use chrono::{DateTime, Utc};
use uuid::Uuid;
use std::fmt;

#[derive(Debug, PartialEq)]
pub struct Address {
    id: Uuid,
    updated_at: DateTime<Utc>,
    kind: AddressKind,
    recipient: Recipient,
    delivery_point: Option<DeliveryPoint>,
    street: Street,
    postal_details: PostalDetails,
    country: String,
}

#[derive(Debug, PartialEq)]
pub enum AddressKind {
    Individual,
    Business,
}

#[derive(Debug, PartialEq)]
pub enum Recipient {
    Individual { name: String },
    Business { company_name: String, contact: Option<String> },
}

impl Recipient {
    fn business_contact(&self) -> Option<&String> {
        match self {
            Recipient::Business { contact, .. } => contact.as_ref(),
            _ => None,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct DeliveryPoint {
    building: Option<String>,
    floor: Option<String>,
    room: Option<String>,
    postbox: Option<String>,
}

#[derive(Debug, PartialEq)]
pub struct Street {
    number: Option<u16>,
    name: String,
}

#[derive(Debug, PartialEq)]
pub struct PostalDetails {
    postcode: String,
    town: String,
    town_location: Option<String>,
}

#[derive(Debug, PartialEq)]
pub enum FrenchAddress {
    Individual(IndividualFrenchAddress),
    Business(BusinessFrenchAddress)
}

#[derive(Debug, PartialEq)]
pub struct IndividualFrenchAddress {
    /// The individual identity
    /// (Civility - title / quality - firstname lastname).
    pub name: String,
    /// Additional information of the internal delivery point
    /// (appartment number, mailbox number, staircase, floor, ...).
    pub internal_delivery: Option<String>,
    /// Additional information of the external delivery point
    /// (Building, residence, entrance, ...).
    pub external_delivery: Option<String>,
    /// Route number and label.
    pub street: Option<String>,
    /// Additional distribution information (hamlet, postal box, ...).
    pub distribution_info: Option<String>,
    /// The postal code and locality destination.
    pub postal: String,
    /// The country name.
    pub country: String
}

#[derive(Debug, PartialEq)]
pub struct BusinessFrenchAddress {
    /// The business name or trade name.
    pub business_name: String,
    /// Identity of the recipient and/or service
    pub recipient: String,
    /// Additional information of the external delivery point
    /// (Building, residence, entrance, ...).
    pub external_delivery: String,
    /// Route number and label.
    pub street: String,
    /// Additional distribution information (BP, Sorting Arrival Department)
    /// and the commune where the company is located if different from the CEDEX
    /// distributor office.
    pub distribution_info: String,
    /// Postal code and destination locality. Or CEDEX code and CEDEX
    /// distributor office.
    pub postal: String,
    /// The country name.
    pub country: String
}

#[derive(Debug)]
pub struct IsoPostalAddress {
    /// <StrtNm>
    pub street_name: Option<String>,
    /// <BldgNb>
    pub building_number: Option<u16>,
    /// <Flr>
    pub floor: Option<String>,
    /// <Room>
    pub room: Option<String>,
    /// <PstBx>
    pub postbox: Option<String>,
    /// <Dept>
    pub department: Option<String>,
    /// <PstCd>
    pub postcode: String,
    /// <TwnNm>
    pub town_name: String,
    /// <TwnLctnNm>
    pub town_location_name: Option<String>,
    /// <Ctry> = "FR"
    pub country: String,
}

#[derive(Debug)]
pub enum IsoAddress {
    IndividualIsoAddress { name: String, pst_addr: IsoPostalAddress },
    BusinessIsoAddress { org_id: String, pst_addr: IsoPostalAddress },
}

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

pub trait AddressConvertible {
    fn to_french(&self) -> Result<FrenchAddress, AddressConversionError>;
    fn to_iso20022(&self) -> Result<IsoAddress, AddressConversionError>;
}

impl AddressConvertible for Address {
    fn to_french(&self) -> Result<FrenchAddress, AddressConversionError> {
        match &self.kind {
            AddressKind::Individual => {
                let name: String = match &self.recipient {
                    Recipient::Individual { name } if !name.is_empty() => name.clone(),
                    _ => return Err(AddressConversionError::MissingField("name".to_string())),
                };

                let internal_delivery = self.delivery_point.as_ref()
                    .map_or_else(|| None, |delivery_point| delivery_point.room.clone());

                let external_delivery = self.delivery_point.as_ref()
                    .map_or_else(|| None, |delivery_point| delivery_point.floor.clone());

                let mut street: String = self.street.name.clone();
                if let Some(street_number) = &self.street.number {
                    street = format!("{street_number} {street}");
                }

                let distribution_info = self.delivery_point.as_ref()
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
                    });

                let postal = format!("{} {}", self.postal_details.postcode, self.postal_details.town);
                
                Ok(FrenchAddress::Individual(IndividualFrenchAddress {
                    name,
                    internal_delivery,
                    external_delivery,
                    street: Some(street),
                    distribution_info,
                    postal,
                    country: self.country.clone()
                }))
            }
            AddressKind::Business => {
                let company_name = match &self.recipient {
                    Recipient::Business { company_name, .. } if !company_name.is_empty() => company_name,
                    _ => return Err(AddressConversionError::MissingField("company_name".to_string())),
                };

                
                if let Some(contact) = self.recipient.business_contact() {
                    todo!()
                }
                if let Some(dp) = &self.delivery_point {
                    let mut dp_line = String::new();
                    if let Some(building) = &dp.building { dp_line.push_str(building); }
                    if let Some(floor) = &dp.floor { if !dp_line.is_empty() { dp_line.push_str(", "); } dp_line.push_str(floor); }
                    if !dp_line.is_empty() { todo!() } // Line 3
                }
                // lines.push(format!(
                //     "{} {}",
                //     self.street.number.as_ref().unwrap_or(&"".to_string()),
                //     self.street.name
                // )); // Line 4
                if let Some(town_loc) = &self.postal_details.town_location {
                    todo!()
                }
                // lines.push(format!("{} {}", self.postal_details.postcode, self.postal_details.town)); // Line 6
                // lines.push(self.country.clone()); // Line 7
                todo!()
                // Ok(FrenchAddress::Business(lines))
            }
        }
    }

    fn to_iso20022(&self) -> Result<IsoAddress, AddressConversionError> {
        let pst_addr = IsoPostalAddress {
            street_name: Some(self.street.name.clone()),
            building_number: self.street.number.clone(),
            floor: self.delivery_point.as_ref().and_then(|dp| dp.floor.clone()),
            room: self.delivery_point.as_ref().and_then(|dp| dp.room.clone()),
            postbox: self.delivery_point.as_ref().and_then(|dp| dp.postbox.clone()),
            department: self.delivery_point.as_ref().and_then(|dp| dp.building.clone()), // Business-only per spec
            postcode: self.postal_details.postcode.clone(),
            town_name: self.postal_details.town.clone(),
            town_location_name: self.postal_details.town_location.clone(),
            country: self.country.clone(),
        };

        match &self.kind {
            AddressKind::Individual => {
                let name = match &self.recipient {
                    Recipient::Individual { name } if !name.is_empty() => name.clone(),
                    _ => return Err(AddressConversionError::MissingField("name".to_string())),
                };
                Ok(IsoAddress::IndividualIsoAddress { name, pst_addr })
            }
            AddressKind::Business => {
                let org_id = match &self.recipient {
                    Recipient::Business { company_name, .. } if !company_name.is_empty() => company_name.clone(),
                    _ => return Err(AddressConversionError::MissingField("company_name".to_string())),
                };
                Ok(IsoAddress::BusinessIsoAddress { org_id, pst_addr })
            }
        }
    }
}

#[cfg(test)]
pub mod tests {
    use crate::domain::address::*;

    #[test]
    fn full_individual_french_address_to_domain() {
        use crate::domain::address::{AddressKind, DeliveryPoint, PostalDetails, Recipient, Street};

        let recipient = Recipient::Individual { name: "Monsieur Jean DELHOURME".to_string() };
        let delivery_point = Some(DeliveryPoint {
            building: None,
            floor: Some("Entrée A Bâtiment Jonquille".to_string()),
            room: Some("Chez Mireille COPEAU Appartement 2".to_string()),
            postbox: Some("CAUDOS".to_string()),
        });
        let street = Street {
            number: Some(25),
            name: "RUE DE L'EGLISE".to_string(),
        };
        let postal_details = PostalDetails {
            postcode: "33380".to_string(),
            town: "MIOS".to_string(),
            town_location: None,
        };
        let address = Address {
            id: Uuid::new_v4(),
            updated_at: Utc::now(),
            kind: AddressKind::Individual,
            recipient,
            delivery_point,
            street,
            postal_details,
            country: "FRANCE".to_string(),
        };

        let expected_french_address = FrenchAddress::Individual(IndividualFrenchAddress {
            name: "Monsieur Jean DELHOURME".to_string(),
            internal_delivery: Some("Chez Mireille COPEAU Appartement 2".to_string()),
            external_delivery: Some("Entrée A Bâtiment Jonquille".to_string()),
            street: Some("25 RUE DE L'EGLISE".to_string()),
            distribution_info: Some("CAUDOS".to_string()),
            postal: "33380 MIOS".to_string(),
            country: "FRANCE".to_string(),
        });

        assert!(address.to_french().is_ok());
        assert_eq!(address.to_french().unwrap(), expected_french_address);
    }
}