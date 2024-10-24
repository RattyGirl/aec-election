use crate::qld_election::count_round_parties_type::Party;
use crate::qld_election::ecq_types::Election;
use election::sql_db::MySQLDB;
use election::SerialiseDB;
use election_derive::SerialiseDB;
use futures::task::SpawnExt;
use quote::__private::ext::RepToTokensExt;
use serde::{de, Deserialize};
use std::io::{Cursor, Read};
use std::str::FromStr;
use futures::{StreamExt, TryStreamExt};
use sqlx::postgres::PgRow;
use sqlx::Row;
use zip::ZipArchive;

const QLD_ELECTION_LINK: &str = "https://resultsdata.elections.qld.gov.au/PublicResults.zip";
const QLD_2020_ELECTION: &str = "https://resultsdata.elections.qld.gov.au/XMLData/publicResults_State2020_aurukun2020_Final.zip";

mod xs {
    pub type UnsignedShort = u8;
    pub type UnsignedInt = u16;
    pub type UnsignedLong = u32;
    pub type Float = f32;
    pub type Str = String;
    pub type Byte = u8;
    pub type DateTime = String;
}

pub async fn read_result(database: &mut MySQLDB) {
    let zip_bytes = reqwest::get(QLD_2020_ELECTION)
        .await
        .unwrap()
        .bytes()
        .await
        .unwrap();
    let mut zip_cursor = Cursor::new(zip_bytes);
    let mut zipfile = ZipArchive::new(zip_cursor).unwrap();
    let mut results_string: String = String::new();
    let results_file = zipfile
        .by_name("publicResults.xml")
        .unwrap()
        .read_to_string(&mut results_string);

    let results: ECQResults = quick_xml::de::from_str(results_string.as_str()).unwrap();

    setup_info(database, &results).await;
    count(database, &results).await;
    println!(
        "{}",
        candidate_section_type::Candidate {
            party: Some("Labor Party".to_string()),
            party_code: Some("ALP".to_string()),
            sitting: Some("NO".to_string()),
            surname: Some("Holmes".to_string()),
            given_names: Some("Chloe G".to_string()),
            ballot_name: "HOLMES, Chloe".to_string(),
            ballot_order: 4
        }
        .insert(database)
        .await
    );
}

async fn setup_info(database: &mut MySQLDB, results: &ECQResults) {
    let qld_election = results.elections.first().unwrap();
    &database
        .run_raw(String::from("DELETE FROM elections"))
        .await;
    &database
        .run_raw(String::from("DELETE FROM booth_district"))
        .await;
    &database
        .run_raw(String::from("DELETE FROM candidate_district"))
        .await;
    &database
        .run_raw(String::from("DELETE FROM candidates;"))
        .await;
    &database.run_raw(String::from("DELETE FROM booths")).await;
    &database
        .run_raw(String::from("DELETE FROM districts"))
        .await;

    //add election
    for election in &results.elections {
        println!("{:?}", &database
            .run_raw(format!(
                "INSERT INTO aec.elections(date, gen_date, name, event_type, state, ec_id) VALUES($${}$$, $${}$$, $${}$$, $${}$$, $${}$$, $${}$$);",
                &election.election_day, &election.gen_date, &election.election_name, &election.event_type, "QLD", &election.id)
            )
            .await);
    }

    //add candidates to db
    for district in &qld_election.districts.districts {
        &database
            .run_raw(format!(
                "INSERT INTO aec.districts(election_id, district_name, enrolment, last_updated, final_count, number, voting_method, voting_system, percent_counted, num_elected) VALUES((SELECT id FROM elections WHERE elections.date LIKE $${}$$ AND elections.name LIKE $${}$$),\
                 $${}$$, $${}$$, $${}$$, $${}$$, $${}$$, $${}$$, $${}$$, $${}$$, $${}$$);",
                &qld_election.election_day, &qld_election.election_name,&district.district_name.clone(), &district.enrolment.clone().to_string(), &district.last_updated.clone(), &district.final_count.clone().map(|x| x.to_string()).unwrap_or("".to_string()), &district.number.clone().to_string(), &district.voting_method.clone(), &district.voting_system.clone(), &district.percent_counted.clone().map(|x| x.to_string()).unwrap_or("".to_string()), &district.num_elected.clone())
            )
            .await;
        for can in &district.candidates.candidates {
            &database
                .run_raw(format!(
                    "INSERT INTO aec.candidates(name, party_name, party_code, sitting, surname, given_names) VALUES($${}$$, $${}$$, $${}$$, {}, $${}$$, $${}$$);",
                    &can.ballot_name,
                    can.party.clone().unwrap_or(String::new()),
                    can.party_code.clone().unwrap_or(String::new()),
                    "NULL", //TODO sitting
                    can.surname.clone().unwrap_or(String::new()),
                    can.given_names.clone().unwrap_or(String::new()),
                ))
                .await;
            &database.run_raw(format!("INSERT INTO aec.candidate_district \
            (candidate_id, district_id, ballot_order) VALUES \
            ((SELECT candidates.id FROM candidates WHERE candidates.name LIKE $${}$$ AND candidates.party_name LIKE $${}$$), \
            (SELECT districts.id FROM districts WHERE districts.district_name LIKE $${}$$), \
            $${}$$);", &can.ballot_name, &can.party.clone().unwrap_or(String::new()), &district.district_name, &can.ballot_order)).await;
        }
    }
    //add booths to db
    for venue in &results.venues.booths {
        &database.run_raw(format!("INSERT INTO aec.booths (ec_id, name, building_name, street_no, street_name, locality, state, postcode, lat, long, abolished)\
        VALUES ($${}$$, $${}$$, $${}$$, $${}$$, $${}$$, $${}$$, $${}$$, $${}$$, $${}$$, $${}$$, $${}$$);",
                                  &venue.id, &venue.name.clone(), &venue.building_name.clone().map(|x| x.to_string()).unwrap_or("".to_string()), &venue.street_no.clone().map(|x| x.to_string()).unwrap_or("".to_string()), &venue.street_name.clone().map(|x| x.to_string()).unwrap_or("".to_string()), &venue.locality.clone().map(|x| x.to_string()).unwrap_or("".to_string()), &venue.state.clone().clone().map(|x| x.to_string()).unwrap_or("".to_string()), &venue.postcode.clone().map(|x| x.to_string()).unwrap_or("".to_string()), &venue.lat.clone().map(|x| x.to_string()).unwrap_or("0".to_string()), &venue.long.clone().map(|x| x.to_string()).unwrap_or("0".to_string()), &venue.abolished.clone())).await;
        for booth_district in &venue.districts {
            //TEMPORARY
            if (booth_district
                .election
                .eq("2024 Ipswich City Council Councillor By-election - Division 4")
                || booth_district.election.eq("Aurukun Councillor By-Election"))
            {
                continue;
            }
            &database.run_raw(format!("INSERT INTO aec.booth_district (booth_id, district_id) VALUES \
            ((SELECT booths.id FROM booths WHERE booths.name LIKE $${}$$ AND booths.building_name LIKE $${}$$),\
             (SELECT districts.id FROM districts WHERE districts.district_name LIKE $${}$$));", &venue.name.clone(), &venue.building_name.clone().unwrap_or("".to_string()), &booth_district.district_name)).await;
        }
    }
}

async fn count(database: &mut MySQLDB, results: &ECQResults) {
    let qld_election = results.elections.first().unwrap();
    &database
        .run_raw(String::from("DELETE FROM unof_prelim_count"))
        .await;

    for district in &qld_election.districts.districts {
        for count_round in &district.count_rounds {
            println!("{:?}", count_round);
            if (!count_round.count_name.eq("Unofficial Preliminary Count")) {
                continue;
            }
            for booth in &count_round.booth_results.booths {
                let booth_id = &booth.id.clone();
                for candidate_result in &booth.primary_vote_results.candidate_results {
                    let query = format!("INSERT INTO aec.unof_prelim_count (candidate_id, booth_id, vote_count) VALUES (\
                (SELECT candidate_id FROM candidate_district cd JOIN candidates c ON cd.candidate_id = c.id JOIN districts d ON cd.district_id = d.id \
                WHERE c.name LIKE $${can_name}$$ AND d.district_name LIKE $${district_name}$$),\
                (SELECT booths.id FROM booths WHERE booths.ec_id LIKE $${booth_id}$$),\
                {vote_count})", can_name=candidate_result.ballot_name, district_name=district.district_name,
                                        booth_id=booth_id, vote_count=candidate_result.count);
                    println!("{}", query);
                    &database.run_raw(query).await;
                }
            }
        }
    }
}
//contestTypes
//
//
//
// #[derive(Debug, PartialEq, Default, Deserialize)]
// #[serde(default)]
// struct Candidates {
//     #[serde(rename = "@count")]
//     count: xs::UnsignedInt,
//     #[serde(rename = "candidate")]
//     candidates: Vec<Candidate>,
// }
//
//
//
//
// #[derive(Debug, PartialEq, Default, Deserialize)]
// #[serde(default)]
// struct IndicativeCountDetailsSelectedCandidate {
//     #[serde(rename = "primaryVotes")]
//     primary_votes: UnsignedInt,
//     #[serde(rename = "afterPreferences")]
//     after_preferences: CountPercentageSection,
//     #[serde(rename = "@ballotName")]
//     ballot_name: xs::Str,
//     #[serde(rename = "@ballotOrderNumber")]
//     ballot_order_number: xs::UnsignedShort,
// }
//
// #[derive(Debug, PartialEq, Default, Deserialize)]
// #[serde(default)]
// struct IndicativeCountDetailsOtherCandidate {
//     #[serde(rename = "@ballotName")]
//     ballot_name: xs::Str,
//     #[serde(rename = "@ballotOrderNumber")]
//     ballot_order_number: xs::UnsignedShort,
//     // #[serde(rename = "preferenceCount")] TODO
//     // preference_count: Vec<IndicativeCountDetailsOtherCandidatePreferenceCount>
// }
//
// #[derive(Debug, PartialEq, Default, Deserialize)]
// #[serde(default)]
// struct Party {
//     #[serde(rename = "@code")]
//     code: xs::Str,
//     #[serde(rename = "@name")]
//     name: xs::Str, //TODO party counts
// }
//
//
//

//---------------------------------------------
//---------------------------------------------
//---------------------------------------------
//---------------------------------------------
//---------------------------------------------
// NEW FORMAT
//---------------------------------------------
//---------------------------------------------
//---------------------------------------------
//---------------------------------------------

fn deserialize_bool<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: de::Deserializer<'de>,
{
    let s: &str = de::Deserialize::deserialize(deserializer)?;

    match s {
        "YES" => Ok(true),
        "NO" => Ok(false),
        _ => Err(de::Error::unknown_variant(s, &["YES", "NO"])),
    }
}

#[derive(Debug, PartialEq, Default, Deserialize)]
#[serde(default)]
struct CountRoundPartiesType {
    #[serde(rename = "@id")]
    id: xs::UnsignedInt,
    #[serde(rename = "@round")]
    round: xs::UnsignedInt,
    #[serde(rename = "@countName")]
    count_name: xs::Str,
    #[serde(rename = "@preliminary")]
    preliminary: xs::Str,
    #[serde(rename = "@unofficial")]
    unofficial: xs::Str,
    #[serde(rename = "@indicative")]
    indicative: xs::Str,
    #[serde(rename = "@preferences")]
    preferences: xs::Str,
    #[serde(rename = "party")]
    parties: Vec<count_round_parties_type::Party>,
}

mod count_round_parties_type {
    use crate::qld_election::xs;
    use serde::Deserialize;

    #[derive(Debug, PartialEq, Default, Deserialize)]
    #[serde(default)]
    pub(crate) struct Party {
        #[serde(rename = "@code")]
        code: xs::Str,
        #[serde(rename = "@name")]
        name: xs::Str,
        #[serde(rename = "count")]
        count: xs::UnsignedLong,
        #[serde(rename = "percentage")]
        percentage: xs::Float,
    }
}

#[derive(Debug, PartialEq, Default, Deserialize)]
#[serde(default)]
struct CountRoundCouncillorType {
    #[serde(rename = "@countName")]
    pub count_name: xs::Str,
    #[serde(rename = "@id")]
    pub id: xs::UnsignedLong,
    #[serde(deserialize_with = "deserialize_bool", rename = "@indicative")]
    pub indicative: bool,
    #[serde(rename = "@preferences")]
    pub preferences: xs::Str,
    #[serde(rename = "@preliminary")]
    pub preliminary: xs::Str,
    #[serde(rename = "@round")]
    pub round: xs::UnsignedInt,
    #[serde(rename = "@unofficial")]
    pub unofficial: xs::Str,
    #[serde(rename = "@lastUpdated")]
    pub last_updated: Option<xs::Str>,
    #[serde(rename = "@percentRollCounted")]
    pub percent_counted: Option<xs::Float>,
    #[serde(rename = "party")]
    pub parties: Vec<Party>,

    #[serde(rename = "totalVotes")]
    pub total_votes: Option<xs::UnsignedLong>,
    #[serde(rename = "totalFormalVotes")]
    pub total_formal_votes: Option<CountPercentageSection>,
    #[serde(rename = "totalInformalVotes")]
    pub total_informal_votes: Option<CountPercentageSection>,
    #[serde(rename = "primaryVoteResults")]
    pub primary_vote_results: Option<PrimaryVoteResultsSection>,
    #[serde(rename = "booths")]
    pub booth_results: BoothsSection,
}

#[derive(Debug, PartialEq, Default, Deserialize)]
#[serde(default)]
struct BoothsSection {
    #[serde(rename = "@count")]
    count: xs::UnsignedInt,

    #[serde(rename = "booth")]
    booths: Vec<booths_section::Booth>,
}

mod booths_section {
    use crate::qld_election::deserialize_bool;
    use crate::qld_election::{
        indicative_count_details, xs, PreferenceDistributionSummary,
        PrimaryVoteResponseResultsSection, PrimaryVoteResultsSection, TwoCandidateVotesSection,
    };
    use serde::Deserialize;

    #[derive(Debug, PartialEq, Default, Deserialize)]
    #[serde(default)]
    pub(crate) struct Booth {
        #[serde(rename = "@id")]
        pub(crate) id: xs::Str,
        #[serde(rename = "@name")]
        pub booth_name: xs::Str,
        #[serde(deserialize_with = "deserialize_bool", rename = "@in")]
        result_in: bool,
        #[serde(rename = "@typeCode")]
        type_code: xs::Str,
        #[serde(rename = "@typeDescription")]
        type_description: xs::Str,
        #[serde(rename = "@lastUpdated")]
        last_updated: Option<xs::Str>,
        #[serde(rename = "votes")]
        votes: xs::UnsignedLong,
        #[serde(rename = "ballots")]
        ballots: xs::UnsignedLong,
        #[serde(rename = "formalVotes")]
        formal_votes: xs::UnsignedLong,
        #[serde(rename = "formalBallots")]
        formal_ballots: xs::UnsignedLong,
        #[serde(rename = "informalVotes")]
        informal_votes: xs::UnsignedLong,
        #[serde(rename = "informalBallots")]
        informal_ballots: xs::UnsignedLong,
        #[serde(rename = "primaryVoteResults")]
        pub primary_vote_results: PrimaryVoteResultsSection,
        #[serde(rename = "twoCandidateVotes")]
        two_candidate_votes: TwoCandidateVotesSection,
        #[serde(rename = "primaryVoteResponseResults")]
        primary_vote_response_results: PrimaryVoteResponseResultsSection,
        #[serde(rename = "indicativeCountDetails")]
        indicative_count_details: indicative_count_details::IndicativeCountDetails,
        #[serde(rename = "preferenceDistributionSummary")]
        preference_redis_summary: PreferenceDistributionSummary,
    }
}

#[derive(Debug, PartialEq, Default, Deserialize)]
#[serde(default)]
struct CountPercentageSection {
    #[serde(rename = "count")]
    count: xs::UnsignedLong,
    #[serde(rename = "percentage")]
    percentage: xs::Float,
}

#[derive(Debug, PartialEq, Default, Deserialize)]
#[serde(default)]
struct PrimaryVoteResultsSection {
    #[serde(rename = "candidate")]
    pub candidate_results: Vec<CandidateResult>,
}

#[derive(Debug, PartialEq, Default, Deserialize)]
#[serde(default)]
struct PrimaryVoteResponseResultsSection {
    #[serde(rename = "response")]
    responses: Vec<ResponseResult>,
}

#[derive(Debug, PartialEq, Default, Deserialize)]
#[serde(default)]
struct CandidateResult {
    #[serde(rename = "@ballotOrderNumber")]
    ballot_order: xs::UnsignedShort,
    #[serde(rename = "@ballotName")]
    pub ballot_name: xs::Str,

    //EXTENSION OF CountPercentageSection
    #[serde(rename = "count")]
    pub count: xs::UnsignedLong,
    #[serde(rename = "percentage")]
    percentage: xs::Float,
}

#[derive(Debug, PartialEq, Default, Deserialize)]
#[serde(default)]
struct ResponseResult {
    #[serde(rename = "@responseOrderNumber")]
    response_order_number: xs::UnsignedShort,
    #[serde(rename = "@responseName")]
    response_name: xs::Str,

    //EXTENSION OF CountPercentageSection
    #[serde(rename = "count")]
    count: xs::UnsignedLong,
    #[serde(rename = "percentage")]
    percentage: xs::Float,
}

#[derive(Debug, PartialEq, Default, Deserialize)]
#[serde(default)]
struct TwoCandidateVotesSection {
    #[serde(rename = "candidate")]
    candidates: Vec<CandidateResult>, //LIMIT OF 2 HERE
}

#[derive(Debug, PartialEq, Default, Deserialize)]
#[serde(default)]
struct PreferenceDistributionSummary {
    #[serde(rename = "primaryVotes")]
    primary_votes: PrimaryVoteResultsSection,
    #[serde(rename = "preferenceDistribution")]
    pref_dis: Vec<PreferenceDistribution>,
}
#[derive(Debug, PartialEq, Default, Deserialize)]
#[serde(default)]
struct PreferenceDistribution {
    #[serde(rename = "@distribution")]
    distribution: xs::UnsignedShort,
    #[serde(rename = "@excludedBallotName")]
    excluded_ballot_name: xs::Str,
    #[serde(rename = "@excludedBallotOrder")]
    excluded_ballot_order: xs::UnsignedShort,
    #[serde(rename = "candidatePreferences")]
    candidate_preferences: Vec<CandidateResult>,
    #[serde(rename = "exhausted")]
    exhausted: CountPercentageSection,
    #[serde(rename = "votesDistributed")]
    votes_distributed: xs::UnsignedLong,
    #[serde(rename = "votesRemainingInCount")]
    votes_remaining_in_count: xs::UnsignedLong,
}

mod indicative_count_details {
    use crate::qld_election::{xs, CountPercentageSection};
    use serde::Deserialize;

    #[derive(Debug, PartialEq, Default, Deserialize)]
    #[serde(default)]
    pub(crate) struct IndicativeCountDetails {
        #[serde(rename = "selectedCandidate")]
        selected_candidate: Vec<SelectedCandidate>,
        #[serde(rename = "otherCandidate")]
        other_candidate: Vec<OtherCandidate>,
    }

    #[derive(Debug, PartialEq, Default, Deserialize)]
    #[serde(default)]
    struct SelectedCandidate {
        #[serde(rename = "@ballotName")]
        ballot_name: xs::Str,
        #[serde(rename = "@ballotOrderNumber")]
        ballot_order_num: xs::UnsignedShort,

        #[serde(rename = "primaryVotes")]
        primary_votes: xs::UnsignedShort,
        #[serde(rename = "afterPreferences")]
        after_preferences: CountPercentageSection,
    }

    #[derive(Debug, PartialEq, Default, Deserialize)]
    #[serde(default)]
    struct OtherCandidate {
        #[serde(rename = "@ballotName")]
        ballot_name: xs::Str,
        #[serde(rename = "@ballotOrderNumber")]
        ballot_order_num: xs::UnsignedShort,

        #[serde(rename = "preferenceCount")]
        after_preferences: Vec<PreferenceCount>, //MAX 2
        #[serde(rename = "exhausted")]
        exhausted: xs::UnsignedLong,
    }

    #[derive(Debug, PartialEq, Default, Deserialize)]
    #[serde(default)]
    struct PreferenceCount {
        #[serde(rename = "$text")]
        count: xs::UnsignedLong,
        #[serde(rename = "@ballotName")]
        ballot_name: xs::Str,
        #[serde(rename = "@ballotOrderNumber")]
        ballot_order_number: xs::UnsignedShort,
    }
}

#[derive(Debug, PartialEq, Default, Deserialize)]
#[serde(default)]
struct CandidateSectionType {
    #[serde(rename = "@count")]
    count: xs::UnsignedInt,
    #[serde(rename = "candidate")]
    candidates: Vec<candidate_section_type::Candidate>,
}
mod candidate_section_type {
    use crate::qld_election::xs;
    use election::sql_db::MySQLDB;
    use election::SerialiseDB;
    use election_derive::SerialiseDB;
    use serde::Deserialize;

    #[derive(Debug, PartialEq, Default, Deserialize, SerialiseDB)]
    #[serde(default)]
    #[db(table_name = "candidates")]
    pub struct Candidate {
        #[serde(rename = "@ballotOrderNumber")]
        pub(crate) ballot_order: xs::UnsignedShort,
        #[serde(rename = "@ballotName")]
        pub(crate) ballot_name: xs::Str,
        #[serde(rename = "@partyCode")]
        #[db(null_value="".to_string())]
        pub(crate) party_code: Option<xs::Str>,
        #[serde(rename = "@party")]
        #[db(null_value="".to_string())]
        pub(crate) party: Option<xs::Str>,
        #[serde(rename = "@sitting")]
        #[db(null_value="".to_string())]
        pub(crate) sitting: Option<xs::Str>,
        #[serde(rename = "surname")]
        #[db(null_value="".to_string())]
        pub(crate) surname: Option<xs::Str>,
        #[serde(rename = "givenNames")]
        #[db(null_value="".to_string())]
        pub(crate) given_names: Option<xs::Str>,
    }
}

#[derive(Debug, PartialEq, Default, Deserialize)]
#[serde(default)]
struct ECQResults {
    #[serde(rename = "remarks")]
    remarks: xs::Str,
    #[serde(rename = "generationDateTime")]
    gen_date: xs::DateTime,
    #[serde(rename = "venues")]
    venues: ecq_types::Venues,
    #[serde(rename = "election")]
    elections: Vec<Election>,
}

mod ecq_types {
    use crate::qld_election::{deserialize_bool, CandidateSectionType, CountRoundCouncillorType};
    use crate::qld_election::{xs, CountRoundPartiesType};
    use serde::Deserialize;

    //<venues>
    #[derive(Debug, PartialEq, Default, Deserialize)]
    #[serde(default)]
    pub struct Venues {
        #[serde(rename = "@count")]
        count: Option<xs::UnsignedInt>,
        #[serde(rename = "booth")]
        pub(crate) booths: Vec<Booth>,
    }

    #[derive(Debug, PartialEq, Default, Deserialize)]
    #[serde(default)]
    pub struct Booth {
        #[serde(rename = "@id")]
        pub(crate) id: xs::Str,
        #[serde(rename = "@name")]
        pub(crate) name: xs::Str,
        #[serde(rename = "@buildingName")]
        pub(crate) building_name: Option<xs::Str>,
        #[serde(rename = "@streetNo")]
        pub(crate) street_no: Option<xs::Str>,
        #[serde(rename = "@streetName")]
        pub(crate) street_name: Option<xs::Str>,
        #[serde(rename = "@locality")]
        pub(crate) locality: Option<xs::Str>,
        #[serde(rename = "@state")]
        pub(crate) state: Option<xs::Str>,
        #[serde(rename = "@postcode")]
        pub(crate) postcode: Option<xs::UnsignedInt>,
        #[serde(rename = "@latitude")]
        pub(crate) lat: Option<xs::Float>,
        #[serde(rename = "@longitude")]
        pub(crate) long: Option<xs::Float>,
        #[serde(deserialize_with = "deserialize_bool", rename = "@abolished")]
        pub(crate) abolished: bool,
        #[serde(rename = "boothDistrict")]
        pub(crate) districts: Vec<BoothDistrict>,
    }

    #[derive(Debug, PartialEq, Default, Deserialize)]
    #[serde(default)]
    pub struct BoothDistrict {
        #[serde(rename = "@districtName")]
        pub district_name: xs::Str,
        #[serde(rename = "@election")]
        pub election: xs::Str,
        #[serde(rename = "@jointType")]
        pub joint_type: Option<xs::Str>,
    }

    #[derive(Debug, PartialEq, Default, Deserialize)]
    #[serde(default)]
    pub struct Election {
        #[serde(rename = "@id")]
        pub(crate) id: xs::UnsignedLong,
        #[serde(rename = "@electionName")]
        pub(crate) election_name: xs::Str,
        #[serde(rename = "@eventType")]
        pub(crate) event_type: xs::Str,
        #[serde(rename = "@electionDay")]
        pub(crate) election_day: xs::Str,
        #[serde(rename = "generationDateTime")]
        pub(crate) gen_date: String,
        #[serde(rename = "parties")]
        parties: Parties,
        #[serde(rename = "districts")]
        pub(crate) districts: Districts,
    }

    #[derive(Debug, PartialEq, Default, Deserialize)]
    #[serde(default)]
    struct Parties {
        #[serde(rename = "countRound")]
        count_rounds: Vec<CountRoundPartiesType>,
    }

    #[derive(Debug, PartialEq, Default, Deserialize)]
    #[serde(default)]
    pub struct Districts {
        //districtsCouncillorSection
        #[serde(rename = "@count")]
        count: xs::UnsignedInt,
        #[serde(rename = "district")]
        pub(crate) districts: Vec<District>,
    }

    #[derive(Debug, PartialEq, Default, Deserialize)]
    #[serde(default)]
    pub struct District {
        //districtsCouncillorSection/District
        #[serde(rename = "@districtName")]
        pub(crate) district_name: xs::Str,
        #[serde(rename = "@enrolment")]
        pub(crate) enrolment: xs::UnsignedInt,

        #[serde(rename = "@lastUpdated")]
        pub(crate) last_updated: xs::Str,
        #[serde(rename = "@final")]
        pub(crate) final_count: Option<xs::Str>,
        #[serde(rename = "@number")]
        pub(crate) number: xs::Byte,
        #[serde(rename = "@votingMethod")]
        pub(crate) voting_method: xs::Str,
        #[serde(rename = "@votingSystem")]
        pub(crate) voting_system: xs::Str,
        #[serde(rename = "@percentRollCounted")]
        pub(crate) percent_counted: Option<xs::Float>,

        #[serde(rename = "numElectedCandidates")]
        pub(crate) num_elected: xs::UnsignedInt,

        #[serde(rename = "candidates")]
        pub(crate) candidates: CandidateSectionType,

        #[serde(rename = "countRound")]
        pub count_rounds: Vec<CountRoundCouncillorType>,
    }
}
