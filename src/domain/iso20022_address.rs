#[derive(Debug)]
pub enum IsoAddress {
    IndividualIsoAddress { name: String, iso_postal_address: IsoPostalAddress },
    BusinessIsoAddress { org_id: String, iso_postal_address: IsoPostalAddress },
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