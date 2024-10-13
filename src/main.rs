#![allow(warnings)]
mod aec_parser;
mod election_server;
mod eml_schema;
mod qld_election;
mod xml_extension;

use election::sql_db::MySQLDB;

use crate::aec_parser::event::ElectionEvent;
use crate::election_server::ElectionServer;
use minidom::Element;
use std::io::Read;

use crate::qld_election::read_result;
use crate::xml_extension::IgnoreNS;
use std::str::FromStr;
use std::{env, io, str};
use zip::ZipArchive;

const EML_NAMESPACE: &str = "urn:oasis:names:tc:evs:schema:eml";
const AEC_SERVER: &str = "mediafeedarchive.aec.gov.au:21";

#[tokio::main]
async fn main() {
    let mut database = MySQLDB::setup("postgresql://aec@localhost/aec", "aec").await;
    read_result(&mut database).await;

    // let mut aec_server = ElectionServer::new(AEC_SERVER);
    // election::election_models::drop_tables(&mut database).await;
    // election::election_models::generate_tables(&mut database).await;
    //
    // //get all elections
    // let election_ids_list = aec_server.get_all_in_dir();
    // // let election_details: Vec<(String, String)> = election_ids_list.iter().map(|x| get_event_identifier(x.to_string())).collect();
    // println!("{:?}", &election_ids_list);
    // //which election?
    // let election_id = "27966";
    // aec_server.quit();
}

fn aec() {}

fn get_event_identifier(event_id: String) -> (String, String) {
    let mut aec_server = ElectionServer::new(AEC_SERVER);
    aec_server.cwd(event_id.as_str());
    aec_server.cwd("Detailed/Preload");
    let latest = aec_server.get_all_in_dir().last().unwrap().clone();
    let zipfile = aec_server.get_zip(latest.as_str());
    let mut zipfile = ZipArchive::new(zipfile).unwrap();
    let mut results_string: String = String::new();
    zipfile
        .by_name(format!("xml/eml-110-event-{}.xml", event_id).as_str())
        .unwrap()
        .read_to_string(&mut results_string)
        .unwrap();
    let mut results_string: Vec<&str> = results_string.split('\n').collect();
    results_string.remove(0);
    let results_string = results_string.join("\n");
    let root = Element::from_str(results_string.as_str()).unwrap();

    let event_name = root
        .get_child_ignore_ns("ElectionEvent")
        .unwrap()
        .get_child_ignore_ns("EventIdentifier")
        .unwrap()
        .get_child_ignore_ns("EventName")
        .unwrap()
        .text();

    (event_id, event_name)
}

fn preload_data(event_id: &str, database: &MySQLDB) {
    println!("Starting {}", event_id);
    let mut aec_server = ElectionServer::new("mediafeedarchive.aec.gov.au:21");

    aec_server.cwd(event_id);
    aec_server.cwd("Detailed/Preload");
    let latest = aec_server.get_all_in_dir().last().unwrap().clone();
    let mut zipfile = ZipArchive::new(aec_server.get_zip(latest.as_str())).unwrap();

    let mut events_string: String = String::new();
    zipfile
        .by_name(format!("xml/eml-110-event-{}.xml", event_id).as_str())
        .unwrap()
        .read_to_string(&mut events_string)
        .unwrap();
    parse_event_preload(events_string, database, event_id);

    // let mut candidates_string: String = String::new();
    // zipfile
    //     .by_name(format!("xml/eml-230-candidates-{}.xml", event_id).as_str())
    //     .unwrap()
    //     .read_to_string(&mut candidates_string)
    //     .unwrap();
    // parse_candidate_preload(candidates_string, database, event_id);
    // let mut polling_string: String = String::new();
    // zipfile
    //     .by_name(format!("xml/aec-mediafeed-pollingdistricts-{}.xml", event_id).as_str())
    //     .unwrap()
    //     .read_to_string(&mut polling_string)
    //     .unwrap();
    // get_all_polling_districts(polling_string, database, event_id);
    // println!("Gotten all polling districts for event {}", event_id);
}

fn parse_event_preload(events_string: String, database: &MySQLDB, _event_id: &str) {
    let mut events_string: Vec<&str> = events_string.split('\n').collect();

    //remove xml start string
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
            let _contest_id = database.insert_one("contests", contest).unwrap_or_default();

            // database.insert_one(
            //     "election_contests",
            //     ElectionContests {
            //         election: ObjectId(oid::ObjectId::from_str(election_id.as_str()).unwrap())
            //             .as_str()
            //             .unwrap(),
            //         contests: ObjectId(oid::ObjectId::from_str(contest_id.as_str()).unwrap())
            //             .as_str()
            //             .unwrap(),
            //     },
            // );
        });
    })
}
