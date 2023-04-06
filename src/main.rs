use minidom::Element;
use mongodb::options::ClientOptions;
use mongodb::sync::Database;
use mongodb::{bson::doc, sync::Client, sync::Collection};
use serde::{Deserialize, Serialize};
use std::io::Read;
use std::str;
use suppaftp::FtpStream;
use zip::ZipArchive;

const NAMESPACE: &str = "urn:oasis:names:tc:evs:schema:eml";

fn main() {
    let mut ftp_stream: FtpStream = FtpStream::connect("mediafeedarchive.aec.gov.au:21").unwrap();
    let _ = ftp_stream.login("anonymous", "").unwrap();

    reset_db();

    //get all elections
    let election_ids = get_all_in_dir(&mut ftp_stream);
    ftp_stream.quit().unwrap();

    election_ids
        .into_iter()
        .for_each(|x| preload_data(x.as_str()));
}

fn reset_db() {
    let mut client_options = ClientOptions::parse("mongodb://127.0.0.1:27017").unwrap();
    client_options.app_name = Some("AEC Election History".to_string());
    let client = Client::with_options(client_options).unwrap();
    client.database("election_history").drop(None).unwrap();
}

fn preload_data(event_id: &str) {
    let mut ftp_stream: FtpStream = FtpStream::connect("mediafeedarchive.aec.gov.au:21").unwrap();
    let _ = ftp_stream.login("anonymous", "").unwrap();

    let mut client_options = ClientOptions::parse("mongodb://127.0.0.1:27017").unwrap();
    client_options.app_name = Some("AEC Election History".to_string());
    let client = Client::with_options(client_options).unwrap();
    let database = client.database("election_history");

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
    get_all_races(events_string, &database, event_id);
    println!("Gotten all races for event {}", event_id);

    let mut candidates_string: String = String::new();
    zipfile
        .by_name(format!("xml/eml-230-candidates-{}.xml", event_id).as_str())
        .unwrap()
        .read_to_string(&mut candidates_string)
        .unwrap();
    get_all_candidates(candidates_string, &database, event_id);
    println!("Gotten all candidates for event {}", event_id);
}

fn get_all_candidates(candidates_string: String, mongodb: &Database, event_id: &str) {
    let mut candidates_string: Vec<&str> = candidates_string.split("\n").collect();
    candidates_string.remove(0);
    let candidates_string = candidates_string.join("\n");
    let root: Element = candidates_string.parse().unwrap();
    let candidate_list = root.get_child("CandidateList", NAMESPACE).unwrap();

    let elections = candidate_list
        .children()
        .filter(|f| f.name().eq("Election"));
    elections.for_each(|election| {
        let election_id = election
            .get_child("ElectionIdentifier", NAMESPACE)
            .unwrap()
            .attr("Id")
            .unwrap();

        election
            .children()
            .filter(|f| f.name().eq("Contest"))
            .for_each(|contest| {
                let contest_id = contest
                    .get_child("ContestIdentifier", NAMESPACE)
                    .unwrap()
                    .attr("Id")
                    .unwrap();

                contest
                    .children()
                    .filter(|f| f.name().eq("Candidate"))
                    .for_each(|candidate| {
                        let candidate_id = candidate
                            .get_child("CandidateIdentifier", NAMESPACE)
                            .unwrap()
                            .attr("Id")
                            .unwrap();
                        let candidate_name = candidate
                            .get_child("CandidateIdentifier", NAMESPACE)
                            .unwrap()
                            .get_child("CandidateName", NAMESPACE)
                            .unwrap()
                            .text();
                        let candidate_profession =
                            candidate.get_child("Profession", NAMESPACE).unwrap().text();

                        mongodb
                            .collection("candidates")
                            .insert_one(
                                Candidate {
                                    id: candidate_id.to_string(),
                                    event_id: event_id.to_string(),
                                    election_id: election_id.to_string(),
                                    contest_id: contest_id.to_string(),
                                    name: candidate_name.to_string(),
                                    profession: candidate_profession.to_string(),
                                },
                                None,
                            )
                            .unwrap();
                    });
            });
    })
}

fn get_all_races(events_string: String, mongodb: &Database, event_id: &str) {
    let mut events_string: Vec<&str> = events_string.split("\n").collect();
    events_string.remove(0);
    let events_string = events_string.join("\n");
    let root: Element = events_string.parse().unwrap();
    let mut iter = root.children();
    let election_event = root.get_child("ElectionEvent", NAMESPACE).unwrap();
    let event_identifier = election_event
        .get_child("EventIdentifier", NAMESPACE)
        .unwrap();

    let name = event_identifier
        .get_child("EventName", NAMESPACE)
        .unwrap()
        .text();

    mongodb
        .collection("election_events")
        .insert_one(
            ElectionEvent {
                id: event_id.to_string(),
                name: name.to_string(),
            },
            None,
        )
        .unwrap();

    let elections = election_event
        .children()
        .filter(|f| f.name().eq("Election"));
    elections.for_each(|election| {
        let election_id = election
            .get_child("ElectionIdentifier", NAMESPACE)
            .unwrap()
            .attr("Id")
            .unwrap();
        let date = election
            .get_child("Date", NAMESPACE)
            .unwrap()
            .get_child("SingleDate", NAMESPACE)
            .unwrap()
            .text();
        let name = election
            .get_child("ElectionIdentifier", NAMESPACE)
            .unwrap()
            .get_child("ElectionName", NAMESPACE)
            .unwrap()
            .text();
        let category = election
            .get_child("ElectionIdentifier", NAMESPACE)
            .unwrap()
            .get_child("ElectionCategory", NAMESPACE)
            .unwrap()
            .text();

        mongodb
            .collection("elections")
            .insert_one(
                Election {
                    id: election_id.to_string(),
                    event_id: event_id.to_string(),
                    date: date.to_string(),
                    name: name.to_string(),
                    category: category.to_string(),
                },
                None,
            )
            .unwrap();

        election
            .children()
            .filter(|f| f.name().eq("Contest"))
            .for_each(|contest| {
                let contest_id = contest
                    .get_child("ContestIdentifier", NAMESPACE)
                    .unwrap()
                    .attr("Id")
                    .unwrap_or("");
                let short_code = contest
                    .get_child("ContestIdentifier", NAMESPACE)
                    .unwrap()
                    .attr("ShortCode")
                    .unwrap_or("");
                let name = contest
                    .get_child("ContestIdentifier", NAMESPACE)
                    .unwrap()
                    .get_child("ContestName", NAMESPACE)
                    .unwrap()
                    .text();
                let position = contest.get_child("Position", NAMESPACE).unwrap().text();
                let number = contest
                    .get_child("NumberOfPositions", NAMESPACE)
                    .unwrap()
                    .text();


                mongodb
                    .collection("contests")
                    .insert_one(
                        Contest {
                            id: contest_id.to_string(),
                            event_id: event_id.to_string(),
                            election_id: election_id.to_string(),
                            short_code: short_code.to_string(),
                            name: name.to_string(),
                            position: position.to_string(),
                            number: number.to_string(),
                        },
                        None,
                    )
                    .unwrap();
            });
    });
}

fn change_dir(mut stream: &mut FtpStream, x: &str) {
    stream.cwd(x).unwrap();
    println!("Current directory: {}", stream.pwd().unwrap());
}

fn get_all_in_dir(mut ftp_stream: &mut FtpStream) -> Vec<String> {
    ftp_stream
        .list(None)
        .unwrap()
        .into_iter()
        .map(|row| row.split(" ").last().unwrap_or("").to_string())
        .collect::<Vec<_>>()
}

#[derive(Debug, Serialize, Deserialize)]
struct ElectionEvent {
    id: String,
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
    id: String,
    event_id: String,
    election_id: String,
    contest_id: String,
    name: String,
    profession: String,
}
