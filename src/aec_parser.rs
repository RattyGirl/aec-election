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
                event_identifier: event_identifier.try_into().unwrap(),
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
                    .try_into()
                    .unwrap(),
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
                    .try_into()
                    .unwrap(),
                candidates: value
                    .get_children_ignore_ns("Candidate")
                    .into_iter()
                    .map(CandidateStructure::try_from)
                    .map(|x| x.unwrap())
                    .collect(),
                affiliations: value
                    .get_children_ignore_ns("Affiliation")
                    .into_iter()
                    .map(AffiliationStructure::try_from)
                    .map(|x| x.unwrap())
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
                event_identifier: event_identifier.try_into().unwrap(),
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
                    .try_into()
                    .unwrap(),
                date: date
                    .map(ComplexDateRangeStructure::try_from)
                    .map(|x| x.unwrap()),
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
                    .try_into()
                    .unwrap(),
                area: value
                    .get_child_ignore_ns("Area")
                    .map(AreaStructure::try_from)
                    .map(|x| x.unwrap()),
                position: value
                    .get_child_ignore_ns("Position")
                    .map(PositionStructure::try_from)
                    .map(|x| x.unwrap()),
                voting_method: value
                    .get_children_ignore_ns("VotingMethod")
                    .into_iter()
                    .map(VotingMethodType::try_from)
                    .map(|x| x.unwrap())
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
                event_identifier: event_identifier.try_into().unwrap(),
                polling_districts: districts
                    .into_iter()
                    .map(PollingDistrictStructure::try_from)
                    .map(|x| x.unwrap())
                    .collect(),
            }
        }
    }
}

pub mod results {
    use crate::eml_schema::{
        CandidateIdentifierStructure, ContestIdentifierStructure, EMLError,
        ElectionIdentifierStructure, EventIdentifierStructure, ManagingAuthorityStructure,
    };
    use crate::xml_extension::IgnoreNS;
    use minidom::Element;
    use std::str::FromStr;

    pub struct ResultsMediaFeed {
        //@Id
        pub(crate) id: String,
        //@Created
        created: String,
        //?ManagingAuthority
        managing_authority: Option<ManagingAuthorityStructure<()>>,
        //?MessageLanguage
        message_language: Option<String>,
        //?Cycle
        cycle: Option<CycleStructure>,
        //Results
        pub(crate) results: ResultsStructure,
    }

    impl From<&Element> for ResultsMediaFeed {
        fn from(value: &Element) -> Self {
            Self {
                id: value.attr("Id").unwrap().to_string(),
                created: value.attr("Created").unwrap().to_string(),
                managing_authority: value
                    .get_child_ignore_ns("ManagingAuthority")
                    .map(|x| x.into()),
                message_language: value
                    .get_child_ignore_ns("MessageLanguage")
                    .map(|x| x.text()),
                cycle: value.get_child_ignore_ns("Cycle").map(|x| x.into()),
                results: value.get_child_ignore_ns("Results").unwrap().into(),
            }
        }
    }

    struct CycleStructure {
        created: String,
        guid: String,
    }
    impl From<&Element> for CycleStructure {
        fn from(value: &Element) -> Self {
            Self {
                created: value.attr("Created").unwrap().to_string(),
                guid: value.text(),
            }
        }
    }

    pub struct ResultsStructure {
        //@Updated
        updated: Option<String>,
        //@Phase
        phase: String,
        //EventIdentifier
        pub(crate) event_identifier: EventIdentifierStructure,
        //Election
        pub(crate) election: Vec<ElectionStructure>,
    }

    impl From<&Element> for ResultsStructure {
        fn from(value: &Element) -> Self {
            Self {
                updated: value.attr("Updated").map(|x| x.to_string()),
                phase: value.attr("Phase").unwrap_or("").to_string(),
                event_identifier: EventIdentifierStructure::try_from(
                    value.get_child_ignore_ns("EventIdentifier").unwrap(),
                )
                .unwrap(),
                election: value
                    .get_children_ignore_ns("Election")
                    .into_iter()
                    .map(ElectionStructure::from)
                    .collect(),
            }
        }
    }

    struct ElectionStructure {
        updated: Option<String>,
        election_identifier: ElectionIdentifierStructure,
        election_type: ElectionType,
    }

    impl From<&Element> for ElectionStructure {
        fn from(value: &Element) -> Self {
            let category: ElectionType = if let Some(category) = value.get_child_ignore_ns("House")
            {
                ElectionType::House(category.into())
            } else if let Some(category) = value.get_child_ignore_ns("Senate") {
                ElectionType::Senate(category.into())
            } else if let Some(category) = value.get_child_ignore_ns("Referendum") {
                ElectionType::Referendum(category.into())
            } else {
                ElectionType::Other
            };
            Self {
                updated: value.attr("Updated").map(|x| x.to_string()),
                election_identifier: ElectionIdentifierStructure::try_from(
                    value.get_child_ignore_ns("ElectionIdentifier").unwrap(),
                )
                .unwrap(),
                election_type: category,
            }
        }
    }

    enum ElectionType {
        House(HouseMediaFeedStructure),
        Senate(SenateMediaFeedStructure),
        Referendum(ReferendumMediaFeedStructure),
        Other,
    }

    struct HouseMediaFeedStructure {
        contests: Vec<HouseContestsStructure>,
    }

    impl From<&Element> for HouseMediaFeedStructure {
        fn from(value: &Element) -> Self {
            Self {
                contests: value
                    .get_child_ignore_ns("Contests")
                    .unwrap()
                    .get_children_ignore_ns("Contest")
                    .into_iter()
                    .map(HouseContestsStructure::from)
                    .collect(),
            }
        }
    }
    struct HouseContestsStructure {
        //@Updated
        updated: Option<String>,
        //@Declared
        declared: Option<String>,
        //@Projected
        projected: Option<String>,
        //ContestIdentifier
        contest_identifier: ContestIdentifierStructure,
        //Enrolment
        enrolment: u32,
        //FirstPreferences
        first_prefs: HouseFirstPreferencesStructure,
        //TwoCandidatePreferred
        tcp: Option<TwoCandidatePreferredStructure>,
    }

    impl From<&Element> for HouseContestsStructure {
        fn from(value: &Element) -> Self {
            Self {
                updated: value.attr("Updated").map(|x| x.to_string()),
                declared: value.attr("Declared").map(|x| x.to_string()),
                projected: value.attr("Projected").map(|x| x.to_string()),
                contest_identifier: value
                    .get_child_ignore_ns("ContestIdentifier")
                    .unwrap()
                    .try_into()
                    .unwrap(),
                enrolment: value
                    .get_child_ignore_ns("Enrolment")
                    .unwrap()
                    .text()
                    .parse()
                    .unwrap_or(0),
                first_prefs: value
                    .get_child_ignore_ns("FirstPreferences")
                    .unwrap()
                    .into(),
                tcp: value
                    .get_child_ignore_ns("TwoCandidatePreferred")
                    .map(|x| x.into()),
            }
        }
    }

    struct HouseFirstPreferencesStructure {
        updated: Option<String>,
        polling_places_returned: Option<u32>,
        polling_places_expected: Option<u32>,
        candidates: Vec<HouseCandidateResultsStructure>,
        ghosts: Vec<HouseCandidateResultsStructure>,
        formal: HouseRawTotal,
        informal: HouseRawTotal,
        total: HouseRawTotal,
    }

    impl From<&Element> for HouseFirstPreferencesStructure {
        fn from(value: &Element) -> Self {
            Self {
                updated: value.attr("Updated").map(|x| x.to_string()),
                polling_places_returned: value
                    .attr("PollingPlacesReturned")
                    .map(|x| u32::from_str(x).unwrap_or(0)),
                polling_places_expected: value
                    .attr("PollingPlacesExpected")
                    .map(|x| u32::from_str(x).unwrap_or(0)),
                candidates: value
                    .get_children_ignore_ns("Candidate")
                    .into_iter()
                    .map(HouseCandidateResultsStructure::from)
                    .collect(),
                ghosts: value
                    .get_children_ignore_ns("Ghost")
                    .into_iter()
                    .map(HouseCandidateResultsStructure::from)
                    .collect(),
                formal: value.get_child_ignore_ns("Formal").unwrap().into(),
                informal: value.get_child_ignore_ns("Informal").unwrap().into(),
                total: value.get_child_ignore_ns("Total").unwrap().into(),
            }
        }
    }
    struct TwoCandidatePreferredStructure {
        updated: Option<String>,
        polling_places_returned: Option<u32>,
        polling_places_expected: Option<u32>,
        candidates: Vec<HouseCandidateResultsStructure>,
    }

    impl From<&Element> for TwoCandidatePreferredStructure {
        fn from(value: &Element) -> Self {
            Self {
                updated: value.attr("Updated").map(|x| x.to_string()),
                polling_places_returned: value
                    .attr("PollingPlacesReturned")
                    .map(|x| u32::from_str(x).unwrap_or(0)),
                polling_places_expected: value
                    .attr("PollingPlacesExpected")
                    .map(|x| u32::from_str(x).unwrap_or(0)),
                candidates: value
                    .get_children_ignore_ns("Candidate")
                    .into_iter()
                    .map(HouseCandidateResultsStructure::from)
                    .collect(),
            }
        }
    }
    struct HouseRawTotal {
        votes: u32,
        matched_historic: Option<u32>,
        votes_by_type: Vec<VotesByTypeStructure>,
    }

    impl From<&Element> for HouseRawTotal {
        fn from(value: &Element) -> Self {
            Self {
                votes: value.text().parse().unwrap_or(0),
                matched_historic: value
                    .attr("MatchedHistoric")
                    .map(|x| u32::from_str(x).unwrap_or(0)),
                votes_by_type: value
                    .get_child_ignore_ns("VotesByType")
                    .unwrap()
                    .get_children_ignore_ns("Votes")
                    .into_iter()
                    .map(VotesByTypeStructure::from)
                    .collect(),
            }
        }
    }
    struct VotesByTypeStructure {
        votes_by_type_enum: String,
        votes: String,
        historic: Option<String>,
        percentage: Option<String>,
        swing: Option<String>,
        matched_historic: Option<String>,
    }

    impl From<&Element> for VotesByTypeStructure {
        fn from(value: &Element) -> Self {
            Self {
                votes: value.text(),
                votes_by_type_enum: value.attr("Type").unwrap().to_string(),
                historic: value.attr("Historic").map(|x| x.to_string()),
                percentage: value.attr("Percentage").map(|x| x.to_string()),
                swing: value.attr("Swing").map(|x| x.to_string()),
                matched_historic: value.attr("MatchedHistoric").map(|x| x.to_string()),
            }
        }
    }
    struct HouseCandidateResultsStructure {
        candidate_identifier: CandidateIdentifierStructure,
        ballot_position: Option<String>,
        elected: Option<String>,
        votes: u32,
        matched_historic: Option<u32>,
        votes_by_type: Vec<VotesByTypeStructure>,
    }

    impl From<&Element> for HouseCandidateResultsStructure {
        fn from(value: &Element) -> Self {
            Self {
                candidate_identifier: value.get_child_ignore_ns("CandidateIdentifier").unwrap().try_into().unwrap(),
                ballot_position: value.get_child_ignore_ns("BallotPosition").map(|x| x.text()),
                elected: value.get_child_ignore_ns("BallotPosition").map(|x| x.text()),
                votes: value.get_child_ignore_ns("Votes").unwrap().text().parse().unwrap(),
                matched_historic: value.get_child_ignore_ns("Votes").unwrap().attr("MatchedHistoric").map(|x| u32::from_str(x).unwrap_or(0)),
                votes_by_type: value
                    .get_child_ignore_ns("VotesByType")
                    .unwrap()
                    .get_children_ignore_ns("Votes")
                    .into_iter()
                    .map(VotesByTypeStructure::from)
                    .collect(),
            }
        }
    }

    struct SenateMediaFeedStructure {

    }

    impl From<&Element> for SenateMediaFeedStructure {
        fn from(value: &Element) -> Self {
            Self{}
        }
    }


    struct ReferendumMediaFeedStructure {}
    impl From<&Element> for ReferendumMediaFeedStructure {
        fn from(value: &Element) -> Self {
            Self{}
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
