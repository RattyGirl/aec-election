use std::io::Read;
use suppaftp::{FtpStream};
use std::str;
use minidom::Element;
use rusqlite::{Connection};
use zip::ZipArchive;


const NAMESPACE: &str = "urn:oasis:names:tc:evs:schema:eml";

fn main() {




    let mut ftp_stream: FtpStream = FtpStream::connect("mediafeedarchive.aec.gov.au:21").unwrap();
    let _ = ftp_stream.login("anonymous", "").unwrap();

    let conn = Connection::open("maindb.db").unwrap();
    reset_db(&conn);


    //get all elections
    let election_ids = get_all_in_dir(&mut ftp_stream);
    ftp_stream.quit().unwrap();

    election_ids.into_iter().for_each(|x| preload_data(&conn, x.as_str()));
}

fn reset_db(conn: &Connection) {
    conn.execute("DROP TABLE IF EXISTS candidates", ()).unwrap();
    conn.execute("DROP TABLE IF EXISTS contests", ()).unwrap();
    conn.execute("DROP TABLE IF EXISTS elections", ()).unwrap();
    conn.execute("DROP TABLE IF EXISTS events", ()).unwrap();
    conn.execute("CREATE TABLE events (\
        id integer primary key,\
        name text\
    )", ()).unwrap();
    conn.execute("CREATE TABLE elections (\
        event_id integer,\
        id text,\
        date text,\
        name text,\
        category text,\
        FOREIGN KEY(event_id) REFERENCES events(id),\
        PRIMARY KEY (event_id, id)\
    )", ()).unwrap();
    conn.execute("CREATE TABLE contests (\
        event_id integer,\
        election_id integer,\
        id text,\
        short_code text,\
        name text,\
        position text,\
        number integer,\
        FOREIGN KEY(event_id) REFERENCES events(id),\
        FOREIGN KEY(election_id) REFERENCES elections(id),\
        PRIMARY KEY (event_id, election_id, id)\
    )", ()).unwrap();
    conn.execute("CREATE TABLE candidates (\
        event_id integer,\
        election_id integer,\
        contest_id text,\
        id integer,\
        name text,\
        profession text,\
        FOREIGN KEY(event_id) REFERENCES events(id),\
        FOREIGN KEY(election_id) REFERENCES elections(id),
        FOREIGN KEY(contest_id) REFERENCES contests(id)\
    )", ()).unwrap();
}

fn preload_data(connection: &Connection, event_id: &str) {
    let mut ftp_stream: FtpStream = FtpStream::connect("mediafeedarchive.aec.gov.au:21").unwrap();
    let _ = ftp_stream.login("anonymous", "").unwrap();

    ftp_stream.cwd(event_id).unwrap();
    ftp_stream.cwd("Detailed/Preload").unwrap();
    let latest = get_all_in_dir(&mut ftp_stream).last().unwrap().clone();
    let file = ftp_stream.retr_as_buffer(latest.as_str()).unwrap();
    let mut zipfile = ZipArchive::new(file).unwrap();

    let mut events_string: String = String::new();
    zipfile.by_name(format!("xml/eml-110-event-{}.xml", event_id).as_str()).unwrap().read_to_string(&mut events_string).unwrap();
    get_all_races(events_string, &connection, event_id);
    println!("Gotten all races for event {}", event_id);

    let mut candidates_string: String = String::new();
    zipfile.by_name(format!("xml/eml-230-candidates-{}.xml", event_id).as_str()).unwrap().read_to_string(&mut candidates_string).unwrap();
    get_all_candidates(candidates_string, &connection, event_id);
    println!("Gotten all candidates for event {}", event_id);
}

fn get_all_candidates(candidates_string: String, connection: &Connection, event_id: &str) {
    let mut candidates_string: Vec<&str> = candidates_string.split("\n").collect();
    candidates_string.remove(0);
    let candidates_string = candidates_string.join("\n");
    let root: Element = candidates_string.parse().unwrap();
    let candidate_list = root.get_child("CandidateList", NAMESPACE).unwrap();

    let elections = candidate_list.children().filter(|f| f.name().eq("Election"));
    elections.for_each(|election| {
        let election_id = election.get_child("ElectionIdentifier", NAMESPACE).unwrap().attr("Id").unwrap();

        election.children().filter(|f| f.name().eq("Contest")).for_each(|contest| {
            let contest_id = contest.get_child("ContestIdentifier", NAMESPACE).unwrap().attr("Id").unwrap();

            contest.children().filter(|f| f.name().eq("Candidate")).for_each(|candidate| {
                let candidate_id = candidate.get_child("CandidateIdentifier", NAMESPACE).unwrap().attr("Id").unwrap();
                let candidate_name = candidate.get_child("CandidateIdentifier", NAMESPACE).unwrap().get_child("CandidateName", NAMESPACE).unwrap().text();
                let candidate_profession = candidate.get_child("Profession", NAMESPACE).unwrap().text();

                connection.execute("INSERT INTO candidates VALUES (?1, ?2, ?3, ?4, ?5, ?6)", (event_id, election_id, contest_id, candidate_id, candidate_name, candidate_profession)).unwrap();
            });
        });
    })
}

fn get_all_races(events_string: String, connection: &Connection, event_id: &str) {
    let mut events_string: Vec<&str> = events_string.split("\n").collect();
    events_string.remove(0);
    let events_string = events_string.join("\n");
    let root: Element = events_string.parse().unwrap();
    let mut iter = root.children();
    let election_event = root.get_child("ElectionEvent", NAMESPACE).unwrap();
    let event_identifier = election_event.get_child("EventIdentifier", NAMESPACE).unwrap();

    let name = event_identifier.get_child("EventName", NAMESPACE).unwrap().text();
    connection.execute("INSERT into events VALUES (?1, ?2)", (event_id, event_identifier.get_child("EventName", NAMESPACE).unwrap().text())).unwrap();

    let elections = election_event.children().filter(|f| f.name().eq("Election"));
    elections.for_each(|election| {
        let election_id = election.get_child("ElectionIdentifier", NAMESPACE).unwrap().attr("Id").unwrap();
        let date = election.get_child("Date", NAMESPACE).unwrap().get_child("SingleDate", NAMESPACE).unwrap().text();
        let name = election.get_child("ElectionIdentifier", NAMESPACE).unwrap().get_child("ElectionName", NAMESPACE).unwrap().text();
        let category = election.get_child("ElectionIdentifier", NAMESPACE).unwrap().get_child("ElectionCategory", NAMESPACE).unwrap().text();
        connection.execute("INSERT into elections VALUES (?1, ?2, ?3, ?4, ?5)", (event_id, election_id, date, name, category)).unwrap();

        election.children().filter(|f| f.name().eq("Contest")).for_each(|contest| {
            let contest_id = contest.get_child("ContestIdentifier", NAMESPACE).unwrap().attr("Id").unwrap_or("");
            let short_code = contest.get_child("ContestIdentifier", NAMESPACE).unwrap().attr("ShortCode").unwrap_or("");
            let name = contest.get_child("ContestIdentifier", NAMESPACE).unwrap().get_child("ContestName", NAMESPACE).unwrap().text();
            let position = contest.get_child("Position", NAMESPACE).unwrap().text();
            let number =  contest.get_child("NumberOfPositions", NAMESPACE).unwrap().text();

            connection.execute("INSERT INTO contests VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)", (event_id, election_id, contest_id, short_code, name, position, number)).unwrap();
        });

    });
}

fn change_dir(mut stream: &mut FtpStream, x: &str) {
    stream.cwd(x).unwrap();
    println!("Current directory: {}", stream.pwd().unwrap());
}

fn get_all_in_dir(mut ftp_stream: &mut FtpStream) -> Vec<String> {
    ftp_stream.list(None).unwrap().into_iter().map(|row| row.split(" ").last().unwrap_or("").to_string()).collect::<Vec<_>>()
}

struct Event {
    id: i32,
    name: String
}

struct Election {
    event: Event,
    id: i32,
    date: String,
    name: String,
    category: String,
}

struct Contest {
    event: Event,
    election: Election,
    id: i32,
    short_code: String,
    name: String,
    position: String,
    number: i32,
}

struct Candidate {
    event: Event,
    election: Election,
    contest: Contest,
    id: i32,
    name: String,
    profession: String,
}