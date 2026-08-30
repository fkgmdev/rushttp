#![allow(clippy::all)]
use std::{
    fs,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    path::PathBuf,
    str,
};

enum BadReq {
    NotFound,
    BadRequest,
    ServerError,
}

fn handle_client(mut stream: TcpStream) {
    let mut buffer = [0; 4096];
    if stream.read(&mut buffer).is_err() {
        let _ = stream.write_all(neg_reply(BadReq::ServerError).as_bytes());
        return;
    }

    let req = str::from_utf8(&buffer).unwrap();
    let parts: Vec<&str> = req
        .lines()
        .next()
        .unwrap_or("")
        .split_whitespace()
        .collect();
    if parts.len() < 3 {
        let _ = stream.write_all(neg_reply(BadReq::BadRequest).as_bytes());
        return;
    }
    let method = parts[0];
    let path = PathBuf::from(
        "public/".to_string()
            + parts[1]
                .split("/")
                .filter(|part| !part.is_empty() && *part != "." && *part != "..")
                .collect::<Vec<&str>>()
                .join("")
                .as_str(),
    );
    let version = parts[2];
    let path = if path.is_dir() {
        path.join("index.html")
    } else {
        path
    };
    println!("Request: {method} {} {version}", path.display());
    if !path.exists() {
        let _ = stream.write_all(neg_reply(BadReq::NotFound).as_bytes());
        return;
    }
    let maybe = fs::read_to_string(path);

    if maybe.is_err() {
        let _ = stream.write_all(neg_reply(BadReq::ServerError).as_bytes());
        return;
    }
    let body = maybe.unwrap();

    let response = format!(
        "HTTP/1.1 200 OK\r\n\
            Content-Length: {}\r\n\
            Content-Type: text/plain\r\n\
            Connection: close\r\n\
            \r\n\
            {}",
        body.len(),
        body
    );

    stream.write_all(response.as_bytes()).unwrap();
    stream.flush().unwrap();
    println!("Sent response: {response}");
}

fn neg_reply(messup: BadReq) -> String {
    let status_str = match messup {
        BadReq::NotFound => "404 Not Found",
        BadReq::BadRequest => "400 Bad Request",
        BadReq::ServerError => "500 Internal Server Error",
    };
    let response = format!("HTTP/1.1 {status_str}\r\nContent-Length: 0\r\n\r\n");
    println!("Sending bad reply: {response}");
    response
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
    println!("listening");
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => handle_client(stream),
            Err(e) => eprintln!("Failed connect: {e}"),
        }
    }
}
