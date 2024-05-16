// Uncomment this block to pass the first stage
use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
};
#[derive(Debug)]
enum HttpMethod {
    GET,
    POST,
    PATCH,
    DELETE,
    OTHER,
}

impl HttpMethod {
    fn new(method: Option<&str>) -> Self {
        if let Some(m) = method {
            match m {
                "GET" => HttpMethod::GET,
                "POST" => HttpMethod::POST,
                "PATCH" => HttpMethod::PATCH,
                "DELETE" => HttpMethod::DELETE,
                _ => HttpMethod::OTHER,
            }
        } else {
            HttpMethod::OTHER
        }
    }
}

#[derive(Debug)]
struct Route {
    method: HttpMethod,
    url: String,
}

impl<'a> From<std::borrow::Cow<'a, str>> for Route {
    fn from(value: std::borrow::Cow<str>) -> Self {
        let mut spilt = value.split_whitespace();
        let method = HttpMethod::new(spilt.next());
        let url = match spilt.next() {
            Some(s) => s.to_string(),
            _ => "".to_string(),
        };
        Route { method, url }
    }
}

fn handle_connection(stream: &mut TcpStream) -> Result<(), std::io::Error> {
    let mut response = "HTTP/1.1 200 OK\r\n\r\n";
    let mut buffer = [0; 1024];
    match stream.read(&mut buffer) {
        Ok(_) => {
            let request: std::borrow::Cow<str> = String::from_utf8_lossy(&buffer);
            let route: Route = request.into();
            let _url = "/".to_string();
            match (route.method, route.url.len()) {
                (HttpMethod::GET, 1) => {
                   
                }
                _ => {
                    response = "HTTP/1.1 404 Not Found\r\n\r\n";
                }
            }
        }
        _ => {}
    }
    stream.write_all(response.as_bytes())
}

fn main() {
    let listener: TcpListener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                println!("accepted new connection...");
                let _ = handle_connection(&mut stream);
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
