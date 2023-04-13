use minidom::Element;

//Candidate List (230)
pub mod candidate {
    use crate::eml_schema::{
        AffiliationStructure, CandidateStructure, ContestIdentifierStructure,
        ElectionIdentifierStructure, EventIdentifierStructure,
    };
    use crate::xml_extension::IgnoreNS;
    use minidom::Element;

    #[derive(Clone)]
    pub struct CandidateList {
        //EventIdentifier
        pub(crate) event_identifier: EventIdentifierStructure,
        //Election
        pub(crate) elections: Vec<Election>,
    }
    impl From<&Element> for CandidateList {
        fn from(value: &Element) -> Self {
            let event_identifier = value.get_child_ignore_ns("EventIdentifier").unwrap();
            let elections: Vec<&Element> = value.get_children_ignore_ns("Election");
            Self {
                event_identifier: event_identifier.into(),
                elections: elections.into_iter().map(Election::from).collect(),
            }
        }
    }
    #[derive(Clone)]
    pub struct Election {
        //ElectionIdentifier
        pub(crate) election_identifier: ElectionIdentifierStructure,
        //Contest
        pub(crate) contests: Vec<Contest>,
    }
    impl From<&Element> for Election {
        fn from(value: &Element) -> Self {
            let contests: Vec<&Element> = value.get_children_ignore_ns("Contest");
            Self {
                election_identifier: value
                    .get_child_ignore_ns("ElectionIdentifier")
                    .unwrap()
                    .into(),
                contests: contests.into_iter().map(Contest::from).collect(),
            }
        }
    }
    #[derive(Clone)]
    pub struct Contest {
        //ContestIdentifier
        pub(crate) contest_identifier: ContestIdentifierStructure,
        //Candidate
        pub(crate) candidates: Vec<CandidateStructure>,
        pub(crate) affiliations: Vec<AffiliationStructure>,
    }
    impl From<&Element> for Contest {
        fn from(value: &Element) -> Self {
            Self {
                contest_identifier: value
                    .get_child_ignore_ns("ContestIdentifier")
                    .unwrap()
                    .into(),
                candidates: value
                    .get_children_ignore_ns("Candidate")
                    .into_iter()
                    .map(CandidateStructure::from)
                    .collect(),
                affiliations: value
                    .get_children_ignore_ns("Affiliation")
                    .into_iter()
                    .map(AffiliationStructure::from)
                    .collect(),
            }
        }
    }
}

//Election Event (110)
pub mod event {
    use crate::aec_parser::AuthorityAddressStructure;
    use crate::eml_schema::{
        AreaStructure, ComplexDateRangeStructure, ContestIdentifierStructure,
        ElectionIdentifierStructure, EventIdentifierStructure, ManagingAuthorityStructure,
        PositionStructure, VotingMethodType,
    };
    use crate::xml_extension::IgnoreNS;
    use minidom::Element;
    use serde::ser::SerializeStruct;
    use serde::{Serialize, Serializer};

    #[derive(Clone)]
    pub struct ElectionEvent {
        //EventIdentifier
        pub(crate) event_identifier: EventIdentifierStructure,
        //ManagingAuthority
        managing_authority: Option<ManagingAuthorityStructure<AuthorityAddressStructure>>,
        //Election
        pub(crate) elections: Vec<Election>,
    }
    impl From<&Element> for ElectionEvent {
        fn from(value: &Element) -> Self {
            let event_identifier = value.get_child_ignore_ns("EventIdentifier").unwrap();
            let managing_authority = value.get_child_ignore_ns("ManagingAuthority");
            let elections: Vec<&Element> = value.get_children_ignore_ns("Election");
            Self {
                event_identifier: event_identifier.into(),
                managing_authority: managing_authority.map(|x| x.into()),
                elections: elections.into_iter().map(Election::from).collect(),
            }
        }
    }
    #[derive(Clone)]
    pub struct Election {
        //ElectionIdentifier
        pub(crate) election_identifier: ElectionIdentifierStructure,
        //Date
        pub(crate) date: Option<ComplexDateRangeStructure>,
        //Contest
        pub(crate) contests: Vec<Contest>,
    }
    impl From<&Element> for Election {
        fn from(value: &Element) -> Self {
            let date = value.get_child_ignore_ns("Date");
            let contests: Vec<&Element> = value.get_children_ignore_ns("Contest");
            Self {
                election_identifier: value
                    .get_child_ignore_ns("ElectionIdentifier")
                    .unwrap()
                    .into(),
                date: date.map(ComplexDateRangeStructure::from),
                contests: contests.into_iter().map(Contest::from).collect(),
            }
        }
    }
    impl Serialize for Election {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            let mut state = serializer.serialize_struct("ElectionStructure", 5)?;
            state.serialize_field("id", &self.election_identifier.id)?;
            state.serialize_field("date", &self.date)?;
            state.serialize_field("name", &self.election_identifier.election_name)?;
            state.serialize_field("category", &self.election_identifier.election_category)?;
            state.end()
            // state.end()
        }
    }
    #[derive(Clone)]
    pub struct Contest {
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
    impl From<&Element> for Contest {
        fn from(value: &Element) -> Self {
            Self {
                contest_identifier: value
                    .get_child_ignore_ns("ContestIdentifier")
                    .unwrap()
                    .into(),
                area: value.get_child_ignore_ns("Area").map(AreaStructure::from),
                position: value
                    .get_child_ignore_ns("Position")
                    .map(PositionStructure::from),
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
    impl Serialize for Contest {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            let mut state = serializer.serialize_struct("ContestStructure", 5)?;
            state.serialize_field("id", &self.contest_identifier.id)?;
            state.serialize_field("short_code", &self.contest_identifier.short_code)?;
            state.serialize_field("name", &self.contest_identifier.contest_name)?;
            state.serialize_field("position", &self.position.clone().map(|x| x.text))?;
            state.serialize_field("number", &self.number_of_positions)?;
            state.end()
        }
    }
}

pub mod polling {
    use crate::eml_schema::{EventIdentifierStructure, PollingDistrictStructure};
    use crate::xml_extension::IgnoreNS;
    use minidom::Element;

    pub struct PollingDistrictListStructure {
        pub(crate) event_identifier: EventIdentifierStructure,
        pub(crate) polling_districts: Vec<PollingDistrictStructure>,
    }

    impl From<&Element> for PollingDistrictListStructure {
        fn from(value: &Element) -> Self {
            let event_identifier = value.get_child_ignore_ns("EventIdentifier").unwrap();
            let districts: Vec<&Element> = value.get_children_ignore_ns("PollingDistrict");
            Self {
                event_identifier: event_identifier.into(),
                polling_districts: districts
                    .into_iter()
                    .map(PollingDistrictStructure::from)
                    .collect(),
            }
        }
    }
}
//Externals

#[derive(Clone, Default)]
pub struct AuthorityAddressStructure;

impl From<Element> for AuthorityAddressStructure {
    fn from(value: Element) -> Self {
        Self {}
    }
}
