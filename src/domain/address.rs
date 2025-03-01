use chrono::{DateTime, Utc};
use strum::EnumString;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq)]
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
    pub street: Option<Street>,
    /// The postal details such as the postcode (or zipcode), town and extra
    /// location information.
    pub postal_details: PostalDetails,
    /// The address country.
    pub country: Country,
}

impl Address {
    pub fn new(
        kind: AddressKind,
        recipient: Recipient,
        delivery_point: Option<DeliveryPoint>,
        street: Option<Street>,
        postal_details: PostalDetails,
        country: Country
    ) -> Self {
        let id = Uuid::new_v4();
        let updated_at = Utc::now();

        Address { 
            id,
            updated_at,
            kind,
            recipient,
            delivery_point,
            street,
            postal_details,
            country 
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum AddressKind {
    Individual,
    Business,
}

#[derive(Clone, Debug, PartialEq)]
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

#[derive(Clone, Debug, PartialEq)]
pub struct DeliveryPoint {
    /// The external delivery point (building, entry, ...).
    pub external: Option<String>,
    /// The internal delivery point (appartment, staircase, ...).
    pub internal: Option<String>,
    /// Complementary delivery point information (P.O 123).
    pub postbox: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Street {
    /// The street number (2, 2BIS, 2D).
    pub number: Option<String>,
    /// The street name ("LE VILLAGE", "RUE DE L'EGLISE").
    pub name: String,
}

#[derive(Clone, Debug, PartialEq)]
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

    mod individual_tests {
        use crate::domain::iso20022_address::{IsoAddress, IsoPostalAddress};
        use super::*;

        #[test]
        fn full_individual_to_french() {
            let address = Address {
                id: Uuid::new_v4(),
                updated_at: Utc::now(),
                kind: AddressKind::Individual,
                recipient: Recipient::Individual { name: "Monsieur Jean DELHOURME".to_string() },
                delivery_point: Some(DeliveryPoint {
                    internal: Some("Chez Mireille COPEAU Appartement 2".to_string()),
                    external: Some("Entrée A Bâtiment Jonquille".to_string()),
                    postbox: Some("CAUDOS".to_string()),
                }),
                street: Some(Street {
                    number: Some("25".to_string()),
                    name: "RUE DE L'EGLISE".to_string(),
                }),
                postal_details: PostalDetails {
                    postcode: "33380".to_string(),
                    town: "MIOS".to_string(),
                    town_location: None,
                },
                country: Country::France,
            };

            let expected = FrenchAddress::Individual(IndividualFrenchAddress {
                name: "Monsieur Jean DELHOURME".to_string(),
                internal_delivery: Some("Chez Mireille COPEAU Appartement 2".to_string()),
                external_delivery: Some("Entrée A Bâtiment Jonquille".to_string()),
                street: Some("25 RUE DE L'EGLISE".to_string()),
                distribution_info: Some("CAUDOS".to_string()),
                postal: "33380 MIOS".to_string(),
                country: "FRANCE".to_string(),
            });

            assert!(address.to_french().is_ok());
            assert_eq!(address.to_french().unwrap(), expected);
        }

        #[test]
        fn full_individual_to_iso20022() {
            let address = Address {
                id: Uuid::new_v4(),
                updated_at: Utc::now(),
                kind: AddressKind::Individual,
                recipient: Recipient::Individual { name: "Monsieur Jean DELHOURME".to_string() },
                delivery_point: Some(DeliveryPoint {
                    internal: Some("Chez Mireille COPEAU Appartement 2".to_string()),
                    external: Some("Entrée A Bâtiment Jonquille".to_string()),
                    postbox: Some("CAUDOS".to_string()),
                }),
                street: Some(Street {
                    number: Some("25".to_string()),
                    name: "RUE DE L'EGLISE".to_string(),
                }),
                postal_details: PostalDetails {
                    postcode: "33380".to_string(),
                    town: "MIOS".to_string(),
                    town_location: None,
                },
                country: Country::France,
            };

            let expected = IsoAddress::IndividualIsoAddress {
                name: "Monsieur Jean DELHOURME".to_string(),
                postal_address: IsoPostalAddress {
                    street_name: Some("RUE DE L'EGLISE".to_string()),
                    building_number: Some("25".to_string()),
                    floor: Some("Entrée A Bâtiment Jonquille".to_string()),
                    room: Some("Chez Mireille COPEAU Appartement 2".to_string()),
                    postbox: Some("CAUDOS".to_string()),
                    department: None,
                    postcode: "33380".to_string(),
                    town_name: "MIOS".to_string(),
                    town_location_name: None,
                    country: "FR".to_string(),
                },
            };

            assert!(address.to_iso20022().is_ok());
            assert_eq!(address.to_iso20022().unwrap(), expected);
        }

        #[test]
        fn minimal_individual_to_french() {
            let address = Address {
                id: Uuid::new_v4(),
                updated_at: Utc::now(),
                kind: AddressKind::Individual,
                recipient: Recipient::Individual { name: "Madame Isabelle RICHARD".to_string() },
                delivery_point: Some(DeliveryPoint {
                    internal: None,
                    external: Some("VILLA BEAU SOLEIL".to_string()),
                    postbox: None,
                }),
                street: Some(Street {
                    number: None,
                    name: "LE VILLAGE".to_string(),
                }),
                postal_details: PostalDetails {
                    postcode: "82500".to_string(),
                    town: "AUTERIVE".to_string(),
                    town_location: None,
                },
                country: Country::France,
            };

            let expected = FrenchAddress::Individual(IndividualFrenchAddress {
                name: "Madame Isabelle RICHARD".to_string(),
                internal_delivery: None,
                external_delivery: Some("VILLA BEAU SOLEIL".to_string()),
                street: Some("LE VILLAGE".to_string()),
                distribution_info: None,
                postal: "82500 AUTERIVE".to_string(),
                country: "FRANCE".to_string(),
            });

            assert!(address.to_french().is_ok());
            assert_eq!(address.to_french().unwrap(), expected);
        }

        #[test]
        fn minimal_individual_to_iso20022() {
            let address = Address {
                id: Uuid::new_v4(),
                updated_at: Utc::now(),
                kind: AddressKind::Individual,
                recipient: Recipient::Individual { name: "Madame Isabelle RICHARD".to_string() },
                delivery_point: Some(DeliveryPoint {
                    internal: None,
                    external: Some("VILLA BEAU SOLEIL".to_string()),
                    postbox: None,
                }),
                street: Some(Street {
                    number: None,
                    name: "LE VILLAGE".to_string(),
                }),
                postal_details: PostalDetails {
                    postcode: "82500".to_string(),
                    town: "AUTERIVE".to_string(),
                    town_location: None,
                },
                country: Country::France,
            };

            let expected = IsoAddress::IndividualIsoAddress {
                name: "Madame Isabelle RICHARD".to_string(),
                postal_address: IsoPostalAddress {
                    street_name: Some("LE VILLAGE".to_string()),
                    building_number: None,
                    floor: Some("VILLA BEAU SOLEIL".to_string()),
                    room: None,
                    postbox: None,
                    department: None,
                    postcode: "82500".to_string(),
                    town_name: "AUTERIVE".to_string(),
                    town_location_name: None,
                    country: "FR".to_string(),
                },
            };

            assert!(address.to_iso20022().is_ok());
            assert_eq!(address.to_iso20022().unwrap(), expected);
        }
    }

    mod business_tests {
        use crate::domain::iso20022_address::{IsoAddress, IsoPostalAddress};

        use super::*;

        #[test]
        fn business_to_french() {
            let address = Address {
                id: Uuid::new_v4(),
                updated_at: Utc::now(),
                kind: AddressKind::Business,
                recipient: Recipient::Business { 
                    company_name: "Société DUPONT".to_string(),
                    contact: Some("Mademoiselle Lucie MARTIN".to_string()),
                },
                delivery_point: Some(DeliveryPoint {
                    internal: None,
                    external: Some("Résidence des Capucins Bâtiment Quater".to_string()),
                    postbox: Some("BP 90432".to_string()),
                }),
                street: Some(Street {
                    number: Some("56".to_string()),
                    name: "RUE EMILE ZOLA".to_string(),
                }),
                postal_details: PostalDetails {
                    postcode: "34092".to_string(),
                    town: "MONTPELLIER CEDEX 5".to_string(),
                    town_location: Some("MONTFERRIER SUR LEZ".to_string()),
                },
                country: Country::France,
            };

            let expected = FrenchAddress::Business(BusinessFrenchAddress {
                business_name: "Société DUPONT".to_string(),
                recipient: Some("Mademoiselle Lucie MARTIN".to_string()),
                external_delivery: Some("Résidence des Capucins Bâtiment Quater".to_string()),
                street: "56 RUE EMILE ZOLA".to_string(),
                distribution_info: Some("BP 90432 MONTFERRIER SUR LEZ".to_string()),
                postal: "34092 MONTPELLIER CEDEX 5".to_string(),
                country: "FRANCE".to_string(),
            });

            assert!(address.to_french().is_ok());
            assert_eq!(address.to_french().unwrap(), expected);
        }

        #[test]
        fn business_to_iso20022() {
            let address = Address {
                id: Uuid::new_v4(),
                updated_at: Utc::now(),
                kind: AddressKind::Business,
                recipient: Recipient::Business { 
                    company_name: "Société DUPONT".to_string(),
                    contact: Some("Mademoiselle Lucie MARTIN".to_string()),
                },
                delivery_point: Some(DeliveryPoint {
                    internal: None,
                    external: Some("Résidence des Capucins Bâtiment Quater".to_string()),
                    postbox: Some("BP 90432".to_string()),
                }),
                street: Some(Street {
                    number: Some("56".to_string()),
                    name: "RUE EMILE ZOLA".to_string(),
                }),
                postal_details: PostalDetails {
                    postcode: "34092".to_string(),
                    town: "MONTPELLIER CEDEX 5".to_string(),
                    town_location: Some("MONTFERRIER SUR LEZ".to_string()),
                },
                country: Country::France,
            };

            let expected = IsoAddress::BusinessIsoAddress {
                company_name: "Société DUPONT".to_string(),
                postal_address: IsoPostalAddress {
                    street_name: Some("RUE EMILE ZOLA".to_string()),
                    building_number: Some("56".to_string()),
                    floor: Some("Résidence des Capucins Bâtiment Quater".to_string()),
                    room: None,
                    postbox: Some("BP 90432".to_string()),
                    department: Some("Mademoiselle Lucie MARTIN".to_string()),
                    postcode: "34092".to_string(),
                    town_name: "MONTPELLIER CEDEX 5".to_string(),
                    town_location_name: Some("MONTFERRIER SUR LEZ".to_string()),
                    country: "FR".to_string(),
                },
            };

            assert!(address.to_iso20022().is_ok());
            assert_eq!(address.to_iso20022().unwrap(), expected);
        }
    }
}