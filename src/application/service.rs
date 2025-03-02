use thiserror::Error;

use crate::domain::*;
use crate::domain::repositories::{AddressRepositoryError, AddressRepository};

#[derive(Error, Debug)]
pub enum AddressServiceError {
    #[error("Invalid json conversion: {0}")]
    InvalidJson(#[from] serde_json::Error),
    #[error("Address conversion error: {0}")]
    ConversionError(#[from] AddressConversionError),
    #[error("Repository error: {0}")]
    PersistenceError(#[from] AddressRepositoryError),
}

/// Short hand for `Result` type.
pub type ServiceResult<T> = std::result::Result<T, AddressServiceError>;

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
    pub fn convert(&self, input: &str, to_format: Format) -> ServiceResult<Either<FrenchAddress, IsoAddress>> {
        let either_converted_addr = match to_format {
            Format::French => {
                // Build from the ISO20022 input
                let iso: IsoAddress = serde_json::from_str(input)?;
                let iso_addr = ConvertedAddress::from_iso20022(iso)?;
                // Convert to french
                let fr_addr = iso_addr.to_french()?;
                Either::French(fr_addr)
            }
            Format::Iso20022 => {
                // Build from the french input
                let french: FrenchAddress = serde_json::from_str(input)?;
                let fr_addr = ConvertedAddress::from_french(french)?;
                // Convert to ISO20022
                let iso_addr = fr_addr.to_iso20022()?;
                Either::Iso20022(iso_addr)
            }
        };

        Ok(either_converted_addr)
    }

    pub fn save(&self, input: &str, from_format: Format) -> ServiceResult<Uuid> {
        let converted_addr = match from_format {
            Format::French => {
                let french: FrenchAddress = serde_json::from_str(input)?;
                ConvertedAddress::from_french(french)?
            }
            Format::Iso20022 => {
                let iso: IsoAddress = serde_json::from_str(input)?;
                ConvertedAddress::from_iso20022(iso)?
            }
        };

        let address = Address::new(converted_addr);
        let id = self.repository.save(address)?;

        Ok(id)
    }

    pub fn update(&self, id: &str, input: &str, from_format: Format) -> ServiceResult<()> {
        let converted_addr = match from_format {
            Format::French => {
                let french: FrenchAddress = serde_json::from_str(input)?;
                ConvertedAddress::from_french(french)?
            }
            Format::Iso20022 => {
                let iso: IsoAddress = serde_json::from_str(input)?;
                ConvertedAddress::from_iso20022(iso)?
            }
        };

        let mut fetched_addr = self.repository.fetch(id)?;
        fetched_addr.update(converted_addr);

        self.repository.update(fetched_addr)?;

        Ok(())
    }

    pub fn fetch(&self, id: &str) -> ServiceResult<Address> {
        let addr = self.repository.fetch(id)?;

        Ok(addr)
    }

    pub fn fetch_format(&self, id: &str, format: Format) -> ServiceResult<Either<FrenchAddress, IsoAddress>> {
        let addr = self.fetch(id)?;
        let converted = addr.as_converted_address();
        
        match format {
            Format::French => Ok(Either::French(converted.to_french()?)),
            Format::Iso20022 => Ok(Either::Iso20022(converted.to_iso20022()?)),
        }
    }

    pub fn delete(&self, id: &str) -> ServiceResult<()> {
        self.repository.delete(id)?;

        Ok(())
    }
}

#[cfg(test)]
pub mod tests {
    use uuid::Uuid; 

    use crate::application::service::Either;
    use crate::application::service::Format;
    use crate::domain::*;
    use crate::domain::repositories::AddressRepositoryError;
    use crate::infrastructure::InMemoryAddressRepository;
    use super::ServiceResult;
    use super::{AddressService, AddressServiceError};

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
            business_name: "Société DUPONT".to_string(),
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
            "business_name": "Société DUPONT",
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
        assert!(matches!(result, Err(AddressServiceError::InvalidJson(_))), "Result was: {result:#?}");
    }

    #[test]
    fn invalid_french_json_format_missing_required_field() {
        let service = service();
        let input = r#"{
            "name": "Monsieur Jean DELHOURME",
            "street": "25 RUE DE L'EGLISE"
        }"#;
        let result = service.convert(input, Format::Iso20022);
        assert!(matches!(result, Err(AddressServiceError::InvalidJson(_))), "Result was: {result:#?}");
    }

    #[test]
    fn invalid_iso_json_format_missing_required_field() {
        let service = service();
        let input = r#"{
            "name": "Monsieur Jean DELHOURME",
            "postal_address": {
                "street_name": "RUE DE L'EGLISE",
                "building_number": "25"
            }
        }"#;
        let result = service.convert(input, Format::French);
        assert!(matches!(result, Err(AddressServiceError::InvalidJson(_))), "Result was: {result:#?}");
    }

    #[test]
    fn save_individual_french() -> ServiceResult<()> {
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
        
        let id = service.save(input, Format::French)?;
        let fetched = service.repository.fetch(&id.to_string())?;
        assert_eq!(fetched.id(), id);

        Ok(())
    }

    #[test]
    fn save_individual_duplicate() -> ServiceResult<()> {
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

        let minimal_input = r#"{
            "name": "Monsieur Jean DELHOURME",
            "street": "25 RUE DE L'EGLISE",
            "postal": "33380 MIOS",
            "country": "FRANCE"
        }"#;

        // Save
        service.save(input, Format::French)?;

        // Recognize duplicated data
        let result = service.save(minimal_input, Format::French);
        assert!(matches!(result, Err(AddressServiceError::PersistenceError(AddressRepositoryError::AlreadyExists(_)))), "result was: {result:#?}");
        
        Ok(())
    }

    #[test]
    fn save_business_iso() -> ServiceResult<()> {
        let service = service();
        let input = r#"{
            "business_name": "Société DUPONT",
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
        
        let id = service.save(input, Format::Iso20022)?;
        let fetched = service.repository.fetch(&id.to_string())?;
        assert_eq!(fetched.id(), id);

        Ok(())
    }

    #[test]
    fn update_existing_individual() -> ServiceResult<()> {
        let service = service();
        // Create individual address
        let input = r#"{
            "name": "Monsieur Jean DELHOURME",
            "street": "25 RUE DE L'EGLISE",
            "postal": "33380 MIOS",
            "country": "FRANCE"
        }"#;

        let id = service.save(input, Format::French)?;
        let addr = service.fetch(&id.to_string())?;
        
        // Update with new street
        let update_input = r#"{
            "name": "Monsieur Jean DELHOURME",
            "street": "10 AVENUE DES CHAMPS",
            "postal": "33380 MIOS",
            "country": "FRANCE"
        }"#;
        
        service.update(&id.to_string(), update_input, Format::French)?;

        // Verify update
        let updated = service.repository.fetch(&id.to_string())?;
        assert_eq!(updated.id(), id);

        let updated_street = updated.street.clone().unwrap();
        assert_eq!(updated_street.name, "AVENUE DES CHAMPS".to_string());
        assert_eq!(updated_street.number, Some("10".to_string()));
        assert!(updated.updated_at() > addr.updated_at());

        Ok(())
    }

    #[test]
    fn update_non_existent() {
        let service = service();
        let input = r#"{
            "name": "Monsieur Jean DELHOURME",
            "street": "25 RUE DE L'EGLISE",
            "postal": "33380 MIOS",
            "country": "FRANCE"
        }"#;
        let uuid = Uuid::new_v4();
        let result = service.update(&uuid.to_string(), input, Format::French);
        assert!(matches!(result, Err(AddressServiceError::PersistenceError(AddressRepositoryError::NotFound(_)))));
    }

    #[test]
    fn fetch_individual_as_french() -> ServiceResult<()> {
        let service = service();
        let input = r#"{
            "name": "Monsieur Jean DELHOURME",
            "street": "25 RUE DE L'EGLISE",
            "postal": "33380 MIOS",
            "country": "FRANCE"
        }"#;
        let saved = service.save(input, Format::French)?;
        let fetched = service.repository.fetch(&saved.to_string())?;

        assert_eq!(fetched.id().to_string(), saved.to_string());

        Ok(())
    }

    #[test]
    fn fetch_non_existent() {
        let service = service();
        let uuid = Uuid::new_v4();
        let result = service.fetch(&uuid.to_string());
        assert!(matches!(result, Err(AddressServiceError::PersistenceError(AddressRepositoryError::NotFound(_)))));
    }

    #[test]
    fn fetch_all_individuals() -> ServiceResult<()> {
        let service = service();
        let input1 = r#"{
            "name": "Monsieur Jean DELHOURME",
            "street": "25 RUE DE L'EGLISE",
            "postal": "33380 MIOS",
            "country": "FRANCE"
        }"#;
        let input2 = r#"{
            "name": "Madame Isabelle RICHARD",
            "street": "10 LE VILLAGE",
            "postal": "82500 AUTERIVE",
            "country": "FRANCE"
        }"#;

        service.save(input1, Format::French)?;
        service.save(input2, Format::French)?;

        let addresses = service.repository.fetch_all()?;

        // Assert the results. In-memory HashMap doesn't guarantee order.
        assert_eq!(addresses.len(), 2);
        assert!(addresses.iter().any(|a| a.recipient == Recipient::Individual { name: "Monsieur Jean DELHOURME".to_string() }));
        assert!(addresses.iter().any(|a| a.recipient == Recipient::Individual { name: "Madame Isabelle RICHARD".to_string() }));

        Ok(())
    }

    #[test]
    fn delete_business_existing() -> ServiceResult<()> {
        let service = service();
        let input = r#"{
            "business_name": "Société DUPONT",
            "postal_address": {
                "street_name": "RUE EMILE ZOLA",
                "building_number": "56",
                "postcode": "34092",
                "town_name": "MONTPELLIER CEDEX 5",
                "country": "FR"
            }
        }"#;
        let saved = service.save(input, Format::Iso20022)?;
        let fetched = service.fetch(&saved.to_string())?;
        // assert that the resource is well saved
        assert_eq!(fetched.id().to_string(), saved.to_string());

        // assert that the delete op went well
        let result = service.delete(&saved.to_string());
        assert!(result.is_ok());

        // assert that the ressource is deleted
        let fetch_result = service.fetch(&saved.to_string());
        assert!(fetch_result.is_err());

        Ok(())
    }

    #[test]
    fn delete_non_existent() {
        let service = service();
        let uuid = Uuid::new_v4();
        let result = service.delete(&uuid.to_string());
        assert!(matches!(result, Err(AddressServiceError::PersistenceError(AddressRepositoryError::NotFound(_)))));
    }
}