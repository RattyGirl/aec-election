use crate::aec_parser::ComplexDateRangeEnum::{End, SingleDate, StartEnd};
use crate::IgnoreNS;
use minidom::Element;

pub struct ElectionEventMessage {
    //EventIdentifier
    pub(crate) event_identifier: EventIdentifierStructure,
    //ManagingAuthority
    managing_authority: Option<ManagingAuthorityStructure>,
    //Election
    pub(crate) elections: Vec<ElectionStructure>,
}

impl From<&Element> for ElectionEventMessage {
    fn from(value: &Element) -> Self {
        let event_identifier = value.get_child_ignore_ns("EventIdentifier").unwrap();
        let managing_authority = value.get_child_ignore_ns("ManagingAuthority");
        let elections: Vec<&Element> = value.get_children_ignore_ns("Election");
        Self {
            event_identifier: event_identifier.into(),
            managing_authority: managing_authority.map(ManagingAuthorityStructure::from),
            elections: elections.into_iter().map(ElectionStructure::from).collect(),
        }
    }
}

pub struct EventIdentifierStructure {
    //@Id
    pub(crate) id: Option<String>,
    //EventName
    pub(crate) event_name: Option<String>,
}

impl From<&Element> for EventIdentifierStructure {
    fn from(value: &Element) -> Self {
        let event_name: Option<String> = value
            .get_child_ignore_ns("EventName").map(|x| x.text());
        Self {
            id: value.attr("Id").map(|x| x.to_string()),
            event_name,
        }
    }
}

pub struct ManagingAuthorityStructure {
    //AuthorityIdentifier
    authority_identifier: AuthorityIdentifierStructure,
    //AuthorityAddress
    authority_address: AuthorityAddressStructure,
}

impl From<&Element> for ManagingAuthorityStructure {
    fn from(value: &Element) -> Self {
        let authority_identifier: AuthorityIdentifierStructure = value
            .get_child_ignore_ns("AuthorityIdentifier")
            .unwrap()
            .into();
        let authority_address: AuthorityAddressStructure = value
            .get_child_ignore_ns("AuthorityAddress")
            .unwrap()
            .into();
        Self {
            authority_identifier,
            authority_address,
        }
    }
}

pub struct ElectionStructure {
    //ElectionIdentifier
    pub(crate) election_identifier: ElectionIdentifierStructure,
    //Date
    pub(crate) date: Option<ComplexDateRangeStructure>,
    //Contest
    pub(crate) contests: Vec<ContestStructure>,
}

impl From<&Element> for ElectionStructure {
    fn from(value: &Element) -> Self {
        let date = value.get_child_ignore_ns("Date");
        let contests: Vec<&Element> = value.get_children_ignore_ns("Contest");
        Self {
            election_identifier: value
                .get_child_ignore_ns("ElectionIdentifier")
                .unwrap()
                .into(),
            date: date.map(ComplexDateRangeStructure::from),
            contests: contests
                .into_iter()
                .map(ContestStructure::from)
                .collect(),
        }
    }
}

pub struct AuthorityIdentifierStructure {
    //@Id
    id: String,
    //text
    text: String,
}

impl From<&Element> for AuthorityIdentifierStructure {
    fn from(value: &Element) -> Self {
        Self {
            id: value.attr("Id").unwrap().to_string(),
            text: value.text(),
        }
    }
}

pub struct AuthorityAddressStructure {
    //TODO
}

impl From<&Element> for AuthorityAddressStructure {
    fn from(value: &Element) -> Self {
        Self {}
    }
}

pub struct ElectionIdentifierStructure {
    //@Id
    pub(crate) id: String,
    //ElectionName
    election_name: String,
    //ElectionCategory
    election_category: String,
}

impl From<&Element> for ElectionIdentifierStructure {
    fn from(value: &Element) -> Self {
        Self {
            id: value.attr("Id").unwrap().to_string(),
            election_name: value.get_child_ignore_ns("ElectionName").unwrap().text(),
            election_category: value
                .get_child_ignore_ns("ElectionCategory")
                .unwrap()
                .text(),
        }
    }
}

enum ComplexDateRangeEnum {
    SingleDate(String),
    End(String),
    StartEnd(String, Option<String>),
}

pub struct ComplexDateRangeStructure {
    //@Type
    date_type: String,
    choice: ComplexDateRangeEnum,
}

impl From<&Element> for ComplexDateRangeStructure {
    fn from(value: &Element) -> Self {
        let date_type = value.attr("Type").unwrap().to_string();
        if let Some(date) = value.get_child_ignore_ns("SingleDate") {
            Self {
                date_type,
                choice: SingleDate(date.text()),
            }
        } else if let Some(end_date) = value.get_child_ignore_ns("End") {
            Self {
                date_type,
                choice: End(end_date.text()),
            }
        } else {
            Self {
                date_type,
                choice: StartEnd(
                    value.get_child_ignore_ns("Start").unwrap().text(),
                    value
                        .get_child_ignore_ns("End").map(|x| x.text()),
                ),
            }
        }
    }
}

pub struct ContestStructure {
    //ContestIdentifier
    contest_identifier: ContestIdentifierStructure,
    //Area
    area: Option<AreaStructure>,
    //Position
    position: Option<PositionStructure>,
    //VotingMethod
    voting_method: Vec<VotingMethodType>,
    //MaxVotes
    max_votes: u32,
    //NumberOfPositions
    number_of_positions: u32,
}

impl From<&Element> for ContestStructure {
    fn from(value: &Element) -> Self {
        Self {
            contest_identifier: value
                .get_child_ignore_ns("ContestIdentifier")
                .unwrap()
                .into(),
            area: value
                .get_child_ignore_ns("Area").map(AreaStructure::from),
            position: value
                .get_child_ignore_ns("Position").map(PositionStructure::from),
            voting_method: value
                .get_children_ignore_ns("VotingMethod")
                .into_iter()
                .map(VotingMethodType::from)
                .collect(),
            max_votes: value
                .get_child_ignore_ns("MaxVotes")
                .unwrap()
                .text()
                .parse()
                .unwrap(),
            number_of_positions: value
                .get_child_ignore_ns("NumberOfPositions")
                .unwrap()
                .text()
                .parse()
                .unwrap(),
        }
    }
}

pub struct ContestIdentifierStructure {
    //@Id
    id: String,
    //@ShortCode
    short_code: Option<String>,
    //ContestName
    contest_name: Option<String>,
}

impl From<&Element> for ContestIdentifierStructure {
    fn from(value: &Element) -> Self {
        Self {
            id: value.attr("Id").unwrap().to_string(),
            short_code: value.attr("ShortCode").map(|x| x.to_string()),
            contest_name: value
                .get_child_ignore_ns("ContestName").map(|x| x.text()),
        }
    }
}

#[allow(non_camel_case_types)]
enum VotingMethodType {
    AMS,
    FPP,
    IRV,
    OPV,
    RCV,
    SPV,
    STV,
    NOR,
    cumulative,
    approval,
    block,
    partylist,
    partisan,
    supplementaryvote,
    other,
}

impl From<&Element> for VotingMethodType {
    fn from(value: &Element) -> Self {
        match value.text().as_str() {
            "AMS" => VotingMethodType::AMS,
            "FPP" => VotingMethodType::FPP,
            "IRV" => VotingMethodType::IRV,
            "OPV" => VotingMethodType::OPV,
            "RCV" => VotingMethodType::RCV,
            "SPV" => VotingMethodType::SPV,
            "STV" => VotingMethodType::STV,
            "NOR" => VotingMethodType::NOR,
            "cumulative" => VotingMethodType::cumulative,
            "approval" => VotingMethodType::approval,
            "block" => VotingMethodType::block,
            "partylist" => VotingMethodType::partylist,
            "partisan" => VotingMethodType::partisan,
            "supplementaryvote" => VotingMethodType::supplementaryvote,
            "other" => VotingMethodType::other,
            _ => VotingMethodType::other,
        }
    }
}

pub struct AreaStructure {
    //@Id
    id: Option<String>,
    //@Type
    area_type: Option<String>,
    //text
    text: String,
}

impl From<&Element> for AreaStructure {
    fn from(value: &Element) -> Self {
        Self {
            id: value.attr("Id").map(|x| x.to_string()),
            area_type: value.attr("Type").map(|x| x.to_string()),
            text: value.text(),
        }
    }
}

pub struct PositionStructure {
    //text
    text: String,
}

impl From<&Element> for PositionStructure {
    fn from(value: &Element) -> Self {
        Self { text: value.text() }
    }
}
