#![allow(clippy::all)]
use core::fmt;
use std::{
    fs,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    path::{Path, PathBuf},
    str,
};
#[derive(Debug)]
enum BadReq {
    NotFound,
    BadRequest,
    ServerError,
}

impl fmt::Display for BadReq {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let msg = match self {
            BadReq::NotFound => "404 not found",
            BadReq::BadRequest => "400 bad request",
            BadReq::ServerError => "500 internal server error",
        };
        write!(f, "{msg}")
    }
}
impl std::error::Error for BadReq {}
enum HttpMethod {
    GET,
    POST,
}
trait ToHttpMethod {
    fn whatmethod(&self) -> Result<HttpMethod, BadReq>;
}

impl ToHttpMethod for str {
    fn whatmethod(&self) -> Result<HttpMethod, BadReq> {
        match self.to_uppercase().as_str() {
            "GET" => Ok(HttpMethod::GET),
            "POST" => Ok(HttpMethod::POST),
            _ => Err(BadReq::BadRequest),
        }
    }
}

trait ToDisplay {
    fn getdisplay(&self) -> &str;
}

impl ToDisplay for HttpMethod {
    fn getdisplay(&self) -> &str {
        match self {
            HttpMethod::GET => "GET",
            HttpMethod::POST => "POST",
        }
    }
}

struct Request {
    path: PathBuf,
    method: HttpMethod,
    version: String,
}

fn parse_request(req: &str) -> Result<Request, BadReq> {
    let parts: Vec<&str> = req
        .lines()
        .next()
        .unwrap_or("")
        .split_whitespace()
        .collect();
    if parts.len() < 3 {
        return Err(BadReq::BadRequest);
    }
    let method = parts[0].whatmethod()?;
    let raw_path = parts[1];
    let clean = raw_path
        .split('/')
        .filter(|part| !part.is_empty() && *part != "." && *part != "..")
        .collect::<Vec<&str>>()
        .join("/");
    let path = PathBuf::from("public/").join(&clean);
    Ok(Request {
        path: path,
        method: method,
        version: parts[2].to_owned(),
    })
}
fn handle_client(mut stream: TcpStream) {
    match handle_client_inner(&mut stream) {
        Ok(reply) => {
            let (header, body) = reply;
            let _ = stream.write_all(header.as_bytes());
            if let Some(contents) = body {
                let _ = stream.write_all(&contents);
            }
        }
        Err(e) => {
            let _ = stream.write_all(neg_reply(e).as_bytes());
        }
    };
}

fn handle_client_inner(stream: &mut TcpStream) -> Result<(String, Option<Vec<u8>>), BadReq> {
    let mut buffer = [0; 4096];
    stream.read(&mut buffer).map_err(|_| BadReq::ServerError)?;

    let req = parse_request(str::from_utf8(&buffer).map_err(|_| BadReq::ServerError)?)?;

    let clean = req
        .path
        .to_str()
        .unwrap()
        .split('/')
        .filter(|part| !part.is_empty() && *part != "." && *part != "..")
        .collect::<Vec<&str>>()
        .join("/");
    let mut path = PathBuf::from("public/").join(&clean);
    if path.is_dir() {
        if !req.path.to_str().unwrap().ends_with('/') {
            let redirect = format!("{}/", req.path.display());
            let response = format!(
                "HTTP/1.1 301 Moved Permanently\r\n\
                    Location: {redirect}\r\n\
                    Content-Length: 0\r\n\
                    Connection: close\r\n\
                    \r\n"
            );
            return Ok((response, None));
        }
        path = path.join("index.html");
    }
    println!(
        "Request: {} {} {}",
        req.method.getdisplay(),
        path.display(),
        req.version,
    );
    if !path.exists() {
        return Err(BadReq::NotFound);
    }
    let maybe = fs::read(&path);

    let body = maybe.map_err(|_| BadReq::ServerError)?;

    let reply = format!(
        "HTTP/1.1 200 OK\r\n\
            Content-Length: {}\r\n\
            Content-Type: {}\r\n\
            Connection: close\r\n\
            \r\n",
        body.len(),
        get_type(path.as_path()),
    );
    Ok((reply, Some(body)))
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
