use rust_server::server::Server;
use std::fs::File;
use std::io::prelude::*;
use std::net::TcpStream;
use std::thread;
use std::time::Duration;

fn main() {
    // let mut s = Server::new(8);
    // s.GET("/", index);
    // s.GET("/ping", pong);
    // s.GET("/sleep", sleep);
    // s.start(7878);
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

fn pong(mut stream: TcpStream) -> Result<(), String> {
    println!("ping received");

    let (status_line, filename) = ("HTTP/1.1 200 OK\r\n\r\n", "pong.html");
    let mut file = File::open(filename).unwrap();

    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    let response = format!("{}{}", status_line, contents);

    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
    Ok(())
}

fn sleep(mut stream: TcpStream) -> Result<(), String> {
    let time = 5;
    println!("sleep received");
    println!("{} second sleeping", time);
    thread::sleep(Duration::from_secs(time));

    let (status_line, filename) = ("HTTP/1.1 200 OK\r\n\r\n", "hello.html");
    let mut file = File::open(filename).unwrap();

    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    let response = format!("{}{}", status_line, contents);

    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
    Ok(())
}
