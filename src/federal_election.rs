use crate::election_server::ElectionServer;
use crate::xml_extension::IgnoreNS;
use crate::AEC_SERVER;
use election::election_models::drop_tables;
use election::sql_db::MySQLDB;
use election::PostGresObj;
use election_derive::PostGresObj;
use futures::poll;
use minidom::Element;
use postgres::types::ToSql;
use quick_xml::Reader;
use std::fmt::format;
use std::io::{Cursor, Read};
use std::str::FromStr;
use zip::ZipArchive;

pub async fn get_preload(db: &mut MySQLDB) {
    let mut aec_server = ElectionServer::new(AEC_SERVER);
    create_tables(db).await;
    //get all elections
    let election_ids_list = aec_server.get_all_in_dir();
    // let election_details: Vec<(String, String)> = election_ids_list.iter().map(|x| get_event_identifier(x.to_string())).collect();
    println!("{:?}", &election_ids_list);
    //which election?
    for election_id in election_ids_list {
        println!("Completing election {}", election_id);
        // aec_server.cwd("../../../../");
        // aec_server.cwd(&election_id);
        // aec_server.cwd("Detailed/Preload");
        // let mut files = aec_server.get_all_in_dir();
        // files.sort();
        // let latest = files.last().unwrap();
        // let zipfile = aec_server.get_zip(latest.as_str());
        // let mut zipfile = ZipArchive::new(zipfile).unwrap();
        // read_aec_pollingdistricts(&election_id, db, &mut zipfile).await;
        // read_eml_110_event_xml(&election_id, db, &mut zipfile).await;
        // read_eml_230_candidates(&election_id, db, &mut zipfile).await;
        // // read_results_preload(&election_id, db, &mut zipfile).await;
        aec_server.cwd("../../../../");
        aec_server.cwd(&election_id);
        aec_server.cwd("Detailed/Light");
        let mut files = aec_server.get_all_in_dir();
        files.sort();
        let latest = files.last().unwrap();
        let zipfile = aec_server.get_zip(latest.as_str());
        let mut zipfile = ZipArchive::new(zipfile).unwrap();
        read_aec_light(&election_id, db, &mut zipfile).await;
        println!("Completed election {}", election_id);
    }
}

async fn read_aec_light(
    election_id: &String,
    db: &mut MySQLDB,
    mut zipfile: &mut ZipArchive<Cursor<Vec<u8>>>,
) {
    let mut results_string: String = String::new();
    zipfile
        .by_name(
            format!(
                "xml/aec-mediafeed-results-detailed-light-{}.xml",
                election_id
            )
            .as_str(),
        )
        .unwrap()
        .read_to_string(&mut results_string)
        .unwrap();
    let mut results_string: Vec<&str> = results_string.split('\n').collect();
    results_string.remove(0);
    let results_string = results_string.join("\n");

    let rootnode: Element = Element::from_str(results_string.as_str()).unwrap();
    for election in rootnode
        .get_child_ignore_ns("Results")
        .unwrap()
        .get_children_ignore_ns("Election")
    {
        if let Some(house) = election.get_child_ignore_ns("House") {
            //house election
            let contests = house
                .get_child_ignore_ns("Contests")
                .unwrap()
                .get_children_ignore_ns("Contest");
            for contest in contests {
                let contest_id = contest
                    .get_child_ignore_ns("ContestIdentifier")
                    .unwrap()
                    .attr("Id")
                    .unwrap()
                    .to_string();
                for polling_place in contest
                    .get_child_ignore_ns("PollingPlaces")
                    .unwrap()
                    .get_children_ignore_ns("PollingPlace")
                {
                    {
                        let updated_timestamp = polling_place.get_child_ignore_ns("FirstPreferences").unwrap().attr("Updated").unwrap_or("").to_string();
                        let primary_candidate_vec: Vec<String> = polling_place
                            .get_child_ignore_ns("FirstPreferences")
                            .unwrap()
                            .get_children_ignore_ns("Candidate")
                            .iter()
                            .map(|candidate| {
                                let vote_struct = PrimaryVote {
                                    timestamp: updated_timestamp.clone(),
                                    event_id: election_id.clone(),
                                    contest_id: contest_id.clone(),
                                    polling_place_id: polling_place
                                        .get_child_ignore_ns("PollingPlaceIdentifier")
                                        .unwrap()
                                        .attr("Id")
                                        .unwrap()
                                        .to_string(),
                                    candidate_id: candidate
                                        .get_child_ignore_ns("CandidateIdentifier")
                                        .unwrap()
                                        .attr("Id")
                                        .unwrap()
                                        .to_string(),
                                    votes: candidate
                                        .get_child_ignore_ns("Votes")
                                        .unwrap()
                                        .text()
                                        .parse()
                                        .unwrap(),
                                };
                                format!(
                                    "($${}$$, $${}$$, $${}$$, $${}$$, $${}$$, $${}$$)",
                                    vote_struct.timestamp,
                                    vote_struct.event_id,
                                    vote_struct.contest_id,
                                    vote_struct.polling_place_id,
                                    vote_struct.candidate_id,
                                    vote_struct.votes
                                )
                            })
                            .collect();
                        let query = format!("INSERT INTO aec.primaryvote (timestamp, event_id, contest_id, polling_place_id, candidate_id, votes) VALUES {}",
                                            primary_candidate_vec.join(", "));
                        &db.run_raw(query).await;
                    }

                    let updated_timestamp = polling_place.get_child_ignore_ns("TwoCandidatePreferred").unwrap().attr("Updated").unwrap_or("").to_string();
                    let two_candidate_vec: Vec<String> = polling_place
                        .get_child_ignore_ns("TwoCandidatePreferred")
                        .unwrap()
                        .get_children_ignore_ns("Candidate")
                        .iter()
                        .map(|candidate| {
                            let vote_struct = TwoCandidatePreferredVote {
                                timestamp: updated_timestamp.clone(),
                                event_id: election_id.clone(),
                                contest_id: contest_id.clone(),
                                polling_place_id: polling_place
                                    .get_child_ignore_ns("PollingPlaceIdentifier")
                                    .unwrap()
                                    .attr("Id")
                                    .unwrap()
                                    .to_string(),
                                candidate_id: candidate
                                    .get_child_ignore_ns("CandidateIdentifier")
                                    .unwrap()
                                    .attr("Id")
                                    .unwrap()
                                    .to_string(),
                                votes: candidate
                                    .get_child_ignore_ns("Votes")
                                    .unwrap()
                                    .text()
                                    .parse()
                                    .unwrap(),
                            };
                            format!(
                                "($${}$$, $${}$$, $${}$$, $${}$$, $${}$$, $${}$$)",
                                vote_struct.timestamp,
                                vote_struct.event_id,
                                vote_struct.contest_id,
                                vote_struct.polling_place_id,
                                vote_struct.candidate_id,
                                vote_struct.votes
                            )
                        })
                        .collect();
                    let query = format!("INSERT INTO aec.twocandidatepreferredvote (timestamp, event_id, contest_id, polling_place_id, candidate_id, votes) VALUES {}",
                                        two_candidate_vec.join(", "));
                    &db.run_raw(query).await;
                }
            }

            //end house election
        } else if let Some(senate) = election.get_child_ignore_ns("Senate") {
        } else if let Some(referendum) = election.get_child_ignore_ns("Referendum") {
        }
    }
}

async fn read_results_preload(
    election_id: &String,
    db: &mut MySQLDB,
    mut zipfile: &mut ZipArchive<Cursor<Vec<u8>>>,
) {
    let mut results_string: String = String::new();
    zipfile
        .by_name(
            format!(
                "xml/aec-mediafeed-results-detailed-preload-{}.xml",
                election_id
            )
            .as_str(),
        )
        .unwrap()
        .read_to_string(&mut results_string)
        .unwrap();
    let mut results_string: Vec<&str> = results_string.split('\n').collect();
    results_string.remove(0);
    let results_string = results_string.join("\n");

    let rootnode: Element = Element::from_str(results_string.as_str()).unwrap();
    for election in rootnode
        .get_child_ignore_ns("Results")
        .unwrap()
        .get_children_ignore_ns("Election")
    {
        if let Some(house) = election.get_child_ignore_ns("House") {
            //house election
            let contests = house
                .get_child_ignore_ns("Contests")
                .unwrap()
                .get_children_ignore_ns("Contest");
            for contest in contests {
                let contest_id = contest
                    .get_child_ignore_ns("ContestIdentifier")
                    .unwrap()
                    .attr("Id")
                    .unwrap()
                    .to_string();
                for polling_place in contest
                    .get_child_ignore_ns("PollingPlaces")
                    .unwrap()
                    .get_children_ignore_ns("PollingPlace")
                {
                    let candidate_vec: Vec<String> = polling_place
                        .get_child_ignore_ns("FirstPreferences")
                        .unwrap()
                        .get_children_ignore_ns("Candidate")
                        .iter()
                        .map(|candidate| {
                            let vote_struct = PrimaryVote {
                                timestamp: "".to_string(),
                                event_id: election_id.clone(),
                                contest_id: contest_id.clone(),
                                polling_place_id: polling_place
                                    .get_child_ignore_ns("PollingPlaceIdentifier")
                                    .unwrap()
                                    .attr("Id")
                                    .unwrap()
                                    .to_string(),
                                candidate_id: candidate
                                    .get_child_ignore_ns("CandidateIdentifier")
                                    .unwrap()
                                    .attr("Id")
                                    .unwrap()
                                    .to_string(),
                                votes: candidate
                                    .get_child_ignore_ns("Votes")
                                    .unwrap()
                                    .text()
                                    .parse()
                                    .unwrap(),
                            };
                            format!(
                                "($${}$$, $${}$$, $${}$$, $${}$$, $${}$$)",
                                vote_struct.event_id,
                                vote_struct.contest_id,
                                vote_struct.polling_place_id,
                                vote_struct.candidate_id,
                                vote_struct.votes
                            )
                        })
                        .collect();
                    let query = format!("INSERT INTO aec.primaryvote (event_id, contest_id, polling_place_id, candidate_id, votes) VALUES {}",
                                        candidate_vec.join(", "));
                    &db.run_raw(query).await;
                }
            }

            //end house election
        } else if let Some(senate) = election.get_child_ignore_ns("Senate") {
        } else if let Some(referendum) = election.get_child_ignore_ns("Referendum") {
        }
    }
}

async fn read_aec_pollingdistricts(
    election_id: &String,
    db: &mut MySQLDB,
    mut zipfile: &mut ZipArchive<Cursor<Vec<u8>>>,
) {
    let mut results_string: String = String::new();
    zipfile
        .by_name(format!("xml/aec-mediafeed-pollingdistricts-{}.xml", election_id).as_str())
        .unwrap()
        .read_to_string(&mut results_string)
        .unwrap();
    let mut results_string: Vec<&str> = results_string.split('\n').collect();
    results_string.remove(0);
    let results_string = results_string.join("\n");

    let rootnode: Element = Element::from_str(results_string.as_str()).unwrap();
    let pollingplace_structs: Vec<String> = rootnode
        .get_child_ignore_ns("PollingDistrictList")
        .unwrap()
        .get_children_ignore_ns("PollingDistrict")
        .iter()
        .map(|districtelement| {
            let district_id = districtelement
                .get_child_ignore_ns("PollingDistrictIdentifier")
                .unwrap()
                .attr("Id")
                .unwrap()
                .to_string();
            let pollingplace_structs: Vec<String> = districtelement
                .get_child_ignore_ns("PollingPlaces")
                .unwrap()
                .get_children_ignore_ns("PollingPlace")
                .iter()
                .map(|pollingplace_element| {
                    let pollingplacestruct = PollingPlace {
                        event_id: election_id.to_string(),
                        district_id: district_id.clone(),
                        id: pollingplace_element
                            .get_child_ignore_ns("PollingPlaceIdentifier")
                            .unwrap()
                            .attr("Id")
                            .unwrap()
                            .to_string(),
                        name: pollingplace_element
                            .get_child_ignore_ns("PollingPlaceIdentifier")
                            .unwrap()
                            .attr("Name")
                            .unwrap()
                            .to_string(),
                        lat: pollingplace_element
                            .get_child_ignore_ns("PhysicalLocation")
                            .unwrap()
                            .get_child_ignore_ns("Address")
                            .unwrap()
                            .get_child_ignore_ns("PostalServiceElements")
                            .unwrap()
                            .get_child_ignore_ns("AddressLatitude")
                            .unwrap()
                            .text(),
                        long: pollingplace_element
                            .get_child_ignore_ns("PhysicalLocation")
                            .unwrap()
                            .get_child_ignore_ns("Address")
                            .unwrap()
                            .get_child_ignore_ns("PostalServiceElements")
                            .unwrap()
                            .get_child_ignore_ns("AddressLongitude")
                            .unwrap()
                            .text(),
                    };
                    // let query = format!("INSERT INTO aec.pollingplace (event_id, district_id, id, name) VALUES ($${}$$, $${}$$, $${}$$, $${}$$)",
                    //                     pollingplacestruct.event_id, pollingplacestruct.district_id, pollingplacestruct.id, pollingplacestruct.name);
                    format!(
                        "($${}$$, $${}$$, $${}$$, $${}$$, $${}$$, $${}$$)",
                        pollingplacestruct.event_id,
                        pollingplacestruct.district_id,
                        pollingplacestruct.id,
                        pollingplacestruct.name,
                        pollingplacestruct.lat,
                        pollingplacestruct.long
                    )
                })
                .collect();
            pollingplace_structs.join(", ")
        })
        .collect();
    let query = format!(
        "INSERT INTO aec.pollingplace (event_id, district_id, id, name, lat, long) VALUES {}",
        pollingplace_structs.join(", ")
    );
    &db.run_raw(query).await;
}

async fn read_eml_110_event_xml(
    election_id: &String,
    db: &mut MySQLDB,
    mut zipfile: &mut ZipArchive<Cursor<Vec<u8>>>,
) {
    let mut results_string: String = String::new();
    zipfile
        .by_name(format!("xml/eml-110-event-{}.xml", election_id).as_str())
        .unwrap()
        .read_to_string(&mut results_string)
        .unwrap();
    let mut results_string: Vec<&str> = results_string.split('\n').collect();
    results_string.remove(0);
    let results_string = results_string.join("\n");

    let rootnode: Element = Element::from_str(results_string.as_str()).unwrap();
    let transaction_id = rootnode
        .get_child_ignore_ns("TransactionId")
        .unwrap()
        .text();

    let election_event = ElectionEvent {
        election_id: election_id.clone(),
        transaction_id,
        name: rootnode
            .get_child_ignore_ns("ElectionEvent")
            .unwrap()
            .get_child_ignore_ns("EventIdentifier")
            .unwrap()
            .get_child_ignore_ns("EventName")
            .unwrap()
            .text(),
    };
    let query = format!("INSERT INTO aec.electionevent (election_id, transaction_id, name) VALUES ($${}$$, $${}$$, $${}$$)", election_event.election_id, election_event.transaction_id, election_event.name);
    &db.run_raw(query).await;

    for electionelement in rootnode
        .get_child_ignore_ns("ElectionEvent")
        .unwrap()
        .get_children_ignore_ns("Election")
    {
        let electionstruct = Election {
            event_id: election_id.clone(),
            identifier: electionelement
                .get_child_ignore_ns("ElectionIdentifier")
                .unwrap()
                .attr("Id")
                .unwrap()
                .to_string(),
            name: electionelement
                .get_child_ignore_ns("ElectionIdentifier")
                .unwrap()
                .get_child_ignore_ns("ElectionName")
                .unwrap()
                .text(),
            category: electionelement
                .get_child_ignore_ns("ElectionIdentifier")
                .unwrap()
                .get_child_ignore_ns("ElectionCategory")
                .unwrap()
                .text(),
            date: electionelement
                .get_child_ignore_ns("Date")
                .unwrap()
                .get_child_ignore_ns("SingleDate")
                .unwrap()
                .text(),
        };
        let query = format!("INSERT INTO aec.election (event_id, identifier, name, category, date) VALUES ($${}$$, $${}$$, $${}$$, $${}$$, $${}$$)",
                            electionstruct.event_id, electionstruct.identifier, electionstruct.name, electionstruct.category, electionstruct.date);
        &db.run_raw(query).await;

        for contestelement in electionelement.get_children_ignore_ns("Contest") {
            let contest_struct = Contest {
                event_id: election_id.clone(),
                identifier: electionstruct.identifier.clone(),
                id: contestelement
                    .get_child_ignore_ns("ContestIdentifier")
                    .unwrap()
                    .attr("Id")
                    .unwrap()
                    .to_string(),
                shortcode: contestelement
                    .get_child_ignore_ns("ContestIdentifier")
                    .unwrap()
                    .attr("ShortCode")
                    .unwrap_or("")
                    .to_string(),
                name: contestelement
                    .get_child_ignore_ns("ContestIdentifier")
                    .unwrap()
                    .get_child_ignore_ns("ContestName")
                    .unwrap()
                    .text(),
            };
            let query = format!("INSERT INTO aec.contest (event_id, identifier, id, shortcode, name) VALUES ($${}$$, $${}$$, $${}$$, $${}$$, $${}$$)",
                                contest_struct.event_id, contest_struct.identifier, contest_struct.id, contest_struct.shortcode, contest_struct.name);
            &db.run_raw(query).await;
        }
    }
}

async fn read_eml_230_candidates(
    election_id: &String,
    db: &mut MySQLDB,
    mut zipfile: &mut ZipArchive<Cursor<Vec<u8>>>,
) {
    let mut results_string: String = String::new();
    if (election_id == "29581") {
        //TODO better way to deal with referendum
        return;
    }
    zipfile
        .by_name(format!("xml/eml-230-candidates-{}.xml", election_id).as_str())
        .unwrap()
        .read_to_string(&mut results_string)
        .unwrap();

    let mut results_string: Vec<&str> = results_string.split('\n').collect();
    results_string.remove(0);
    let results_string = results_string.join("\n");

    let rootnode: Element = Element::from_str(results_string.as_str()).unwrap();

    for electionelement in rootnode
        .get_child_ignore_ns("CandidateList")
        .unwrap()
        .get_children_ignore_ns("Election")
    {
        let electionstruct = Election {
            event_id: election_id.clone(),
            identifier: electionelement
                .get_child_ignore_ns("ElectionIdentifier")
                .unwrap()
                .attr("Id")
                .unwrap()
                .to_string(),
            name: electionelement
                .get_child_ignore_ns("ElectionIdentifier")
                .unwrap()
                .get_child_ignore_ns("ElectionName")
                .unwrap()
                .text(),
            category: electionelement
                .get_child_ignore_ns("ElectionIdentifier")
                .unwrap()
                .get_child_ignore_ns("ElectionCategory")
                .unwrap()
                .text(),
            date: "".to_string(),
        };
        for contest in electionelement.get_children_ignore_ns("Contest") {
            let contest_id = contest
                .get_child_ignore_ns("ContestIdentifier")
                .unwrap()
                .attr("Id")
                .unwrap();
            for candidate in contest.get_children_ignore_ns("Candidate") {
                let candidate = Candidate {
                    event_id: electionstruct.event_id.clone(),
                    election_identifier: electionstruct.identifier.clone(),
                    contest_id: contest_id.to_string(),
                    id: candidate
                        .get_child_ignore_ns("CandidateIdentifier")
                        .unwrap()
                        .attr("Id")
                        .unwrap()
                        .to_string(),
                    name: candidate
                        .get_child_ignore_ns("CandidateIdentifier")
                        .unwrap()
                        .get_child_ignore_ns("CandidateName")
                        .unwrap()
                        .text(),
                    affiliation: match candidate.get_child_ignore_ns("Affiliation") {
                        None => "".to_string(),
                        Some(affiliation) => affiliation
                            .get_child_ignore_ns("AffiliationIdentifier")
                            .unwrap()
                            .attr("ShortCode")
                            .unwrap()
                            .to_string(),
                    },
                };
                let query = format!("INSERT INTO aec.candidate (event_id, election_identifier, contest_id, id, name, affiliation) VALUES ($${}$$, $${}$$, $${}$$, $${}$$, $${}$$, $${}$$)",
                                    candidate.event_id, candidate.election_identifier, candidate.contest_id, candidate.id, candidate.name, candidate.affiliation);
                &db.run_raw(query).await;
            }
        }
    }
}

async fn create_tables(db: &mut MySQLDB) {
    // &db.run_raw(ElectionEvent::postgres_drop()).await;
    // &db.run_raw(ElectionEvent::postgres_create()).await;
    // &db.run_raw(Election::postgres_drop()).await;
    // &db.run_raw(Election::postgres_create()).await;
    // &db.run_raw(Contest::postgres_drop()).await;
    // &db.run_raw(Contest::postgres_create()).await;
    // &db.run_raw(Candidate::postgres_drop()).await;
    // &db.run_raw(Candidate::postgres_create()).await;
    // &db.run_raw(PollingPlace::postgres_drop()).await;
    // &db.run_raw(PollingPlace::postgres_create()).await;
    &db.run_raw(PrimaryVote::postgres_drop()).await;
    &db.run_raw(PrimaryVote::postgres_create()).await;
    &db.run_raw(TwoCandidatePreferredVote::postgres_drop()).await;
    &db.run_raw(TwoCandidatePreferredVote::postgres_create()).await;
}

/**
    STRUCTURE FOR DATABASE
**/

///Essentially when the election happens
#[derive(PostGresObj, Debug)]
struct ElectionEvent {
    election_id: String,
    transaction_id: String, //TODO check if this is the same for all messages
    name: String,
}

//TODO foreign keys
#[derive(PostGresObj, Debug)]
struct Election {
    event_id: String,
    identifier: String,
    name: String,
    category: String,
    date: String,
}

#[derive(PostGresObj, Debug)]
struct Contest {
    event_id: String,
    identifier: String,
    id: String,
    shortcode: String,
    name: String,
}

#[derive(PostGresObj, Debug)]
struct Candidate {
    event_id: String,
    election_identifier: String,
    contest_id: String,
    //ref details
    id: String,
    name: String,
    affiliation: String,
}

#[derive(PostGresObj, Debug)]
struct PollingPlace {
    event_id: String,
    district_id: String,
    id: String,
    name: String,
    lat: String,
    long: String,
}

#[derive(PostGresObj, Debug)]
struct PrimaryVote {
    timestamp: String,
    event_id: String,
    contest_id: String,
    polling_place_id: String,
    candidate_id: String,
    votes: i32,
}

#[derive(PostGresObj, Debug)]
struct TwoCandidatePreferredVote {
    timestamp: String,
    event_id: String,
    contest_id: String,
    polling_place_id: String,
    candidate_id: String,
    votes: i32,
}
