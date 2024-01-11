use std::io::{Cursor};
use std::net::ToSocketAddrs;
use suppaftp::FtpStream;

pub struct ElectionServer<A: ToSocketAddrs> {
    addr: A,
    ftp_stream: FtpStream,
}

impl<A: std::net::ToSocketAddrs> ElectionServer<A> {
    pub fn new(addr: A) -> Self {
        let mut ftp_stream = FtpStream::connect(&addr).unwrap();
        ftp_stream.login("anonymous", "").unwrap();
        Self { addr, ftp_stream }
    }

    pub fn quit(&mut self) {
        self.ftp_stream.quit().unwrap();
    }

    pub fn get_all_in_dir(&mut self) -> Vec<String> {
        self.ftp_stream
            .list(None)
            .unwrap()
            .into_iter()
            .map(|row| row.split(' ').last().unwrap_or("").to_string())
            .collect::<Vec<_>>()
    }

    pub fn cwd(&mut self, path: &str) {
        self.ftp_stream.cwd(path).unwrap();
    }

    pub fn get_zip(&mut self, path: &str) -> Cursor<Vec<u8>> {
        self.ftp_stream.retr_as_buffer(path).unwrap()
    }
}
