use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum IsoAddress {
    IndividualIsoAddress {
        name: String,
        postal_address: IsoPostalAddress,
    },
    BusinessIsoAddress {
        business_name: String,
        postal_address: IsoPostalAddress,
    },
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
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
