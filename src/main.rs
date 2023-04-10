mod aec_parser;
mod database;
mod xml_extension;

use crate::aec_parser::{CandidateListMessage, ElectionEventMessage, EventIdentifierStructure};
use crate::database::{CustomDB, MongoDB};
use crate::xml_extension::IgnoreNS;
use minidom::Element;
use mongodb::bson::Bson::ObjectId;
use mongodb::bson::{bson, doc, oid, Bson, Document};
use serde::{Deserialize, Serialize};
use std::io::Read;
use std::str;
use std::str::FromStr;
use suppaftp::FtpStream;
use zip::ZipArchive;

const EML_NAMESPACE: &str = "urn:oasis:names:tc:evs:schema:eml";

fn main() {
    let mut ftp_stream: FtpStream = FtpStream::connect("mediafeedarchive.aec.gov.au:21").unwrap();
    ftp_stream.login("anonymous", "").unwrap();
    let database = MongoDB::setup("mongodb://127.0.0.1:27017", "election_history");

    reset_preload_db(&database);
    reset_lightprogress_db(&database);

    //get all elections
    let election_ids = get_all_in_dir(&mut ftp_stream);
    ftp_stream.quit().unwrap();

    election_ids.into_iter().for_each(|x| {
        preload_data(x.as_str(), &database);
        load_results(x.as_str(), &database);
    });
}

fn reset_lightprogress_db(_database: &impl CustomDB) {}

fn reset_preload_db(database: &impl CustomDB) {
    database.drop::<Document>("candidates");
    database.drop::<Document>("contests");
    database.drop::<Document>("election_events");
    database.drop::<Document>("elections");
    database.drop::<Document>("polling_district_list");
    database.drop::<Document>("affiliations");

    //relationships
    database.drop::<Document>("election_events_elections");
    database.drop::<Document>("election_contests");
}

fn load_results(event_id: &str, database: &impl CustomDB) {
    println!("Starting {}", event_id);
    let mut ftp_stream: FtpStream = FtpStream::connect("mediafeedarchive.aec.gov.au:21").unwrap();
    ftp_stream.login("anonymous", "").unwrap();

    ftp_stream.cwd(event_id).unwrap();
    ftp_stream.cwd("Detailed/LightProgress").unwrap();
    let latest = get_all_in_dir(&mut ftp_stream).last().unwrap().clone();
    let file = ftp_stream.retr_as_buffer(latest.as_str()).unwrap();
    let mut zipfile = ZipArchive::new(file).unwrap();

    let mut results_string: String = String::new();
    zipfile
        .by_name(
            format!(
                "xml/aec-mediafeed-results-detailed-lightprogress-{}.xml",
                event_id
            )
            .as_str(),
        )
        .unwrap()
        .read_to_string(&mut results_string)
        .unwrap();
    get_all_simple_results(results_string, database, event_id);
    println!("Gotten all races for event {}", event_id);
}

fn preload_data(event_id: &str, database: &impl CustomDB) {
    println!("Starting {}", event_id);
    let mut ftp_stream: FtpStream = FtpStream::connect("mediafeedarchive.aec.gov.au:21").unwrap();
    ftp_stream.login("anonymous", "").unwrap();

    ftp_stream.cwd(event_id).unwrap();
    ftp_stream.cwd("Detailed/Preload").unwrap();
    let latest = get_all_in_dir(&mut ftp_stream).last().unwrap().clone();
    let file = ftp_stream.retr_as_buffer(latest.as_str()).unwrap();
    let mut zipfile = ZipArchive::new(file).unwrap();

    let mut events_string: String = String::new();
    zipfile
        .by_name(format!("xml/eml-110-event-{}.xml", event_id).as_str())
        .unwrap()
        .read_to_string(&mut events_string)
        .unwrap();
    parse_event_preload(events_string, database, event_id);
    let mut candidates_string: String = String::new();
    zipfile
        .by_name(format!("xml/eml-230-candidates-{}.xml", event_id).as_str())
        .unwrap()
        .read_to_string(&mut candidates_string)
        .unwrap();
    parse_candidate_preload(candidates_string, database, event_id);
    // let mut polling_string: String = String::new();
    // zipfile
    //     .by_name(format!("xml/aec-mediafeed-pollingdistricts-{}.xml", event_id).as_str())
    //     .unwrap()
    //     .read_to_string(&mut polling_string)
    //     .unwrap();
    // get_all_polling_districts(polling_string, database, event_id);
    // println!("Gotten all polling districts for event {}", event_id);
}

fn get_all_simple_results(
    results_light_progress: String,
    database: &impl CustomDB,
    event_id: &str,
) {
    let mut results_light_progress: Vec<&str> = results_light_progress.split('\n').collect();
    results_light_progress.remove(0);
    let results_light_progress = results_light_progress.join("\n");
    // let root = Element::from_str(results_light_progress.as_str()).unwrap();
    // let candidate_list = root.get_child_ignore_ns("Results").unwrap();
}

fn parse_candidate_preload(candidates_string: String, database: &impl CustomDB, event_id: &str) {
    //
    // let election_event_id = database
    //     .insert_one("election_events", election_event.event_identifier.clone())
    //     .unwrap_or("".to_string());

    let mut candidates_string: Vec<&str> = candidates_string.split('\n').collect();
    candidates_string.remove(0);
    let candidates_string = candidates_string.join("\n");
    let root: Element = candidates_string.parse().unwrap();
    let candidate_list: CandidateListMessage = root
        .get_child("CandidateList", EML_NAMESPACE)
        .unwrap()
        .try_into()
        .unwrap();

    let election_event_id = candidate_list.event_identifier.id.unwrap_or("".to_string());

    candidate_list.elections.into_iter().for_each(|election| {
        let election_id = election.election_identifier.id;
        election.contests.into_iter().for_each(|contest| {
            let contest_id = contest.contest_identifier.id;
            contest.candidates.into_iter().for_each(|candidate| {
                let candidate_id = database.insert_one(
                    "candidates", candidate
                );
            })
        });
    });
        //
        //                         database.insert_one(
        //                             "candidates",
        //                             Candidate {
        //                                 id: candidate_id.parse().unwrap(),
        //                                 event_id: event_id.parse().unwrap(),
        //                                 election_id: election_id.to_string(),
        //                                 contest_id: contest_id.to_string(),
        //                                 affiliation_id,
        //                                 name: candidate_name,
        //                                 profession: candidate_profession,
        //                                 gender: candidate_gender,
        //                             },
        //                         );
        //                     });
        //             });
        //     })
}

fn parse_event_preload(events_string: String, database: &impl CustomDB, event_id: &str) {
    let mut events_string: Vec<&str> = events_string.split('\n').collect();
    events_string.remove(0);
    let events_string = events_string.join("\n");
    let root: Element = events_string.parse().unwrap();
    let election_event: ElectionEventMessage = root
        .get_child("ElectionEvent", EML_NAMESPACE)
        .unwrap()
        .try_into()
        .unwrap();

    let election_event_id = database
        .insert_one("election_events", election_event.event_identifier.clone())
        .unwrap_or("".to_string());

    election_event.elections.into_iter().for_each(|election| {
        let election_id = database.insert_one("elections", election.clone()).unwrap_or("".to_string());

        database.insert_one("election_events_elections", doc! {
            "election_event": Bson::ObjectId(oid::ObjectId::from_str(election_event_id.as_str()).unwrap()),
            "election": ObjectId(oid::ObjectId::from_str(election_id.as_str()).unwrap()),
        });

        election.contests.into_iter().for_each(|contest| {
            let contest_id = database.insert_one("contests", contest).unwrap_or("".to_string());

            database.insert_one("election_contests", doc! {
                "election": ObjectId(oid::ObjectId::from_str(election_id.as_str()).unwrap()),
                "contests": ObjectId(oid::ObjectId::from_str(contest_id.as_str()).unwrap()),
            });
        });

    })
}

fn get_all_in_dir(ftp_stream: &mut FtpStream) -> Vec<String> {
    ftp_stream
        .list(None)
        .unwrap()
        .into_iter()
        .map(|row| row.split(' ').last().unwrap_or("").to_string())
        .collect::<Vec<_>>()
}

#[derive(Debug, Serialize, Deserialize)]
struct ElectionEventSerialize {
    id: i32,
    name: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Election {
    id: String,
    event_id: String,
    date: String,
    name: String,
    category: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Contest {
    id: String,
    event_id: String,
    election_id: String,
    short_code: String,
    name: String,
    position: String,
    number: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Candidate {
    id: i32,
    event_id: i32,
    election_id: String,
    contest_id: String,
    affiliation_id: Option<i32>,
    name: String,
    profession: String,
    gender: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct PollingDistrict {
    id: String,
    event_id: String,
    short_code: String,
    name: String,
    state_id: String,
    derivation: String,
    products_industry: String,
    location: String,
    demographic: String,
    area: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Affiliation {
    id: i32,
    short_code: String,
    registered_name: String,
    affiliation_type: String,
}

impl Affiliation {
    fn from(element: &Element) -> Self {
        Self {
            id: element
                .get_child_ignore_ns("AffiliationIdentifier")
                .unwrap()
                .attr("Id")
                .unwrap()
                .parse()
                .unwrap(),
            short_code: element
                .get_child_ignore_ns("AffiliationIdentifier")
                .unwrap()
                .attr("ShortCode")
                .unwrap()
                .to_string(),
            registered_name: element
                .get_child_ignore_ns("AffiliationIdentifier")
                .unwrap()
                .get_child_ignore_ns("RegisteredName")
                .unwrap()
                .text(),
            affiliation_type: element
                .get_child_ignore_ns("Type")
                .unwrap_or(&Element::bare("", ""))
                .text(),
        }
    }
}

impl PollingDistrict {
    fn from(element: &Element, event_id: &str) -> Self {
        Self {
            id: element
                .get_child_ignore_ns("PollingDistrictIdentifier")
                .unwrap()
                .attr("Id")
                .unwrap()
                .to_string(),
            event_id: event_id.to_string(),
            short_code: element
                .get_child_ignore_ns("PollingDistrictIdentifier")
                .unwrap()
                .attr("ShortCode")
                .unwrap()
                .to_string(),
            name: element
                .get_child_ignore_ns("PollingDistrictIdentifier")
                .unwrap()
                .get_child_ignore_ns("Name")
                .unwrap()
                .text(),
            state_id: element
                .get_child_ignore_ns("PollingDistrictIdentifier")
                .unwrap()
                .get_child_ignore_ns("StateIdentifier")
                .unwrap()
                .attr("Id")
                .unwrap()
                .to_string(),
            derivation: element
                .get_child_ignore_ns("NameDerivation")
                .unwrap()
                .text(),
            products_industry: element
                .get_child_ignore_ns("ProductsIndustry")
                .unwrap()
                .text(),
            location: element.get_child_ignore_ns("Location").unwrap().text(),
            demographic: element.get_child_ignore_ns("Demographic").unwrap().text(),
            area: element.get_child_ignore_ns("Area").unwrap().text(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct PollingPlace {}
