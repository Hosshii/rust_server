use crate::error::ServerError;
use crate::message::Header;
use crate::message::Request;
use crate::method::Method;
use crate::worker::ThreadPool;
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::rc::Rc;
use std::sync::{Arc, Mutex};

pub trait Handler {
    fn serve_http(&self, writer: &dyn ResponseWriter, req: &Request);
}

struct GetEntry {
    path: String,
    handler: Rc<dyn Handler>,
}
impl GetEntry {
    fn new(path: String, handler: Rc<dyn Handler>) -> Self {
        GetEntry { path, handler }
    }
}

struct PostEntry {
    path: String,
    handler: Rc<dyn Handler>,
}
impl PostEntry {
    fn new(path: String, handler: Rc<dyn Handler>) -> Self {
        PostEntry { path, handler }
    }
}

pub struct DefaultServeMux {
    get: HashMap<String, GetEntry>,
    post: HashMap<String, PostEntry>,
}

impl Handler for DefaultServeMux {
    fn serve_http(&self, writer: &dyn ResponseWriter, req: &Request) {
        self.handler(req).serve_http(writer, req);
    }
}

impl DefaultServeMux {
    fn new() -> Self {
        DefaultServeMux {
            get: HashMap::new(),
            post: HashMap::new(),
        }
    }

    // fn handler(&self, r: &Request) -> impl Handler {
    fn handler(&self, r: &Request) -> Rc<dyn Handler> {
        match r.method {
            Method::Get => {
                if let Some(x) = self.get.get(&r.path) {
                    x.handler.clone()
                } else {
                    Rc::new(NotFoundHandler::new())
                }
                // Box::new(NotFoundHandler::new())
            }
            _ => Rc::new(NotFoundHandler::new()),
        }
    }
}

pub trait ResponseWriter {
    fn write(&mut self, data: Vec<u8>);
    fn header(&mut self, headers: Header);
    fn write_header(&mut self, code: usize);
}

pub struct Response<'a> {
    conn: &'a Conn,
    req: &'a Request,
}
impl<'a> ResponseWriter for Response<'a> {
    fn write(&mut self, data: Vec<u8>) {}
    fn header(&mut self, headers: Header) {}
    fn write_header(&mut self, code: usize) {}
}

impl<'a> Response<'a> {
    fn read_request(&self) -> Result<(), ServerError> {
        self.req.parse(&self.conn.stream)
    }
}

pub struct Server {
    pool: ThreadPool,
    addr: String,
    handler: Option<Arc<(dyn Handler + Send + Sync)>>,
}

pub type StreamBuffer = [u8; 1024];

impl Server {
    pub fn new(size: usize) -> Self {
        let pool = ThreadPool::new(size).unwrap();

        Server {
            pool,
            handler: None,
            addr: "8080".to_string(),
        }
    }

    pub fn listen_and_serve(self) -> Result<(), ServerError> {
        let mut addr: &str = &self.addr;
        if addr == "" {
            addr = "127.0.0.1:8080";
        }
        let listener = TcpListener::bind(addr).unwrap();
        self.serve(listener)
    }

    /// ## warning
    /// after calling this method, self will moved
    fn serve(self, listener: TcpListener) -> Result<(), ServerError> {
        let srvarc = Arc::new(self);
        for stream in listener.incoming() {
            let mut stream = stream.unwrap();
            let mut c = Conn::new(srvarc.clone(), stream);
            srvarc.pool.execute(move || c.serve());
        }
        Ok(())
    }
}

struct Conn {
    server: Arc<Server>,
    stream: Arc<TcpStream>,
}

impl Conn {
    fn new(server: Arc<Server>, mut stream: TcpStream) -> Conn {
        Conn {
            server: server,
            stream: Arc::new(stream),
        }
    }

    fn serve(&self) {
        let serve_handler = ServeHandler {
            server: self.server.clone(),
        };
        serve_handler.serve_http(&Response {}, &Request::new())
    }
}

struct ServeHandler {
    server: Arc<Server>,
}

impl ServeHandler {
    pub fn serve_http(&self, rw: &impl ResponseWriter, req: &Request) {
        let mut handler: Arc<dyn Handler> = Arc::new(DefaultServeMux::new());
        if let Some(x) = &self.server.handler {
            handler = x.clone();
        }

        handler.serve_http(rw, req);
    }
}

struct NotFoundHandler;
impl Handler for NotFoundHandler {
    fn serve_http(&self, writer: &dyn ResponseWriter, req: &Request) {}
}
impl NotFoundHandler {
    fn new() -> Self {
        NotFoundHandler {}
    }
}
