#![allow(clippy::all)]
use std::{
    fs,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    path::{Path, PathBuf},
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
    let raw_path = parts[1];
    let clean = raw_path
        .split('/')
        .filter(|part| !part.is_empty() && *part != "." && *part != "..")
        .collect::<Vec<&str>>()
        .join("/");
    let mut path = PathBuf::from("public/").join(&clean);
    if path.is_dir() {
        if !raw_path.ends_with('/') {
            let redirect = format!("{raw_path}/");
            let response = format!(
                "HTTP/1.1 301 Moved Permanently\r\n\
                    Location: {redirect}\r\n\
                    Content-Length: 0\r\n\
                    Connection: close\r\n\
                    \r\n"
            );
            stream.write_all(response.as_bytes()).unwrap();
            stream.flush().unwrap();
            println!("Redirecting to: {redirect}");
            return;
        }
        path = path.join("index.html");
    }
    let version = parts[2];
    println!("Request: {method} {} {version}", path.display());
    if !path.exists() {
        let _ = stream.write_all(neg_reply(BadReq::NotFound).as_bytes());
        return;
    }
    let maybe = fs::read(&path);

    if maybe.is_err() {
        let _ = stream.write_all(neg_reply(BadReq::ServerError).as_bytes());
        return;
    }
    let body = maybe.unwrap();

    let response_header = format!(
        "HTTP/1.1 200 OK\r\n\
            Content-Length: {}\r\n\
            Content-Type: {}\r\n\
            Connection: close\r\n\
            \r\n",
        body.len(),
        get_type(path.as_path()),
    );

    let _ = stream.write_all(response_header.as_bytes()).unwrap();
    let _ = stream.write_all(&body);
    stream.flush().unwrap();
    println!("Sent response, header: {response_header}");
}
fn get_type(path: &Path) -> &'static str {
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("html") | Some("htm") => "text/html",
        Some("css") => "text/css",
        Some("js") => "application/javascript",
        Some("json") => "application/json",
        Some("txt") => "text/plain",
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("svg") => "image/svg+xml",
        Some("pdf") => "application/pdf",
        Some("zip") => "application/zip",
        Some("webp") => "image/webp",
        _ => "application/octet-stream",
    }
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
    let listener = TcpListener::bind("0.0.0.0:8080").unwrap();
    println!("listening");
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => handle_client(stream),
            Err(e) => eprintln!("Failed connect: {e}"),
        }
    }
}
