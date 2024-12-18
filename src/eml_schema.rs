use crate::eml_schema::ComplexDateRangeEnum::{End, SingleDate, StartEnd};
use crate::xml_extension::IgnoreNS;
use minidom::Element;
use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};

// EML Parser Error
#[derive(Debug)]
pub enum EMLError {
    /// An error representing an inability to find an element: A inside another element: B
    CannotFindElement(String, String),
}

// AEC Types
#[derive(Clone)]
pub struct WheelChairAccessType(String);

impl TryFrom<&Element> for WheelChairAccessType {
    type Error = EMLError;

    fn try_from(value: &Element) -> Result<Self, Self::Error> {
        Ok(Self(value.text()))
    }
}

impl Serialize for WheelChairAccessType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_newtype_struct("WheelChairAccessType", &self.0)
    }
}

#[derive(Clone)]
pub struct PhysicalLocationStructure {
    lat: String,
    long: String,
    address_details: String,
    premises: String,
    address_line_1: String,
    address_line_2: String,
    suburb: String,
    state: String,
    postcode: String,
}

impl TryFrom<&Element> for PhysicalLocationStructure {
    type Error = EMLError;

    fn try_from(value: &Element) -> Result<Self, Self::Error> {
        let address = match value.get_child_ignore_ns("Address") {
            None => {
                return Err(EMLError::CannotFindElement(
                    "Address".to_string(),
                    String::from(value),
                ))
            }
            Some(add) => add,
        };

        let postal_service_elements = match address.get_child_ignore_ns("PostalServiceElements") {
            None => {
                return Err(EMLError::CannotFindElement(
                    "PostalServiceElements".to_string(),
                    String::from(address),
                ))
            }
            Some(pse) => pse,
        };

        let address_lines = match address.get_child_ignore_ns("AddressLines") {
            None => {
                return Err(EMLError::CannotFindElement(
                    "AddressLines".to_string(),
                    String::from(address),
                ))
            }
            Some(address_lines) => address_lines,
        };

        Ok(Self {
            lat: postal_service_elements
                .get_child_ignore_ns("AddressLatitude")
                .unwrap()
                .text(),
            long: postal_service_elements
                .get_child_ignore_ns("AddressLongitude")
                .unwrap()
                .text(),
            address_details: address
                .attr("AddressDetailsKey")
                .unwrap_or_default()
                .to_string(),
            premises: address_lines
                .get_child_with_type("Premises")
                .map(|x| x.text())
                .unwrap_or_else(|| "".to_string()),
            address_line_1: address_lines
                .get_child_with_type("AddressLine1")
                .map(|x| x.text())
                .unwrap_or_else(|| "".to_string()),
            address_line_2: address_lines
                .get_child_with_type("AddressLine2")
                .map(|x| x.text())
                .unwrap_or_else(|| "".to_string()),
            suburb: address_lines
                .get_child_with_type("Suburb")
                .map(|x| x.text())
                .unwrap_or_else(|| "".to_string()),
            state: address_lines
                .get_child_with_type("State")
                .map(|x| x.text())
                .unwrap_or_else(|| "".to_string()),
            postcode: address_lines
                .get_child_with_type("Postcode")
                .map(|x| x.text())
                .unwrap_or_else(|| "".to_string()),
        })
    }
}
impl Serialize for PhysicalLocationStructure {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("PhysicalLocationStructure", 9)?;
        state.serialize_field("lat", &self.lat)?;
        state.serialize_field("long", &self.long)?;
        state.serialize_field("address_details", &self.address_details)?;

        state.serialize_field("premises", &self.premises)?;
        state.serialize_field("address_line_1", &self.address_line_1)?;
        state.serialize_field("address_line_2", &self.address_line_2)?;

        state.serialize_field("suburb", &self.suburb)?;
        state.serialize_field("state", &self.state)?;
        state.serialize_field("postcode", &self.postcode)?;
        state.end()
    }
}
#[derive(Clone)]
pub enum PollingPlaceLocationEnum {
    Physical(PhysicalLocationStructure),
    Postal(PhysicalLocationStructure),
    Electronic(xs::Token),
    Other(String),
}

impl Serialize for PollingPlaceLocationEnum {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self.clone() {
            PollingPlaceLocationEnum::Physical(l) => {
                serializer.serialize_newtype_struct("Physical", &l)
            }
            PollingPlaceLocationEnum::Postal(l) => {
                serializer.serialize_newtype_struct("Postal", &l)
            }
            PollingPlaceLocationEnum::Electronic(l) => {
                serializer.serialize_newtype_struct("Electronic", &l)
            }
            PollingPlaceLocationEnum::Other(l) => serializer.serialize_newtype_struct("Other", &l),
        }
    }
}
#[derive(Clone)]
pub struct PollingPlaceLocation(PollingPlaceLocationEnum);

impl Serialize for PollingPlaceLocation {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_newtype_struct("PollingPlaceLocation", &self.0)
    }
}
#[derive(Clone)]
pub struct PollingPlaceIdentifierStructure {
    //@Id
    pub(crate) id: xs::Nmtoken,
    //@Name
    name: Option<String>,
    //@ShortCode
    short_code: Option<xs::Nmtoken>,
}

impl TryFrom<&Element> for PollingPlaceIdentifierStructure {
    type Error = EMLError;

    fn try_from(value: &Element) -> Result<Self, Self::Error> {
        Ok(Self {
            id: xs::Nmtoken(value.attr("Id").unwrap().to_string()),
            name: value.attr("Name").map(|x| x.to_string()),
            short_code: value.attr("ShortCode").map(|x| xs::Nmtoken(x.to_string())),
        })
    }
}

// XML Types
pub mod xs {
    use crate::eml_schema::EMLError;
    use serde::{Serialize, Serializer};

    #[derive(Clone)]
    pub struct Nmtoken(pub String);
    impl Serialize for Nmtoken {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            serializer.serialize_newtype_struct("NMTOKEN", &self.0)
        }
    }

    #[derive(Clone)]
    pub struct PositiveInteger(pub u32);

    impl TryFrom<String> for PositiveInteger {
        type Error = EMLError;

        fn try_from(value: String) -> Result<Self, Self::Error> {
            Ok(Self(value.parse().unwrap()))
        }
    }

    impl Serialize for PositiveInteger {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            serializer.serialize_newtype_struct("PositiveInteger", &self.0)
        }
    }
    #[derive(Clone)]
    pub struct Token(pub String);
    impl Serialize for Token {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            serializer.serialize_newtype_struct("Token", &self.0)
        }
    }
}

// EML v5 Simple
#[derive(Clone)]
pub struct ConfirmationReferenceType(pub String);
#[derive(Clone)]
pub struct CountingAlgorithmType;
#[derive(Clone)]
pub struct DateType;
#[derive(Clone)]
pub struct EmailType;
#[derive(Clone)]
pub struct ErrorCodeType;
#[derive(Clone)]
pub struct GenderType;
#[derive(Clone)]
pub struct LanguageType;
#[derive(Clone)]
pub struct MessageTypeType;
#[derive(Clone)]
pub struct SealUsageType;
#[derive(Clone)]
pub struct ShortCodeType(xs::Nmtoken);
impl Serialize for ShortCodeType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_newtype_struct("ShortCodeType", &self.0)
    }
}
#[derive(Clone)]
pub struct TelephoneNumberType;
#[derive(Clone)]
pub struct VotingChannelType;
#[allow(non_camel_case_types)]
#[derive(Clone)]
pub enum VotingMethodType {
    Ams,
    Fpp,
    Irv,
    Opv,
    Rcv,
    Spv,
    Stv,
    Nor,
    cumulative,
    approval,
    block,
    partylist,
    partisan,
    supplementaryvote,
    other,
}
impl TryFrom<&Element> for VotingMethodType {
    type Error = EMLError;

    fn try_from(value: &Element) -> Result<Self, Self::Error> {
        Ok(match value.text().as_str() {
            "AMS" => VotingMethodType::Ams,
            "FPP" => VotingMethodType::Fpp,
            "IRV" => VotingMethodType::Irv,
            "OPV" => VotingMethodType::Opv,
            "RCV" => VotingMethodType::Rcv,
            "SPV" => VotingMethodType::Spv,
            "STV" => VotingMethodType::Stv,
            "NOR" => VotingMethodType::Nor,
            "cumulative" => VotingMethodType::cumulative,
            "approval" => VotingMethodType::approval,
            "block" => VotingMethodType::block,
            "partylist" => VotingMethodType::partylist,
            "partisan" => VotingMethodType::partisan,
            "supplementaryvote" => VotingMethodType::supplementaryvote,
            "other" => VotingMethodType::other,
            _ => VotingMethodType::other,
        })
    }
}
#[derive(Clone)]
pub struct VotingValueType;
#[derive(Clone)]
pub struct WriteInType;
#[derive(Clone)]
pub struct YesNoType;

// EML v5 Complex
#[derive(Clone)]
pub struct AffiliationIdentifierStructure {
    //@Id
    pub id: Option<xs::Nmtoken>,
    //@DisplayOrder
    display_order: Option<xs::PositiveInteger>,
    //@ShortCode
    short_code: Option<ShortCodeType>,
    //@ExpectedConfirmationReference
    expected_confirmation_reference: Option<ConfirmationReferenceType>,
    //RegisteredName
    registered_name: xs::Token,
}
#[derive(Clone)]
pub struct AffiliationStructure {
    //AffiliationIdentifier
    pub affiliation_identifier: AffiliationIdentifierStructure,
    //Type
    affiliation_type: Option<xs::Token>,
    //Description
    description: Option<xs::Token>,
    //Logo
    logo: Vec<LogoStructure>,
}

impl TryFrom<&Element> for AffiliationStructure {
    type Error = EMLError;

    fn try_from(value: &Element) -> Result<Self, Self::Error> {
        Ok(Self {
            affiliation_identifier: value
                .get_child_ignore_ns("AffiliationIdentifier")
                .unwrap()
                .try_into()
                .unwrap(),
            affiliation_type: value
                .get_child_ignore_ns("Type")
                .map(|x| xs::Token(x.text())),
            description: value
                .get_child_ignore_ns("Description")
                .map(|x| xs::Token(x.text())),
            logo: value
                .get_children_ignore_ns("Logo")
                .into_iter()
                .map(LogoStructure::try_from)
                .map(|x| x.unwrap())
                .collect(),
        })
    }
}
impl TryFrom<&Element> for AffiliationIdentifierStructure {
    type Error = EMLError;

    fn try_from(value: &Element) -> Result<Self, Self::Error> {
        Ok(Self {
            id: value.attr("Id").map(|x| xs::Nmtoken(x.to_string())),
            display_order: value
                .attr("Id")
                .map(|x| xs::PositiveInteger(x.parse().unwrap_or(1))),
            short_code: value
                .attr("ShortCode")
                .map(|x| ShortCodeType(xs::Nmtoken(x.to_string()))),
            expected_confirmation_reference: value
                .attr("ExpectedConfirmationReference")
                .map(|x| ConfirmationReferenceType(x.to_string())),
            registered_name: xs::Token(value.get_child_ignore_ns("RegisteredName").unwrap().text()),
        })
    }
}

impl Serialize for AffiliationIdentifierStructure {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("AffiliationIdentifierStructure", 3)?;
        state.serialize_field("id", &self.id)?;
        state.serialize_field("short_code", &self.short_code)?;
        state.serialize_field("name", &self.registered_name)?;
        state.end()
    }
}

#[derive(Clone)]
pub struct AgentIdentifierStructure<PersonNameStructure> {
    //@Id
    id: Option<xs::Nmtoken>,
    //@DisplayOrder
    display_order: Option<xs::PositiveInteger>,
    //AgentName
    agent_name: PersonNameStructure,
}
#[derive(Clone)]
pub struct AgentStructure<OfficialAddressStructure, PersonNameStructure> {
    //@Id
    id: Option<xs::Nmtoken>,
    //@DisplayOrder
    display_order: Option<xs::PositiveInteger>,
    //@Role
    role: Option<xs::Token>,
    //AgentIdentifier
    agent_identifier: AgentIdentifierStructure<PersonNameStructure>,
    //Affiliation
    affiliation: Option<AffiliationStructure>,
    //OfficialAddress
    official_address: Option<OfficialAddressStructure>,
    //Contact
    contact: Option<ContactDetailsStructure>,
}
#[derive(Clone)]
pub struct AreaStructure {
    //@Id
    id: Option<xs::Nmtoken>,
    //@DisplayOrder
    display_order: Option<xs::PositiveInteger>,
    //@Type
    area_type: Option<xs::Token>,
    //text
    text: String,
}
impl TryFrom<&Element> for AreaStructure {
    type Error = EMLError;

    fn try_from(value: &Element) -> Result<Self, Self::Error> {
        Ok(Self {
            id: value.attr("Id").map(|x| xs::Nmtoken(x.to_string())),
            display_order: value
                .attr("DisplayOrder")
                .map(|x| xs::PositiveInteger(x.parse().unwrap_or(1))),
            area_type: value.attr("Type").map(|x| xs::Token(x.to_string())),
            text: value.text(),
        })
    }
}
impl TryFrom<&Element> for PositionStructure {
    type Error = EMLError;

    fn try_from(value: &Element) -> Result<Self, Self::Error> {
        Ok(Self { text: value.text() })
    }
}
#[derive(Clone)]
pub struct AuditInformationStructure {}
#[derive(Clone)]
pub struct AuthorityIdentifierStructure {
    //@Id
    id: String,
    //text
    text: String,
}

impl TryFrom<&Element> for AuthorityIdentifierStructure {
    type Error = EMLError;

    fn try_from(value: &Element) -> Result<Self, Self::Error> {
        Ok(Self {
            id: value.attr("Id").unwrap().to_string(),
            text: value.text(),
        })
    }
}
#[derive(Clone)]
pub struct BallotIdentifierStructure {}
#[derive(Clone)]
pub struct BallotIdentifierRangeStructure {}
#[derive(Clone)]
pub struct BinaryItemStructure {}
#[derive(Clone)]
pub struct CandidateIdentifierStructure {
    //@Id
    id: u32,
    //CandidateName
    candidate_name: Option<String>,
}
#[derive(Clone)]
pub struct CandidateStructure {
    //CandidateIdentifier
    candidate_identifier: CandidateIdentifierStructure,
    //Gender
    gender: Option<String>,
    //Affiliation
    pub affiliation: Option<AffiliationStructure>,
    //Profession
    profession: Option<String>,
}
impl TryFrom<&Element> for CandidateStructure {
    type Error = EMLError;

    fn try_from(value: &Element) -> Result<Self, Self::Error> {
        Ok(Self {
            candidate_identifier: value
                .get_child_ignore_ns("CandidateIdentifier")
                .unwrap()
                .try_into()
                .unwrap(),
            gender: value.get_child_ignore_ns("Gender").map(|x| x.text()),
            affiliation: value
                .get_child_ignore_ns("Affiliation")
                .map(|x| x.try_into().unwrap()),
            profession: value.get_child_ignore_ns("Profession").map(|x| x.text()),
        })
    }
}
impl Serialize for CandidateStructure {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("CandidateStructure", 4)?;
        state.serialize_field("id", &self.candidate_identifier.id)?;
        state.serialize_field("name", &self.candidate_identifier.candidate_name)?;
        state.serialize_field("profession", &self.profession)?;
        state.serialize_field("gender", &self.gender)?;
        state.end()
    }
}
impl TryFrom<&Element> for CandidateIdentifierStructure {
    type Error = EMLError;

    fn try_from(value: &Element) -> Result<Self, Self::Error> {
        Ok(Self {
            id: value.attr("Id").unwrap().parse().unwrap(),
            candidate_name: value.get_child_ignore_ns("CandidateName").map(|x| x.text()),
        })
    }
}
#[derive(Clone)]
pub struct ChannelStructure {}
#[derive(Clone)]
enum ComplexDateRangeEnum {
    SingleDate(String),
    End(String),
    StartEnd(String, Option<String>),
}
#[derive(Clone)]
pub struct ComplexDateRangeStructure {
    //@Type
    date_type: String,
    choice: ComplexDateRangeEnum,
}
impl Serialize for ComplexDateRangeEnum {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match &self {
            SingleDate(x) => {
                let mut state = serializer.serialize_struct("SingleDate", 1)?;
                state.serialize_field("date", x)?;
                state.end()
            }
            End(x) => {
                let mut state = serializer.serialize_struct("End", 1)?;
                state.serialize_field("end_date", x)?;
                state.end()
            }
            StartEnd(x, y) => {
                let mut state = serializer.serialize_struct("StartEnd", 2)?;
                state.serialize_field("start", x)?;
                state.serialize_field("end", y)?;
                state.end()
            }
        }
    }
}

impl TryFrom<&Element> for ComplexDateRangeStructure {
    type Error = EMLError;

    fn try_from(value: &Element) -> Result<Self, Self::Error> {
        let date_type = value.attr("Type").unwrap().to_string();
        if let Some(date) = value.get_child_ignore_ns("SingleDate") {
            Ok(Self {
                date_type,
                choice: SingleDate(date.text()),
            })
        } else if let Some(end_date) = value.get_child_ignore_ns("End") {
            Ok(Self {
                date_type,
                choice: End(end_date.text()),
            })
        } else {
            Ok(Self {
                date_type,
                choice: StartEnd(
                    value.get_child_ignore_ns("Start").unwrap().text(),
                    value.get_child_ignore_ns("End").map(|x| x.text()),
                ),
            })
        }
    }
}

impl Serialize for ComplexDateRangeStructure {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("ComplexDateRangeStructure", 2)?;
        state.serialize_field("date_type", &self.date_type)?;
        state.serialize_field("choice", &self.choice)?;
        state.end()
    }
}

#[derive(Clone)]
pub struct ContactDetailsStructure {}
#[derive(Clone)]
pub struct ContestIdentifierStructure {
    //@Id
    pub(crate) id: String,
    //@ShortCode
    pub(crate) short_code: Option<String>,
    //ContestName
    pub(crate) contest_name: Option<String>,
}
impl TryFrom<&Element> for ContestIdentifierStructure {
    type Error = EMLError;

    fn try_from(value: &Element) -> Result<Self, Self::Error> {
        Ok(Self {
            id: value.attr("Id").unwrap().to_string(),
            short_code: value.attr("ShortCode").map(|x| x.to_string()),
            contest_name: value.get_child_ignore_ns("ContestName").map(|x| x.text()),
        })
    }
}
#[derive(Clone)]
pub struct CountMetricsStructure {}
#[derive(Clone)]
pub struct CountQualifierStructure {}
#[derive(Clone)]
pub struct DocumentIdentifierStructure {}
#[derive(Clone)]
pub struct ElectionGroupStructure {}
#[derive(Clone)]
pub struct ElectionIdentifierStructure {
    //@Id
    pub(crate) id: String,
    //ElectionName
    pub(crate) election_name: Option<String>,
    //ElectionCategory
    pub(crate) election_category: Option<String>,
}

impl TryFrom<&Element> for ElectionIdentifierStructure {
    type Error = EMLError;

    fn try_from(value: &Element) -> Result<Self, Self::Error> {
        Ok(Self {
            id: value.attr("Id").unwrap().to_string(),
            election_name: value.get_child_ignore_ns("ElectionName").map(|x| x.text()),
            election_category: value
                .get_child_ignore_ns("ElectionCategory")
                .map(|x| x.text()),
        })
    }
}
#[derive(Clone)]
pub struct EmailStructure {}
#[derive(Clone)]
pub struct EventIdentifierStructure {
    //@Id
    pub(crate) id: Option<String>,
    //EventName
    pub(crate) event_name: Option<String>,
}

impl TryFrom<&Element> for EventIdentifierStructure {
    type Error = EMLError;

    fn try_from(value: &Element) -> Result<Self, Self::Error> {
        let event_name: Option<String> = value.get_child_ignore_ns("EventName").map(|x| x.text());
        Ok(Self {
            id: value.attr("Id").map(|x| x.to_string()),
            event_name,
        })
    }
}
impl Serialize for EventIdentifierStructure {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("EventIdentifierStructure", 2)?;
        state.serialize_field("name", &self.event_name.clone().unwrap_or_default())?;
        state.serialize_field("id", &self.id.clone().unwrap_or_default())?;
        state.end()
    }
}

#[derive(Clone)]
pub struct EventQualifierStructure {}
#[derive(Clone)]
pub struct IncomingGenericCommunicationStructure {}
#[derive(Clone)]
pub struct InternalGenericCommunicationStructure {}
#[derive(Clone)]
pub struct LogoStructure {}

impl TryFrom<&Element> for LogoStructure {
    type Error = EMLError;

    fn try_from(_value: &Element) -> Result<Self, Self::Error> {
        //TODO
        Ok(Self {})
    }
}

#[derive(Clone)]
pub struct ManagingAuthorityStructure<AuthorityAddressStructure> {
    //AuthorityIdentifier
    authority_identifier: AuthorityIdentifierStructure,
    //AuthorityAddress
    authority_address: AuthorityAddressStructure,
}

impl<AuthorityAddressStructure: Default> From<&Element>
    for ManagingAuthorityStructure<AuthorityAddressStructure>
{
    fn from(value: &Element) -> Self {
        Self {
            authority_identifier: value
                .get_child_ignore_ns("AuthorityIdentifier")
                .unwrap()
                .try_into()
                .unwrap(),
            authority_address: AuthorityAddressStructure::default(),
        }
    }
}

#[derive(Clone)]
pub struct MessageStructure {}
#[derive(Clone)]
pub struct NominatingOfficerStructure {}
#[derive(Clone)]
pub struct OutgoingGenericCommunicationStructure {}
#[derive(Clone)]
pub struct PeriodStructure {}
#[derive(Clone)]
pub struct PollingDistrictStructure {
    //PollingDistrictIdentifier
    polling_district_identifier: PollingDistrictIdentifierStructure,
    //NameDerivation
    name_derivation: String,
    //ProductsIndustry
    product_industry: String,
    //Location
    location: String,
    //Demographic
    demographic: String,
    //Area
    area: xs::PositiveInteger,
    //PollingPlace
    pub(crate) polling_places: Vec<PollingPlaceStructure>,
}

impl TryFrom<&Element> for PollingDistrictStructure {
    type Error = EMLError;

    fn try_from(value: &Element) -> Result<Self, Self::Error> {
        Ok(Self {
            polling_district_identifier: value
                .get_child_ignore_ns("PollingDistrictIdentifier")
                .unwrap()
                .try_into()
                .unwrap(),
            name_derivation: value.get_child_ignore_ns("NameDerivation").unwrap().text(),
            product_industry: value
                .get_child_ignore_ns("ProductsIndustry")
                .unwrap()
                .text(),
            location: value.get_child_ignore_ns("Location").unwrap().text(),
            demographic: value.get_child_ignore_ns("Demographic").unwrap().text(),
            area: value
                .get_child_ignore_ns("Area")
                .unwrap()
                .text()
                .try_into()
                .unwrap(),
            polling_places: value
                .get_child_ignore_ns("PollingPlaces")
                .unwrap()
                .get_children_ignore_ns("PollingPlace")
                .into_iter()
                .map(PollingPlaceStructure::try_from)
                .map(|x| x.unwrap())
                .collect(),
        })
    }
}

impl Serialize for PollingDistrictStructure {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("PollingDistrictStructure", 9)?;
        state.serialize_field("id", &self.polling_district_identifier.id)?;
        state.serialize_field("state", &self.polling_district_identifier.state_identifier)?;
        state.serialize_field("short_code", &self.polling_district_identifier.short_code)?;
        state.serialize_field("name", &self.polling_district_identifier.name)?;
        state.serialize_field("name_derivation", &self.name_derivation)?;
        state.serialize_field("product_industry", &self.product_industry)?;
        state.serialize_field("location", &self.location)?;
        state.serialize_field("demographic", &self.demographic)?;
        state.serialize_field("area", &self.area)?;
        state.end()
    }
}

#[derive(Clone)]
pub struct PollingDistrictIdentifierStructure {
    //@Id
    id: u32,
    //@ShortCode
    short_code: String,
    //Name
    name: String,
    //StateIdentifier
    state_identifier: String,
}

impl TryFrom<&Element> for PollingDistrictIdentifierStructure {
    type Error = EMLError;

    fn try_from(value: &Element) -> Result<Self, Self::Error> {
        Ok(Self {
            id: value.attr("Id").unwrap().parse().unwrap(),
            name: value.get_child_ignore_ns("Name").unwrap().text(),
            short_code: value.attr("ShortCode").unwrap().to_string(),
            state_identifier: value
                .get_child_ignore_ns("StateIdentifier")
                .unwrap()
                .attr("Id")
                .unwrap()
                .to_string(),
        })
    }
}

#[derive(Clone)]
pub struct PollingPlaceStructure {
    //PhysicalLocation/PostalLocation/ElectronicLocation/OtherLocation
    location: PollingPlaceLocation,
    //PollingPlaceIdentifier
    pub(crate) identifier: PollingPlaceIdentifierStructure,
    //WheelchairAccess
    wheelchair: Option<WheelChairAccessType>,
    // pub(crate) district: Option<ObjectId>,
}

impl TryFrom<&Element> for PollingPlaceStructure {
    type Error = EMLError;

    fn try_from(value: &Element) -> Result<Self, Self::Error> {
        let location: PollingPlaceLocationEnum =
            if let Some(location) = value.get_child_ignore_ns("PhysicalLocation") {
                PollingPlaceLocationEnum::Physical(location.try_into().unwrap())
            } else if let Some(location) = value.get_child_ignore_ns("PostalLocation") {
                PollingPlaceLocationEnum::Postal(location.try_into().unwrap())
            } else if let Some(location) = value.get_child_ignore_ns("ElectronicLocation") {
                PollingPlaceLocationEnum::Electronic(xs::Token(location.text()))
            } else if let Some(location) = value.get_child_ignore_ns("OtherLocation") {
                PollingPlaceLocationEnum::Other(location.text())
            } else {
                PollingPlaceLocationEnum::Other("".to_string())
            };

        Ok(Self {
            location: PollingPlaceLocation(location),
            identifier: value
                .get_child_ignore_ns("PollingPlaceIdentifier")
                .unwrap()
                .try_into()
                .unwrap(),
            wheelchair: value
                .get_child_ignore_ns("WheelchairAccess")
                .map(|x| x.try_into().unwrap()),
            // district: None,
        })
    }
}

impl Serialize for PollingPlaceStructure {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("PollingDistrictStructure", 6)?;
        state.serialize_field("location", &self.location)?;
        state.serialize_field("wheelchair", &self.wheelchair)?;
        state.serialize_field("id", &self.identifier.id)?;
        state.serialize_field("name", &self.identifier.name)?;
        state.serialize_field("short_code", &self.identifier.short_code)?;
        // state.serialize_field("district_id", &self.district)?;
        state.end()
    }
}
#[derive(Clone)]
pub struct PositionStructure {
    //text
    pub(crate) text: String,
}
#[derive(Clone)]
pub struct ProcessingUnitStructure {}
#[derive(Clone)]
pub struct ProposalIdentifierStructure {}
#[derive(Clone)]
pub struct ProposalStructure {}
#[derive(Clone)]
pub struct ProposerStructure {}
#[derive(Clone)]
pub struct ProxyStructure {}
#[derive(Clone)]
pub struct ReferendumOptionIdentifierStructure {}
#[derive(Clone)]
pub struct ReportingUnitIdentifierStructure {}
#[derive(Clone)]
pub struct ResponsibleOfficerStructure {}
#[derive(Clone)]
pub struct ResultsReportingStructure {}
#[derive(Clone)]
pub struct ScrutinyRequirementStructure {}
#[derive(Clone)]
pub struct SealStructure {}
#[derive(Clone)]
pub struct SimpleDateRangeStructure {}
#[derive(Clone)]
pub struct TelephoneStructure {}
#[derive(Clone)]
pub struct VoterIdentificationStructure {}
#[derive(Clone)]
pub struct VoterInformationStructure {}
#[derive(Clone)]
pub struct VTokenStructure {}
#[derive(Clone)]
pub struct VTokenQualifiedStructure {}
