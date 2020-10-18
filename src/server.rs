use crate::error::ServerError;
use crate::message::Conn;
use crate::message::Header;
use crate::message::ResponseWriter;
use crate::message::{Message, Request, Response};
use crate::method::Method;
use crate::worker::ThreadPool;
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::rc::Rc;
use std::sync::{Arc, Mutex};

pub fn listen_and_serve(
    size: usize,
    addr: String,
    handler: Option<Arc<dyn Handler + Send + Sync>>,
) -> Result<(), ServerError> {
    let s = Server::new(size, addr, handler);
    s.listen_and_serve()
}

pub trait Handler {
    fn serve_http(&self, writer: &mut dyn ResponseWriter, req: &Request);
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
    fn serve_http(&self, writer: &mut dyn ResponseWriter, req: &Request) {
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

pub struct Server {
    pool: ThreadPool,
    addr: String,
    handler: Option<Arc<(dyn Handler + Send + Sync)>>,
}

pub type StreamBuffer = [u8; 1024];

impl Server {
    pub(crate) fn new(
        size: usize,
        addr: String,
        handler: Option<Arc<dyn Handler + Send + Sync>>,
    ) -> Self {
        let pool = ThreadPool::new(size).unwrap();

        Server {
            pool,
            handler,
            addr,
        }
    }

    pub(crate) fn listen_and_serve(self) -> Result<(), ServerError> {
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

pub(crate) struct ServeHandler {
    server: Arc<Server>,
}

impl ServeHandler {
    pub fn new(server: Arc<Server>) -> Self {
        ServeHandler { server }
    }

    pub fn serve_http(&self, rw: &mut dyn ResponseWriter, req: &Request) {
        let mut handler: Arc<dyn Handler> = Arc::new(DefaultServeMux::new());
        if let Some(x) = &self.server.handler {
            handler = x.clone();
        }

        handler.serve_http(rw, req);
    }
}

struct NotFoundHandler;
impl Handler for NotFoundHandler {
    fn serve_http(&self, writer: &mut dyn ResponseWriter, req: &Request) {
        let mut headers: Header = HashMap::new();
        headers.insert("x-my-headers".to_string(), "hello world".to_string());
        writer.header(headers);

        let mut file = File::open("404.html").unwrap();
        let mut not_found_html = String::new();
        file.read_to_string(&mut not_found_html).unwrap();

        writer.write(not_found_html.as_bytes().to_vec());
        writer.write_header(404);
        writer.send()
    }
}
impl NotFoundHandler {
    fn new() -> Self {
        NotFoundHandler {}
    }
}
