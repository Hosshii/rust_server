use rust_server::message::Message;
use rust_server::method::Method;
use std::fs::File;
use std::io::prelude::*;
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
        index(stream);
        println!(
            "{}",
            format!(
                "method: {}, path: {}, version: {}",
                m.method, m.path, m.version
            )
        );
        println!("{:?}", m.headers);
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

fn index(mut stream: TcpStream) -> Result<(), String> {
    println!("index received");

    let (status_line, filename) = ("HTTP/1.1 200 OK\r\n\r\n", "hello.html");
    let mut file = File::open(filename).unwrap();

    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    let response = format!("{}{}", status_line, contents);

    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
    Ok(())
}
