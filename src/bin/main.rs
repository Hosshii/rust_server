use rust_server::error::ServerError;
// use rust_server::header::Header;
use rust_server::message::{Header, Request, ResponseBody, ResponseWriter};
use rust_server::method::Method;
use rust_server::server::{DefaultServeMux, Handler, ServeMux, Server};
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

fn main() -> Result<(), ServerError> {
    let mut m = DefaultServeMux::new();
    m.handle(Method::Get, "/hello".to_string(), Arc::new(Index::new()));
    m.handle(Method::Get, "/sleep".to_string(), Arc::new(Sleep::new()));
    let s = Server::new(8, "127.0.0.1:7878".to_string(), Arc::new(m));
    s.listen_and_serve()
}

struct Index;
impl Handler for Index {
    fn serve_http(
        &self,
        writer: &mut dyn ResponseWriter,
        req: &Request,
    ) -> Result<(), ServerError> {
        println!("not found");
        let mut headers: Header = HashMap::new();
        headers.insert("x-my-headers".to_string(), "hello world".to_string());
        writer.header(headers);

        let mut file = File::open("hello.html").unwrap();
        let mut not_found_html = String::new();
        file.read_to_string(&mut not_found_html).unwrap();

        writer.write(ResponseBody::StringBody(not_found_html));
        writer.write_header(404);
        writer.send();
        Ok(())
    }
}
impl Index {
    const fn new() -> Self {
        Index {}
    }
}

struct Sleep;
impl Handler for Sleep {
    fn serve_http(
        &self,
        writer: &mut dyn ResponseWriter,
        req: &Request,
    ) -> Result<(), ServerError> {
        let time = 5;
        println!("sleep received");
        println!("{} second sleeping", time);
        thread::sleep(Duration::from_secs(time));
        let mut file = File::open("hello.html").unwrap();
        let mut not_found_html = String::new();
        file.read_to_string(&mut not_found_html).unwrap();

        writer.write(ResponseBody::StringBody(not_found_html));
        writer.write_header(404);
        writer.send();
        Ok(())
    }
}
impl Sleep {
    const fn new() -> Self {
        Sleep {}
    }
}
