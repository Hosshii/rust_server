use crate::worker::{PoolCreationErr, ThreadPool};
use std::collections::HashMap;
use std::fmt;
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};

pub struct Server<'a> {
    pool: ThreadPool,
    get_handle: HashMap<String, &'a (dyn Fn(TcpStream) -> Result<(), String>)>,
}

const GET: &str = "GET";

// #[derive(Clone)]
struct HttpHeader {
    method: String,
    path: String,
}

impl<'a> Server<'a> {
    pub fn new(size: usize) -> Server<'a> {
        let pool = ThreadPool::new(size).unwrap();

        Server {
            pool,
            get_handle: HashMap::new(),
        }
    }

    /// start the server
    ///
    /// specify the port number
    pub fn start(&self, port: u32) {
        let ip_addr = "127.0.0.1";
        let address = format!("{}:{}", ip_addr, port);
        let listener = TcpListener::bind(address).unwrap();
        for stream in listener.incoming() {
            let mut stream = stream.unwrap();
            self.parce(&mut stream)
                .and_then(|h| {
                    let get = GET.to_string();
                    self.pool
                        .execute(|| (self.get_handle.get(&h.path).unwrap())(stream).unwrap());
                    Ok(())
                    // match &h.path {
                    //     get => (self.get_handle.get(&h.path).unwrap())(stream),
                    //     _ => Ok(()),
                    // }
                })
                .unwrap();
        }
    }

    pub fn GET(
        &mut self,
        path: impl Into<String>,
        func: (&'a dyn Fn(TcpStream) -> Result<(), String>),
    ) {
        self.get_handle.insert(path.into(), func);
    }

    fn parce(&self, stream: &mut TcpStream) -> Result<HttpHeader, String> {
        let mut st = String::new();
        stream.read_to_string(&mut st).unwrap();
        let mut buf = String::new();
        for i in st.chars() {
            if i == '\r' {
                break;
            }
            // buf += i;
            buf.push(i);
        }
        let req_info: Vec<&str> = buf.split(" ").collect();
        let test = HttpHeader {
            method: GET.to_string(),
            path: req_info[1].to_string(),
        };
        let a = req_info[0].to_string();
        let get = GET.to_string();
        match a {
            // GET => Ok(HttpHeader {
            //     method: GET,
            //     path: req_info[1],
            // }),
            get => Ok(test),
            _ => Err("parsing eerr".to_string()),
        }
    }
}
