use crate::error::ServerError;
use crate::request::Request;
use crate::worker::ThreadPool;
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;

pub trait Handler {
    fn serve_http(&self, writer: &dyn ResponseWriter, req: &Request);
}

pub struct DefaultHandler;

impl Handler for DefaultHandler {
    fn serve_http(&self, writer: &dyn ResponseWriter, req: &Request) {}
}

impl DefaultHandler {
    fn new() -> Self {
        DefaultHandler {}
    }
}

pub struct Writer {}
impl ResponseWriter for Writer {
    fn write(&self, data: Vec<u8>) {}
}

pub trait ResponseWriter {
    fn write(&self, data: Vec<u8>);
}

pub struct Server<'a> {
    pool: ThreadPool,
    get_handle: HashMap<String, fn(TcpStream) -> Result<(), String>>,
    addr: String,
    handler: Option<&'a dyn Handler>,
}

pub type StreamBuffer = [u8; 1024];

impl<'a> Server<'a> {
    pub fn new(size: usize) -> Server<'a> {
        let pool = ThreadPool::new(size).unwrap();

        Server {
            pool,
            get_handle: HashMap::new(),
            handler: None,
            addr: "8080".to_string(),
        }
    }

    pub fn listen_and_serve(&self) -> Result<(), ServerError> {
        let mut addr: &str = &(self.addr);
        if addr == "" {
            addr = "127.0.0.1:8080";
        }
        let listener = TcpListener::bind(addr).unwrap();
        self.serve(listener)
    }

    fn serve(&self, listener: TcpListener) -> Result<(), ServerError> {
        for stream in listener.incoming() {
            let mut stream = stream.unwrap();
            let srvarc = Arc::new(*self);
            let mut c = Conn::new(srvarc, stream);
            self.pool.execute(|| c.serve());
        }
        Ok(())
    }
}

struct Conn<'a> {
    server: Arc<Server<'a>>,
    stream: Arc<TcpStream>,
}

impl<'a> Conn<'a> {
    fn new(server: Arc<Server<'a>>, stream: TcpStream) -> Conn<'a> {
        Conn {
            server: server,
            stream: Arc::new(stream),
        }
    }

    fn serve(&self) {
        let serve_handler = ServeHandler {
            server: &self.server,
        };
        // serve_handler.serve_http(Writer {}, &Request::new())
    }
}

struct ServeHandler<'a> {
    server: &'a Server<'a>,
}

impl<'a> ServeHandler<'a> {
    pub fn serve_http(&self, rw: &impl ResponseWriter, req: &Request) {
        let mut handler: &dyn Handler = &DefaultHandler::new();

        if let Some(x) = self.server.handler {
            handler = x;
        }

        handler.serve_http(rw, req);
    }
}
