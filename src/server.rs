use crate::error::ServerError;
use crate::message::Conn;
use crate::message::Header;
use crate::message::ResponseWriter;
use crate::message::{Request, ResponseBody};
use crate::method::Method;
use crate::worker::ThreadPool;
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::net::TcpListener;

use std::sync::Arc;

const NOT_FOUND_HANDLER: NotFoundHandler = NotFoundHandler::new();

pub trait HandlerServeMux: Handler + ServeMux {}

pub trait Handler {
    fn serve_http(&self, writer: &mut dyn ResponseWriter, req: &Request)
        -> Result<(), ServerError>;
}

pub trait ServeMux {
    fn handle(&mut self, method: Method, pattern: String, handler: Arc<dyn Handler + Send + Sync>);
}

struct GetEntry {
    path: String,
    handler: Arc<dyn Handler + Send + Sync>,
}
impl GetEntry {
    fn new(path: String, handler: Arc<dyn Handler + Send + Sync>) -> Self {
        GetEntry { path, handler }
    }
}

struct PostEntry {
    path: String,
    handler: Arc<dyn Handler + Send + Sync>,
}
impl PostEntry {
    fn new(path: String, handler: Arc<dyn Handler + Send + Sync>) -> Self {
        PostEntry { path, handler }
    }
}

pub struct DefaultServeMux {
    get: HashMap<String, GetEntry>,
    post: HashMap<String, PostEntry>,
}

impl HandlerServeMux for DefaultServeMux {}

impl Handler for DefaultServeMux {
    fn serve_http(
        &self,
        writer: &mut dyn ResponseWriter,
        req: &Request,
    ) -> Result<(), ServerError> {
        self.handler(req).serve_http(writer, req)
    }
}
impl ServeMux for DefaultServeMux {
    fn handle(&mut self, method: Method, pattern: String, handler: Arc<dyn Handler + Send + Sync>) {
        match method {
            Method::Get => {
                let e = GetEntry::new(pattern, handler);
                self.get.insert(e.path.clone(), e);
            }
            Method::Post => {
                let e = PostEntry::new(pattern, handler);
                self.post.insert(e.path.clone(), e);
            }
            _ => {}
        }
    }
}

impl DefaultServeMux {
    pub fn new() -> Self {
        DefaultServeMux {
            get: HashMap::new(),
            post: HashMap::new(),
        }
    }

    // fn handler(&self, r: &Request) -> impl Handler {
    fn handler(&self, r: &Request) -> Arc<dyn Handler> {
        match r.method {
            Method::Get => {
                if let Some(x) = self.get.get(&r.path) {
                    x.handler.clone()
                } else {
                    Arc::new(NOT_FOUND_HANDLER)
                }
                // Box::new(NotFoundHandler::new())
            }
            _ => Arc::new(NOT_FOUND_HANDLER),
        }
    }
}

pub struct Server {
    pool: ThreadPool,
    addr: String,
    handler: Arc<(dyn HandlerServeMux + Send + Sync)>,
}

pub type StreamBuffer = [u8; 1024];

impl Server {
    pub fn new(size: usize, addr: String, handler: Arc<dyn HandlerServeMux + Send + Sync>) -> Self {
        let pool = ThreadPool::new(size).unwrap();
        Server {
            pool,
            handler,
            addr,
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
            let stream = stream.unwrap();
            let c = Conn::new(srvarc.clone(), stream);
            srvarc.pool.execute(move || {
                c.serve();
            });
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

    pub fn serve_http(
        &self,
        rw: &mut dyn ResponseWriter,
        req: &Request,
    ) -> Result<(), ServerError> {
        self.server.handler.serve_http(rw, req)
    }
}

struct NotFoundHandler;
impl Handler for NotFoundHandler {
    fn serve_http(
        &self,
        writer: &mut dyn ResponseWriter,
        _req: &Request,
    ) -> Result<(), ServerError> {
        println!("not found");
        let mut headers: Header = HashMap::new();
        headers.insert("x-my-headers".to_string(), "hello world".to_string());
        writer.header(headers);

        let mut file = File::open("404.html").unwrap();
        let mut not_found_html = String::new();
        file.read_to_string(&mut not_found_html).unwrap();

        writer.write(ResponseBody::StringBody(not_found_html));
        writer.write_header(404);
        writer.send();
        Ok(())
    }
}
impl NotFoundHandler {
    const fn new() -> Self {
        NotFoundHandler {}
    }
}
