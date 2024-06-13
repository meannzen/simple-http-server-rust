use core::panic;
use http_server_starter_rust::{parse_request, Method, Request, Response, ThreadPool};
use std::{
    env,
    fs::File,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();
    let pool = ThreadPool::new(4);
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        pool.execute(|| handle_connection(stream));
    }
}

fn handle_connection(mut stream: TcpStream) {
    let request = match parse_request(&mut stream) {
        Ok(r) => r,
        Err(_) => Default::default(),
    };

    match handle_request(request) {
        Ok(response) => {
            let _ = response.write(stream);
        }
        Err(_) => {
            panic!("Server error")
        }
    }
}

fn handle_request(request: Request) -> std::io::Result<Response> {
    let response = if &request.path == "/" {
        Response::ok()
    } else if let Some(content) = request.path.strip_prefix("/echo/") {
        Response::ok()
            .set_header("Content-Type", "text/plain")
            .set_header("Content-Length", content.len().to_string().as_str())
            .set_body(content.as_bytes())
    } else if &request.path == "/user-agent" {
        let user_agent = match request.header.get("User-Agent") {
            Some(value) => value.to_owned(),
            None => "".to_owned(),
        };

        Response::ok()
            .set_header("Content-type", "text/plain")
            .set_header("Content-Length", user_agent.len().to_string().as_str())
            .set_body(user_agent.as_bytes())
    } else if let Some(file_name) = request.path.strip_prefix("/files/") {
        let dir = match env::args().nth(2) {
            Some(path) => path,
            _ => "/".to_string(),
        };

        let file_path = format!("{dir}/{file_name}");

        let response = match request.method {
            Method::GET => {
                let content = match File::open(&file_path) {
                    Ok(mut file) => {
                        let mut buf = String::new();
                        match file.read_to_string(&mut buf) {
                            Ok(_) => Some(buf),
                            Err(_) => None,
                        }
                    }
                    Err(_) => None,
                };

                let response = match content {
                    Some(text) => Response::ok()
                        .set_header("Content-Type", "application/octet-stream")
                        .set_header("Content-Length", text.len().to_string().as_str())
                        .set_body(text.as_bytes()),
                    None => Response::not_found(),
                };

                response
            }
            Method::POST => {
                let result = match File::create(&file_path) {
                    Ok(mut file) => {
                        let writen = match file.write_all(&request.body) {
                            Ok(_) => Ok(()),
                            Err(_) => Err(()),
                        };
                        writen
                    }
                    Err(_) => Err(()),
                };
                let response = match result {
                    Ok(_) => Response::created(),
                    _ => Response::not_found(),
                };
                response
            }
        };

        response
    } else {
        Response::not_found()
    };

    Ok(response)
}
