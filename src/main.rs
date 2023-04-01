use std::io::Read;
use suppaftp::{FtpError, FtpStream};
use std::str;
use minidom::Element;
use rusqlite::{Connection, Result};
use zip::ZipArchive;

fn main() {
    let mut ftp_stream: FtpStream = FtpStream::connect("mediafeedarchive.aec.gov.au:21").unwrap();
    let _ = ftp_stream.login("anonymous", "").unwrap();

    let conn = Connection::open("maindb.db").unwrap();
    reset_db(&conn);

    //get all elections
    let election_ids = get_all_in_dir(&mut ftp_stream);
    election_ids.into_iter().for_each(|x| preload_data(&conn, &mut ftp_stream, x.as_str()));

    // Terminate the connection to the server.
    let _ = ftp_stream.quit();
}

fn reset_db(conn: &Connection) {
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
}

fn preload_data(connection: &Connection, mut ftp_stream: &mut FtpStream, event_id: &str) {
    ftp_stream.cwd(event_id).unwrap();
    ftp_stream.cwd("Detailed/Preload").unwrap();
    let latest = get_all_in_dir(&mut ftp_stream).last().unwrap().clone();
    let file = ftp_stream.retr_as_buffer(latest.as_str()).unwrap();
    let mut zipfile = ZipArchive::new(file).unwrap();
    let mut events_string: String = String::new();
    zipfile.by_name(format!("xml/eml-110-event-{}.xml", event_id).as_str()).unwrap().read_to_string(&mut events_string).unwrap();

    const NAMESPACE: &str = "urn:oasis:names:tc:evs:schema:eml";
    let mut events_string: Vec<&str> = events_string.split("\n").collect();
    events_string.remove(0);
    let candidates_string = events_string.join("\n");
    let root: Element = candidates_string.parse().unwrap();
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

    ftp_stream.cwd("/").unwrap();
}

fn change_dir(mut stream: &mut FtpStream, x: &str) {
    stream.cwd(x).unwrap();
    println!("Current directory: {}", stream.pwd().unwrap());
}

fn get_all_in_dir(mut ftp_stream: &mut FtpStream) -> Vec<String> {
    ftp_stream.list(None).unwrap().into_iter().map(|row| row.split(" ").last().unwrap_or("").to_string()).collect::<Vec<_>>()
}


