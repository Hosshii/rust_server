use crate::error::ParseError;
use crate::header::HttpHeader;
use crate::method::Method;
use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::net::TcpStream;
use std::str::FromStr;

type Path = String;
type Version = String;
type Header = HashMap<String, String>;

const BodyMaxSize: usize = 1024 * 256;
const HeaderMaxSize: usize = 1024 * 80;
const MessageMaxSize: usize = BodyMaxSize + HeaderMaxSize + 1024;
const http_11: &str = "HTTP/1.1";

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum MessageState {
    FirstLine,
    Header,
    Body,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Message {
    pub method: Method,
    pub path: Path,
    pub version: Version,
    pub headers: Vec<Header>,
    pub message: Option<[u8; MessageMaxSize]>,
    state: MessageState,
}

impl Message {
    pub fn new() -> Message {
        Message {
            method: Method::Other,
            path: "/".to_string(),
            version: http_11.to_string(),
            headers: Vec::new(),
            message: None,
            state: MessageState::FirstLine,
        }
    }

    pub fn parse(&mut self, msg: &TcpStream) -> Result<(), Box<dyn std::error::Error>> {
        for result in BufReader::new(msg).lines() {
            let line = result?;
            match &self.state {
                FirstLine => {
                    let v: Vec<&str> = line.split(" ").collect();
                    if v.len() < 2 {
                        return Err(Box::new(ParseError::ReadHeaderError));
                    }
                    if let Ok(x) = Method::from_str(v[0]) {
                        self.method = x;
                    } else {
                        return Err(Box::new(ParseError::ReadHeaderError));
                    }
                    if v.len() < 3 {
                        self.version = v[1].to_string();
                    } else {
                        self.path = v[1].to_string();
                        self.version = v[2].to_string();
                    }
                }
                Header => (),
                Body => (),
            }
        }
        Ok(())
    }
}
