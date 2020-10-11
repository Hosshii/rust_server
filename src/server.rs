use crate::worker::{PoolCreationErr, ThreadPool};
use std::collections::HashMap;
use std::fmt;
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};

pub struct Server {
    pool: ThreadPool,
    get_handle: HashMap<String, fn(TcpStream)>,
}

const GET: &str = "GET";

// #[derive(Clone)]
struct HttpHeader {
    method: String,
    path: String,
}

struct HttpMessage {
    all: stream_buffer,
    header: HttpHeader,
}

pub type stream_buffer = [u8; 1024];

impl Server {
    pub fn new(size: usize) -> Server {
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
            println!("before parce");
            self.parce(&mut stream)
                .and_then(|h| {
                    let get = GET.to_string();
                    let a = self.get_handle[&h.header.path];
                    // let b: fn(TcpStream) = handle_connection;
                    // let c = || a(stream);
                    // let d = move || b(stream);
                    println!("before pool");
                    self.pool
                        // .execute(|| (self.get_handle.get(&h.path).unwrap())(stream));
                        .execute(move || a(stream));
                    // .execute(|| handle_connection(stream));
                    // .execute(|| crate::index());
                    Ok(())
                    // match &h.path {
                    //     get => (self.get_handle.get(&h.path).unwrap())(stream),
                    //     _ => Ok(()),
                    // }
                })
                .unwrap();
        }
    }

    pub fn GET(&mut self, path: impl Into<String>, func: fn(TcpStream)) {
        self.get_handle.insert(path.into(), func);
    }

    fn parce(&self, stream: &mut TcpStream) -> Result<HttpMessage, String> {
        let mut st = [0; 1024];
        stream.read(&mut st).unwrap();
        println!("here");
        let mut buf = String::new();
        for i in st.iter() {
            if *i == ('\r' as u8) {
                break;
            }
            // buf += i;
            buf.push(*i as char);
        }
        let req_info: Vec<&str> = buf.split(" ").collect();
        let test = HttpHeader {
            method: GET.to_string(),
            path: req_info[1].to_string(),
        };
        let msg = HttpMessage {
            header: test,
            all: st,
        };
        let a = req_info[0].to_string();
        let get = GET.to_string();
        match a {
            // GET => Ok(HttpHeader {
            //     method: GET,
            //     path: req_info[1],
            // }),
            get => Ok(msg),
            _ => Err("parsing eerr".to_string()),
        }
    }
}

pub fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();

    let get = b"GET / HTTP/1.1\r\n";
    let sleep = b"GET /sleep HTTP/1.1\r\n";

    let (status_line, filename) = if buffer.starts_with(get) {
        ("HTTP/1.1 200 OK\r\n\r\n", "hello.html")
    } else if buffer.starts_with(sleep) {
        // thread::sleep(Duration::from_secs(5));
        ("HTTP/1.1 200 OK\r\n\r\n", "hello.html")
    } else {
        ("HTTP/1.1 404 Not Found\r\n\r\n", "404.html")
    };

    // let mut file = File::open(filename).unwrap();

    let mut contents = String::new();
    // file.read_to_string(&mut contents).unwrap();

    let response = format!("{}{}", status_line, contents);

    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}
