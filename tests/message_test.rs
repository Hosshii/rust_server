use rust_server::message::Message;
use rust_server::method::Method;
use std::io::Read;
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::time::Duration;

const host: &str = "127.0.0.1";
const port: &str = "7878";
const addr: &str = "127.0.0.1:7878";

#[test]
fn parse_header() {
    let listener = TcpListener::bind(addr).unwrap();
    thread::spawn(move || {
        thread::sleep(Duration::from_secs(1));
        send_request();
    });
    for stream in listener.incoming().take(1) {
        let mut stream = stream.unwrap();
        let mut m = Message::new();
        m.parse(&stream);
        println!(
            "{}",
            format!(
                "method: {}, path: {}, version: {}",
                m.method, m.path, m.version
            )
        );
        assert_eq!(m.method, Method::Get);
        assert_eq!(m.path, "/");
        assert_eq!(m.version, "HTTP/1.1");
    }
    // unimplemented!();
}

fn send_request() {
    println!("start to sending task");
    let add = format!("http://{}", addr);
    let _ = reqwest::blocking::get(&add).unwrap();
    println!("end");
}
