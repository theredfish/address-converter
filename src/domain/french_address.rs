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