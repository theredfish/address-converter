use chrono::{DateTime, Utc};
use strum::EnumString;
use uuid::Uuid;
use std::fmt::{self};
use std::string::ToString;
use std::str::FromStr;

#[derive(Debug, PartialEq)]
pub struct Address {
    /// The unique identifier of the address.
    id: Uuid,
    /// Datetime in UTC of the last modification. Both creation and update dates
    /// are tracked with this field.
    updated_at: DateTime<Utc>,
    /// The type of address. Can be an individual or a business. This
    /// information is used for specific conversion rules depending on the type.
    kind: AddressKind,
    /// Keep track of the receipient details. This information is tracked to
    /// build back addresses where the recipient is required. Some standards
    /// like ISO 20022 store this information outside of the postal address.
    recipient: Recipient,
    /// Extra delivery point information such as the building, the entry or
    /// postbox.
    delivery_point: Option<DeliveryPoint>,
    /// The street address information.
    street: Street,
    /// The postal details such as the postcode (or zipcode), town and extra
    /// location information.
    postal_details: PostalDetails,
    /// The address country.
    country: Country,
}

#[derive(Debug, PartialEq)]
pub enum AddressKind {
    Individual,
    Business,
}

#[derive(Debug, PartialEq)]
pub enum Recipient {
    /// An individual recipient (M. John Doe, Mirabelle Prune)
    Individual { name: String },
    /// The recipient information of a business. Can be composed of both
    /// the business denomination (or brand) and service name or contact
    ///
    /// # Example 1
    /// 
    /// Société DUPONT
    /// Mademoiselle Lucie MARTIN
    /// 
    /// # Example 2
    /// 
    /// DURAND SA
    /// Service achat
    Business { company_name: String, contact: Option<String> },
}

impl Recipient {
    fn denomination(&self) -> Option<String> {
        match self {
            Recipient::Business { contact, .. } => contact.clone(),
            Recipient::Individual { name } => Some(name.clone()),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct DeliveryPoint {
    /// The external delivery point (building, entry, ...).
    external: Option<String>,
    /// The internal delivery point (appartment, staircase, ...).
    internal: Option<String>,
    /// Complementary delivery point information (P.O 123).
    postbox: Option<String>,
}

#[derive(Debug, PartialEq)]
pub struct Street {
    /// The street number (2, 2BIS, 2D).
    number: Option<String>,
    /// The street name ("LE VILLAGE", "RUE DE L’EGLISE").
    name: String,
}

#[derive(Debug, PartialEq)]
pub struct PostalDetails {
    /// The zipcode or postcode of the postal address (56000, K1A 0A6)
    postcode: String,
    /// The town of the postal address.
    town: String,
    /// Complementary town information for distribution.
    town_location: Option<String>,
}

#[derive(Debug, PartialEq)]
pub enum FrenchAddress {
    /// An individual french address.
    Individual(IndividualFrenchAddress),
    /// A business french address.
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
    pub recipient: Option<String>,
    /// Additional information of the external delivery point
    /// (Building, residence, entrance, ...).
    pub external_delivery: Option<String>,
    /// Route number and label.
    pub street: String,
    /// Additional distribution information (BP, Sorting Arrival Department)
    /// and the commune where the company is located if different from the CEDEX
    /// distributor office.
    pub distribution_info: Option<String>,
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
    pub building_number: Option<String>,
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

#[derive(Clone, Debug, strum_macros::Display, EnumString, PartialEq)]
#[strum(serialize_all = "UPPERCASE", ascii_case_insensitive)]
pub enum Country {
    #[strum(serialize = "FRANCE", serialize = "FR")]
    France
}

impl Country {
    pub fn iso_code(&self) -> &'static str {
        match self {
            Country::France => "FR",
        }
    }
}

#[derive(Debug)]
pub enum IsoAddress {
    IndividualIsoAddress { name: String, iso_postal_address: IsoPostalAddress },
    BusinessIsoAddress { org_id: String, iso_postal_address: IsoPostalAddress },
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
                    .map_or_else(|| None, |contact| Some(contact));

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

#[cfg(test)]
pub mod tests {
    use crate::domain::address::*;

    #[test]
    fn full_individual_french_address_to_domain() {
        use crate::domain::address::{AddressKind, DeliveryPoint, PostalDetails, Recipient, Street};

        let recipient = Recipient::Individual { name: "Monsieur Jean DELHOURME".to_string() };
        let delivery_point = Some(DeliveryPoint {
            internal: Some("Chez Mireille COPEAU Appartement 2".to_string()),
            external: Some("Entrée A Bâtiment Jonquille".to_string()),
            postbox: Some("CAUDOS".to_string()),
        });
        let street = Street {
            number: Some("25".to_string()),
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
            country: Country::France,
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