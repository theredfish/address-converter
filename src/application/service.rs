use serde_json::Value;
use std::result::Result;

use crate::domain::address::Address;
use crate::domain::address_conversion::AddressConvertible;
use crate::domain::french_address::FrenchAddress;
use crate::domain::iso20022_address::*;
use crate::domain::repositories::*;

pub struct AddressService {
    pub repository: Box<dyn AddressRepository>
}

#[derive(Debug, PartialEq)]
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

#[derive(Debug, PartialEq)]
pub enum Format {
    French,
    Iso20022
}

impl AddressService {
    pub fn new(repository: Box<dyn AddressRepository>) -> Self {
        Self { repository }
    }

    /// Converts a json raw string input into an internal representation of an
    /// address. The returned address is either a french address of an iso20022.
    /// 
    /// The given input could have been converted back and forth to DTOs. But
    /// for simplicity reason we decided to use the same format representation
    /// as the value objects which allows a straightforward data mapping.
    pub fn convert(&self, input: &str, to_format: Format) -> Result<Either<FrenchAddress, IsoAddress>, String> {
        // Build the `Address` which is our internal domain model capable of
        // conversions.
        let address = self.build_address(input, &to_format)?;

        let converted_address = match to_format {
            Format::French => {
                let addr = address.to_french().map_err(|e| e.to_string())?;
                Either::French(addr)
            }
            Format::Iso20022 => {
                let addr = address.to_iso20022().map_err(|e| e.to_string())?;
                Either::Iso20022(addr)
            }
        };

        Ok(converted_address)
    }

    /// Since we only support two formats for the conversion, we can take a
    /// quick shortcut there by deserializing to the opposite format.
    /// For example if the conversion is into ISO20022, then we can determine
    /// that the source input is a french address. By deserializing the a
    /// FrenchAddress we will also be able to determine if the provided input
    /// is correct.
    fn build_address(&self, input: &str, to_format: &Format) -> Result<Address, String> {
        // Builds the Value object (our DTO here) and check for valid json
        let value: Value = serde_json::from_str(input).map_err(|e| e.to_string())?;

        // Deserialize to the correct value object based on the provided format.
        // If the desired format is ISO20022 then we deserialize the source
        // as a french address, and vice versa
        let addr = match to_format {
            Format::French => {
                let iso: IsoAddress = serde_json::from_value(value).map_err(|e| e.to_string())?;
                Address::from_iso20022(iso)
            },
            Format::Iso20022 => {
                let french: FrenchAddress = serde_json::from_value(value).map_err(|e| e.to_string())?;
                Address::from_french(french)
            }
        }.map_err(|e| e.to_string())?;

        Ok(addr)
    }
}

#[cfg(test)]
pub mod tests {
    use crate::{application::service::{Either, Format}, domain::{french_address::{BusinessFrenchAddress, FrenchAddress, IndividualFrenchAddress}, iso20022_address::{IsoAddress, IsoPostalAddress}}, infrastructure::in_memory_repository::InMemoryAddressRepository};
    use super::AddressService;

    fn service() -> AddressService {
        let repo = InMemoryAddressRepository::new();
        AddressService::new(Box::new(repo))
    }

    #[test]
    fn individual_french_to_iso() {
        let service = service();
        let input = r#"{
            "name": "Monsieur Jean DELHOURME",
            "internal_delivery": "Chez Mireille COPEAU Appartement 2",
            "external_delivery": "Entrée A Bâtiment Jonquille",
            "street": "25 RUE DE L'EGLISE",
            "distribution_info": "CAUDOS",
            "postal": "33380 MIOS",
            "country": "FRANCE"
        }"#;
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
        let result = service.convert(input, Format::Iso20022);
        assert!(result.is_ok(), "result was {result:#?}");
        assert_eq!(result.unwrap(), Either::Iso20022(expected));
    }

    #[test]
    fn individual_iso_to_french() {
        let service = service();
        let input = r#"{
            "name": "Monsieur Jean DELHOURME",
            "postal_address": {
                "street_name": "RUE DE L'EGLISE",
                "building_number": "25",
                "room": "Chez Mireille COPEAU Appartement 2",
                "postbox": "CAUDOS",
                "postcode": "33380",
                "town_name": "MIOS",
                "country": "FR"
            }
        }"#;
        let expected = FrenchAddress::Individual(IndividualFrenchAddress {
            name: "Monsieur Jean DELHOURME".to_string(),
            internal_delivery: Some("Chez Mireille COPEAU Appartement 2".to_string()),
            external_delivery: None,
            street: Some("25 RUE DE L'EGLISE".to_string()),
            distribution_info: Some("CAUDOS".to_string()),
            postal: "33380 MIOS".to_string(),
            country: "FRANCE".to_string(),
        });
        let result = service.convert(input, Format::French);
        assert!(result.is_ok(), "result was {result:#?}");
        assert_eq!(result.unwrap(), Either::French(expected));
    }

    #[test]
    fn business_french_to_iso() {
        let service = service();
        let input = r#"{
            "business_name": "Société DUPONT",
            "recipient": "Mademoiselle Lucie MARTIN",
            "external_delivery": "Résidence des Capucins Bâtiment Quater",
            "street": "56 RUE EMILE ZOLA",
            "distribution_info": "BP 90432 MONTFERRIER SUR LEZ",
            "postal": "34092 MONTPELLIER CEDEX 5",
            "country": "FRANCE"
        }"#;
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
        let result = service.convert(input, Format::Iso20022);
        assert!(result.is_ok(), "result was {result:#?}");
        assert_eq!(result.unwrap(), Either::Iso20022(expected));
    }

    #[test]
    fn business_iso_to_french() {
        let service = service();
        let input = r#"{
            "company_name": "Société DUPONT",
            "postal_address": {
                "street_name": "RUE EMILE ZOLA",
                "building_number": "56",
                "department": "Mademoiselle Lucie MARTIN",
                "postbox": "BP 90432",
                "town_location_name": "MONTFERRIER SUR LEZ",
                "postcode": "34092",
                "town_name": "MONTPELLIER CEDEX 5",
                "country": "FR"
            }
        }"#;
        let expected = FrenchAddress::Business(BusinessFrenchAddress {
            business_name: "Société DUPONT".to_string(),
            recipient: Some("Mademoiselle Lucie MARTIN".to_string()),
            external_delivery: None,
            street: "56 RUE EMILE ZOLA".to_string(),
            distribution_info: Some("BP 90432 MONTFERRIER SUR LEZ".to_string()),
            postal: "34092 MONTPELLIER CEDEX 5".to_string(),
            country: "FRANCE".to_string(),
        });
        let result = service.convert(input, Format::French);
        assert!(result.is_ok(), "result was {result:#?}");
        assert_eq!(result.unwrap(), Either::French(expected));
    }

    #[test]
    fn invalid_raw_french_input() {
        let service = service();
        let input = "Monsieur Jean DELHOURME, 25 RUE DE L'EGLISE, 33380 MIOS, FRANCE";
        let result = service.convert(input, Format::Iso20022);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("expected value"));
    }

    // TODO: Capture Error::InvalidFormat => Add application error + thiserror to map from domain

    // #[test]
    // fn invalid_french_json_format() {
    //     let service = service();
    //     let input = r#"{
    //         "name": "Monsieur Jean DELHOURME",
    //         "street": "25 RUE DE L'EGLISE"
    //     }"#;
    //     let result = service.convert(input, Format::Iso20022);
    //     assert!(result.is_err());
    //     println!("result was {:#?}", result);
    //     assert!(result.unwrap_err().contains("missing field `postal`"));
    // }

    // #[test]
    // fn invalid_iso_json_format() {
    //     let service = service();
    //     let input = r#"{
    //         "name": "Monsieur Jean DELHOURME",
    //         "postal_address": {
    //             "street_name": "RUE DE L'EGLISE",
    //             "building_number": "25"
    //         }
    //     }"#;
    //     let result = service.convert(input, Format::French);
    //     assert!(result.is_err());
    //     assert!(result.unwrap_err().contains("missing field `postcode`"));
    // }
}