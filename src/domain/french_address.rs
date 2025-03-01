use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};

use super::address::{PostalDetails, Street};
use super::address_conversion::AddressConversionError;

/// Regex to capture the optional street number (e.g., 25, 2BIS) and the mandatory
/// street name. Capture group indexes will be conserved.
static STREET_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^(?:(\d+[a-zA-Z]*) )?(.+)$").unwrap());
/// Regex to capture the mandatory postalcode/zipcode and town information.
static POSTAL_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^(\d{5})\s+(.+)$").unwrap());
/// Regex to capture poxbox details. Here we consider that two letter followed
/// by a suite of digits correspond to the postbox details (e.g., PO 1234, BP 123).
static POSTBOX_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^[A-Z]{2}\s+\d+").unwrap());
/// Regex to capture the town location information. There are two groups, the
/// first for the postbox (ignored), the second for the townlocation.
/// (e.g., BP 90432 MONTFERRIER SUR LEZ -> MONTFERRIER SUR LEZ)
static TOWN_LOCATION_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^(?:[A-Z]{2}\s+\d+\s+)?(.+)$").unwrap());

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FrenchAddress {
    /// An individual french address.
    Individual(IndividualFrenchAddress),
    /// A business french address.
    Business(BusinessFrenchAddress)
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
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

#[derive(Debug, PartialEq, Serialize, Deserialize)]
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

pub struct FrenchAddressParser;

impl FrenchAddressParser {
    pub fn parse_street(street: &str) -> Result<Street, AddressConversionError> {
        if street.is_empty() {
            return Err(AddressConversionError::InvalidFormat("Street cannot be empty".to_string()));
        }
        if let Some(caps) = STREET_REGEX.captures(street) {
            let number = caps.get(1).map(|m| m.as_str().to_string());
            let name = caps.get(2).map_or("".to_string(), |m| m.as_str().to_string());
            if name.is_empty() {
                return Err(AddressConversionError::InvalidFormat("Street name cannot be empty".to_string()));
            }
            
            return Ok(Street { number, name });
        }
        
        Err(AddressConversionError::InvalidFormat("Invalid street format".to_string()))
    }

    pub fn parse_postal(postal: &str) -> Result<PostalDetails, AddressConversionError> {
        const POSTAL_ERROR: &str = "Postal information should contain a postcode/zipcode and a town (e.g., '44000 NANTES')";

        if let Some(caps) = POSTAL_REGEX.captures(postal) {
            let postcode = caps.get(1)
                .map(|m| m.as_str().to_string())
                .ok_or(AddressConversionError::InvalidFormat(POSTAL_ERROR.to_string()))?;
            let town = caps.get(2)
                .map(|m| m.as_str().to_string())
                .ok_or(AddressConversionError::InvalidFormat(POSTAL_ERROR.to_string()))?;
            
            Ok(PostalDetails {
                postcode,
                town,
                town_location: None,
            })
        } else {
            Err(AddressConversionError::InvalidFormat(POSTAL_ERROR.to_string()))
        }
    }

    pub fn parse_postbox(distribution_info: &str) -> Result<Option<String>, AddressConversionError> {
        if distribution_info.is_empty() {
            return Err(AddressConversionError::InvalidFormat("Distribution info cannot be empty if provided".to_string()));
        }

        if let Some(caps) = POSTBOX_REGEX.captures(distribution_info) {
            let postbox = caps.get(0).map(|m| m.as_str().to_string());
            Ok(postbox)
        } else {
            Ok(None)
        }
    }

    pub fn parse_town_location(distribution_info: &str) -> Result<Option<String>, AddressConversionError> {
        if distribution_info.is_empty() {
            return Err(AddressConversionError::InvalidFormat("Distribution info cannot be empty if provided".to_string()));
        }

        if let Some(caps) = TOWN_LOCATION_REGEX.captures(distribution_info) {
            let town_location = caps.get(1).map(|m| m.as_str().to_string());
            
            Ok(town_location)
        } else {
            Ok(None)
        }
    }
}
