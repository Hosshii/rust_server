use crate::error::ServerError;
use crate::header::{ContentType, HttpHeader};
use crate::method::Method;
use crate::server::{ServeHandler, Server};
use crate::status_code::StatusCode;
use std::collections::HashMap;
use std::io::prelude::*;
use std::io::Read;
use std::io::{BufRead, BufReader};
use std::net::TcpStream;
use std::str::FromStr;
use std::sync::Arc;
use uncased;

type Path = String;
type Version = String;
pub type Header = HashMap<String, String>;

const HTTP_11: &str = "HTTP/1.1";

pub struct Message {
    pub req: Request,
    pub res: Response,
    conn: Conn,
}

pub trait ResponseWriter {
    fn write(&mut self, data: ResponseBody);
    fn header(&mut self, headers: Header);
    fn write_header(&mut self, code: usize);
    fn send(&mut self);
}

impl ResponseWriter for Message {
    fn write(&mut self, data: ResponseBody) {
        self.res.body = Some(data);
    }
    fn write_header(&mut self, code: usize) {
        self.res.status_code = StatusCode::from_num(code).unwrap_or_else(|_| StatusCode::Ok);
    }
    fn header(&mut self, headers: Header) {
        self.res.headers = headers;
    }
    fn send(&mut self) {
        self.conn
            .stream
            .write(self.res.format().as_bytes())
            .unwrap();
        self.conn.stream.flush();
    }
}

impl Message {
    pub(crate) fn new(conn: Conn) -> Self {
        Message {
            req: Request::new(),
            res: Response::new(),
            conn,
        }
    }
}

pub(crate) struct Conn {
    server: Arc<Server>,
    stream: TcpStream,
}

impl Conn {
    pub fn new(server: Arc<Server>, stream: TcpStream) -> Conn {
        Conn {
            server: server,
            stream: stream,
        }
    }

    pub fn serve(self) -> Result<(), ServerError> {
        let serve_handler = ServeHandler::new(self.server.clone());
        let msg = &mut Message::new(self);
        msg.req.parse(&mut msg.conn.stream)?;
        let req = msg.req.clone();
        serve_handler.serve_http(msg, &req)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum RequestState {
    FirstLine,
    Header,
    Body,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum RequestBody {
    StringBody(String),
    BytesBody(Vec<u8>),
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Request {
    pub method: Method,
    pub path: Path,
    pub version: Version,
    pub headers: Header,
    pub body: Option<RequestBody>,
    state: RequestState,
    pub content_length: u64,
    pub content_type: ContentType,
}

impl Request {
    pub fn new() -> Request {
        Request {
            method: Method::Other,
            path: "/".to_string(),
            version: HTTP_11.to_string(),
            headers: HashMap::new(),
            body: None,
            state: RequestState::FirstLine,
            content_length: 0,
            content_type: ContentType::TextPlain,
        }
    }

    pub fn parse(&mut self, msg: &mut TcpStream) -> Result<(), ServerError> {
        println!("parse start");
        let mut buf = String::new();
        let mut reader = BufReader::new(msg);
        while reader.read_line(&mut buf).unwrap() > 0 {
            buf.pop();
            buf.pop();
            match &self.state {
                RequestState::FirstLine => {
                    let v: Vec<&str> = buf.split(" ").collect();
                    read_first_line(self, v)?;
                    self.state = RequestState::Header;
                }
                RequestState::Header => {
                    if buf == "" {
                        self.state = RequestState::Body;
                        break;
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
        read_body(self, reader)?;

        Ok(())
    }
}

fn read_first_line(msg: &mut Request, v: Vec<&str>) -> Result<(), ServerError> {
    if v.len() < 2 {
        return Err(ServerError::ReadHeaderError);
    }

    if let Ok(x) = Method::from_str(v[0]) {
        msg.method = x;
    } else {
        return Err(ServerError::ReadHeaderError);
    };

    if v.len() < 3 {
        msg.version = v[1].to_string();
    } else {
        msg.path = v[1].to_string();
        msg.version = v[2].to_string();
    }
    Ok(())
}

fn read_header(msg: &mut Request, v: Vec<&str>) -> Result<(), ServerError> {
    if v.len() < 2 {
        return Err(ServerError::ReadHeaderError);
    }

    match v[0] {
        x if uncased::eq(x, HttpHeader::ContentLength.as_str()) => {
            let content_length: u64 = v[1].trim().parse().unwrap_or_else(|_| 0);
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

fn read_body(msg: &mut Request, reader: BufReader<&mut TcpStream>) -> Result<(), ServerError> {
    let mut v = Vec::new();
    let mut chunk = reader.take(msg.content_length);
    let _ = chunk.read_to_end(&mut v).unwrap();
    match msg.content_type {
        ContentType::ApplicationJson | ContentType::TextHtml | ContentType::TextPlain => {
            msg.body = Some(RequestBody::StringBody(
                String::from_utf8_lossy(&v).to_string(),
            ));
        }
        ContentType::ImageJpeg | ContentType::ImagePng => {
            msg.body = Some(RequestBody::BytesBody(v))
        }
    }
    Ok(())
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum ResponseBody {
    StringBody(String),
    BytesBody(Vec<u8>),
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Response {
    pub version: Version,
    pub status_code: StatusCode,
    pub headers: Header,
    pub body: Option<ResponseBody>,
    pub content_length: u64,
    pub content_type: ContentType,
}

impl Response {
    fn new() -> Self {
        Response {
            version: HTTP_11.to_string(),
            status_code: StatusCode::Ok,
            headers: HashMap::new(),
            body: None,
            content_length: 0,
            content_type: ContentType::TextPlain,
        }
    }

    fn format(&self) -> String {
        let status_line = format!(
            "{} {} {}\r\n",
            self.version,
            self.status_code.as_num(),
            self.status_code.as_str()
        );

        let mut headers = String::new();
        for (key, value) in &self.headers {
            let s = format!("{}: {}\r\n", key, value);
            headers = headers + &s;
        }

        let mut body = "";
        if let Some(x) = &self.body {
            match x {
                ResponseBody::BytesBody(_y) => todo!(),
                ResponseBody::StringBody(y) => body = y,
            }
        }
        format!("{}{}{}", status_line, headers, body)
    }
}
