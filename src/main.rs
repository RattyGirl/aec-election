mod aec_parser;
mod database;
mod eml_schema;
mod xml_extension;

use crate::aec_parser::candidate::CandidateList;
use crate::aec_parser::event::ElectionEvent;
use crate::aec_parser::polling::PollingDistrictListStructure;
use crate::database::{CustomDB, MongoDB};
use crate::xml_extension::IgnoreNS;
use minidom::Element;
use mongodb::bson::Bson::ObjectId;
use mongodb::bson::{doc, oid, Document};
use mongodb::sync::Cursor;
use std::io::Read;
use std::str::FromStr;
use std::{env, str};

use suppaftp::FtpStream;
use zip::ZipArchive;

const EML_NAMESPACE: &str = "urn:oasis:names:tc:evs:schema:eml";

fn main() {
    let args: Vec<String> = env::args().collect();
    let option = &args[1];

    let mut ftp_stream: FtpStream = FtpStream::connect("mediafeedarchive.aec.gov.au:21").unwrap();
    ftp_stream.login("anonymous", "").unwrap();
    let database = MongoDB::setup("mongodb://127.0.0.1:27017", "election_history");

    // reset_preload_db(&database);
    reset_lightprogress_db(&database);

    //get all elections
    let election_ids = get_all_in_dir(&mut ftp_stream);
    ftp_stream.quit().unwrap();
    election_ids.into_iter().for_each(|x| {
        if option.eq("preload") {
            preload_data(x.as_str(), &database);
        }
        load_results(x.as_str(), &database);
    });
}

fn reset_lightprogress_db(_database: &impl CustomDB) {}

fn reset_preload_db(database: &impl CustomDB) {
    database
        .list_tables()
        .into_iter()
        .for_each(|x| database.drop::<Document>(x.as_str()));
}

fn load_results(event_id: &str, database: &impl CustomDB) {
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
    let mut polling_string: String = String::new();
    zipfile
        .by_name(format!("xml/aec-mediafeed-pollingdistricts-{}.xml", event_id).as_str())
        .unwrap()
        .read_to_string(&mut polling_string)
        .unwrap();
    get_all_polling_districts(polling_string, database, event_id);
    println!("Gotten all polling districts for event {}", event_id);
}

fn get_all_simple_results(
    results_light_progress: String,
    database: &impl CustomDB,
    event_id: &str,
) {
    let mut results_light_progress: Vec<&str> = results_light_progress.split('\n').collect();
    results_light_progress.remove(0);
    let results_light_progress = results_light_progress.join("\n");
    let root = Element::from_str(results_light_progress.as_str()).unwrap();
    let results_list = aec_parser::results::ResultsMediaFeed::from(&root);

    let election_event_id = results_list.results.event_identifier.id.unwrap_or_default();

    let search: Vec<Document> = database
        .find(
            "election_events",
            doc! {
                "id": election_event_id
            },
        )
        .map(|x| x.unwrap())
        .collect();
    let election_event_id = search
        .first()
        .unwrap()
        .get_object_id("_id")
        .unwrap()
        .to_string();
}

fn get_all_polling_districts(polling_districts: String, database: &impl CustomDB, event_id: &str) {
    let mut polling_districts: Vec<&str> = polling_districts.split('\n').collect();
    polling_districts.remove(0);
    let polling_districts = polling_districts.join("\n");
    let root = Element::from_str(polling_districts.as_str()).unwrap();

    let polling_district_list: PollingDistrictListStructure = root
        .get_child_ignore_ns("PollingDistrictList")
        .unwrap()
        .try_into()
        .unwrap();

    let election_event_id = polling_district_list
        .event_identifier
        .id
        .unwrap_or_default();

    let search: Vec<Document> = database
        .find(
            "election_events",
            doc! {
                "id": election_event_id
            },
        )
        .map(|x| x.unwrap())
        .collect();
    let election_event_id = search
        .first()
        .unwrap()
        .get_object_id("_id")
        .unwrap()
        .to_string();

    polling_district_list
        .polling_districts
        .into_iter()
        .for_each(|district| {
            let district_id = database
                .insert_one("polling_districts", district.clone())
                .unwrap();
            database.many_to_many_connection(
                "polling_district",
                "election_event",
                district_id.as_str(),
                election_event_id.as_str(),
            );
            district
                .polling_places
                .into_iter()
                .for_each(|mut polling_place| {
                    polling_place.district =
                        Some(oid::ObjectId::from_str(district_id.as_str()).unwrap());
                    let polling_place_id = database
                        .insert_one("polling_places", polling_place)
                        .unwrap();
                });
        });
}

fn parse_candidate_preload(candidates_string: String, database: &impl CustomDB, event_id: &str) {
    let mut candidates_string: Vec<&str> = candidates_string.split('\n').collect();
    candidates_string.remove(0);
    let candidates_string = candidates_string.join("\n");
    let root: Element = candidates_string.parse().unwrap();
    let candidate_list: CandidateList = root
        .get_child("CandidateList", EML_NAMESPACE)
        .unwrap()
        .try_into()
        .unwrap();

    let election_event_id = candidate_list.event_identifier.id.unwrap_or_default();

    candidate_list.elections.into_iter().for_each(|election| {
        let election_id = election.election_identifier.id;
        election.contests.into_iter().for_each(|contest| {
            let contest_id = contest.contest_identifier.id;
            contest.candidates.into_iter().for_each(|candidate| {
                //TODO affiliation linking
                let candidate_id = database.insert_one("candidates", candidate.clone());
                match candidate.affiliation {
                    None => {}
                    Some(affiliation) => {
                        let search: Cursor<Document> = database.find(
                            "affiliations",
                            doc! {
                                "id": affiliation.clone().affiliation_identifier.id.unwrap().0
                            },
                        );
                        let list: Vec<Document> = search.map(|x| x.unwrap()).collect();
                        let affiliation_id = if list.is_empty() {
                            database
                                .insert_one("affiliations", affiliation.affiliation_identifier)
                                .unwrap()
                        } else {
                            list.first()
                                .unwrap()
                                .get_object_id("_id")
                                .unwrap()
                                .to_string()
                        };
                        //connect affiliation
                        database.many_to_many_connection(
                            "candidate",
                            "affiliation",
                            candidate_id.unwrap().as_str(),
                            affiliation_id.as_str(),
                        );
                    }
                }
            })
        });
    });
}

fn parse_event_preload(events_string: String, database: &impl CustomDB, event_id: &str) {
    let mut events_string: Vec<&str> = events_string.split('\n').collect();
    events_string.remove(0);
    let events_string = events_string.join("\n");
    let root: Element = events_string.parse().unwrap();
    let election_event: ElectionEvent = root
        .get_child("ElectionEvent", EML_NAMESPACE)
        .unwrap()
        .try_into()
        .unwrap();

    let election_event_id = database
        .insert_one("election_events", election_event.event_identifier.clone())
        .unwrap_or_default();

    election_event.elections.into_iter().for_each(|election| {
        let election_id = database
            .insert_one("elections", election.clone())
            .unwrap_or_default();

        database.many_to_many_connection(
            "election_event",
            "election",
            election_event_id.as_str(),
            election_id.as_str(),
        );

        election.contests.into_iter().for_each(|contest| {
            let contest_id = database.insert_one("contests", contest).unwrap_or_default();

            database.insert_one(
                "election_contests",
                doc! {
                    "election": ObjectId(oid::ObjectId::from_str(election_id.as_str()).unwrap()),
                    "contests": ObjectId(oid::ObjectId::from_str(contest_id.as_str()).unwrap()),
                },
            );
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
