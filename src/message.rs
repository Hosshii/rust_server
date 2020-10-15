use crate::error::ParseError;
use crate::header::{ContentType, HttpHeader};
use crate::method::Method;
use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::net::TcpStream;
use std::str::FromStr;
use uncased;

type Path = String;
type Version = String;
type Header = HashMap<String, String>;

const BODY_MAX_SIZE: usize = 1024 * 256;
const HEADER_MAX_SIZE: usize = 1024 * 80;
const MESSAGE_MAX_SIZE: usize = BODY_MAX_SIZE + HEADER_MAX_SIZE + 1024;
const HTTP_11: &str = "HTTP/1.1";

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
    pub headers: Header,
    pub message: Option<[u8; MESSAGE_MAX_SIZE]>,
    state: MessageState,
    pub content_length: u64,
    pub content_type: ContentType,
}

impl Message {
    pub fn new() -> Message {
        Message {
            method: Method::Other,
            path: "/".to_string(),
            version: HTTP_11.to_string(),
            headers: HashMap::new(),
            message: None,
            state: MessageState::FirstLine,
            content_length: 0,
            content_type: ContentType::TextPlain,
        }
    }

    pub fn parse(&mut self, msg: &TcpStream) -> Result<(), ParseError> {
        println!("parse start");
        let mut buf = String::new();
        let mut reader = BufReader::new(msg);
        while reader.read_line(&mut buf).unwrap() > 0 {
            buf.pop();
            buf.pop();
            match &self.state {
                MessageState::FirstLine => {
                    let v: Vec<&str> = buf.split(" ").collect();
                    read_first_line(self, v)?;
                    self.state = MessageState::Header;
                }
                MessageState::Header => {
                    if buf == "" {
                        self.state = MessageState::Body;
                        if self.content_length <= 0 {
                            break;
                        }
                        continue;
                    }
                    let v: Vec<&str> = buf.split(":").collect();
                    read_header(self, v)?
                }
                _ => {
                    println!("{:?}", self.state);
                    break;
                }
            }
            buf.clear();
        }

        Ok(())
    }
}

fn read_first_line(msg: &mut Message, v: Vec<&str>) -> Result<(), ParseError> {
    if v.len() < 2 {
        return Err(ParseError::ReadHeaderError);
    }

    if let Ok(x) = Method::from_str(v[0]) {
        msg.method = x;
    } else {
        return Err(ParseError::ReadHeaderError);
    };

    if v.len() < 3 {
        msg.version = v[1].to_string();
    } else {
        msg.path = v[1].to_string();
        msg.version = v[2].to_string();
    }
    Ok(())
}

fn read_header(msg: &mut Message, v: Vec<&str>) -> Result<(), ParseError> {
    if v.len() < 2 {
        return Err(ParseError::ReadHeaderError);
    }

    match v[0] {
        x if uncased::eq(x, HttpHeader::ContentLength.as_str()) => {
            let content_length: u64 = v[1].parse().unwrap_or_else(|_| 0);
            msg.content_length = content_length;
        }
        x if uncased::eq(x, HttpHeader::ContentType.as_str()) => {
            if let Ok(x) = ContentType::from_str(v[1]) {
                msg.content_type = x;
            }
        }
        _ => {
            msg.headers.insert(v[0].to_string(), v[1].to_string());
        }
    }

    Ok(())
}
