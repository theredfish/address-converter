use chrono::{DateTime, Utc};
use strum::EnumString;
use uuid::Uuid;

#[derive(Debug, PartialEq)]
pub struct Address {
    /// The unique identifier of the address.
    pub id: Uuid,
    /// Datetime in UTC of the last modification. Both creation and update dates
    /// are tracked with this field.
    pub updated_at: DateTime<Utc>,
    /// The type of address. Can be an individual or a business. This
    /// information is used for specific conversion rules depending on the type.
    pub kind: AddressKind,
    /// Keep track of the receipient details. This information is tracked to
    /// build back addresses where the recipient is required. Some standards
    /// like ISO 20022 store this information outside of the postal address.
    pub recipient: Recipient,
    /// Extra delivery point information such as the building, the entry or
    /// postbox.
    pub delivery_point: Option<DeliveryPoint>,
    /// The street address information.
    pub street: Street,
    /// The postal details such as the postcode (or zipcode), town and extra
    /// location information.
    pub postal_details: PostalDetails,
    /// The address country.
    pub country: Country,
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
    pub fn denomination(&self) -> Option<String> {
        match self {
            Recipient::Business { contact, .. } => contact.clone(),
            Recipient::Individual { name } => Some(name.clone()),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct DeliveryPoint {
    /// The external delivery point (building, entry, ...).
    pub external: Option<String>,
    /// The internal delivery point (appartment, staircase, ...).
    pub internal: Option<String>,
    /// Complementary delivery point information (P.O 123).
    pub postbox: Option<String>,
}

#[derive(Debug, PartialEq)]
pub struct Street {
    /// The street number (2, 2BIS, 2D).
    pub number: Option<String>,
    /// The street name ("LE VILLAGE", "RUE DE L’EGLISE").
    pub name: String,
}

#[derive(Debug, PartialEq)]
pub struct PostalDetails {
    /// The zipcode or postcode of the postal address (56000, K1A 0A6)
    pub postcode: String,
    /// The town of the postal address.
    pub town: String,
    /// Complementary town information for distribution.
    pub town_location: Option<String>,
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

#[cfg(test)]
pub mod tests {
    use crate::domain::address::*;
    use crate::domain::address_conversion::AddressConvertible;
    use crate::domain::french_address::*;
    use std::str::FromStr;

    #[test]
    fn it_should_parse_country() {
        assert_eq!(Country::from_str("france"), Ok(Country::France));
        assert_eq!(Country::from_str("FRANCE"), Ok(Country::France));
        assert_eq!(Country::from_str("fr"), Ok(Country::France));
        assert_eq!(Country::from_str("FR"), Ok(Country::France));
        assert_eq!(Country::France.to_string(), "FRANCE");
        assert_eq!(Country::France.iso_code(), "FR");
    }

    #[test]
    fn full_individual_address_to_french() {
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