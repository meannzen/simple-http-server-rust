use codecrafters_http_server::{parse_request, Method, Request, Response};
use flate2::write::GzEncoder;
use flate2::Compression;
use std::{
    env,
    fs::File,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    thread,
};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();
    for stream in listener.incoming() {
        thread::spawn(move || {
            let stream = stream.unwrap();
            handle_connection(stream)
        });
    }

    Ok(())
}

fn handle_connection(mut stream: TcpStream) {
    let mut buf = [0u8; 1024];
    while let Ok(n) = stream.read(&mut buf) {
        if n == 0 {
            break;
        }
        let request = parse_request(&buf[0..n]).unwrap_or_default();
        match handle_request(request) {
            Ok(response) => {
                dbg!(&response);
                let _ = response.write(&stream);
            }
            Err(_) => {
                panic!("Server error")
            }
        }
    }
}

fn handle_request(request: Request) -> std::io::Result<Response> {
    let response = if &request.path == "/" {
        Response::ok()
    } else if let Some(content) = request.path.strip_prefix("/echo/") {
        let accept_encoding = request
            .header
            .get("Accept-Encoding")
            .filter(|&text| text.contains("gzip"));

        match accept_encoding {
            Some(_) => {
                let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
                encoder
                    .write_all(content.as_bytes())
                    .expect("Failed to write to encoder");
                let gzipped_data = encoder.finish().expect("Failed to finish encoding");
                Response::ok()
                    .set_header("Content-Type", "text/plain")
                    .set_header("Content-Length", gzipped_data.len().to_string().as_str())
                    .set_header("Content-Encoding", "gzip")
                    .set_body(gzipped_data)
            }
            None => Response::ok()
                .set_header("Content-Type", "text/plain")
                .set_header("Content-Length", content.len().to_string().as_str())
                .set_body(content.as_bytes()),
        }
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
                    Ok(mut file) => match file.write_all(&request.body) {
                        Ok(_) => Ok(()),
                        Err(_) => Err(()),
                    },
                    Err(_) => Err(()),
                };
                match result {
                    Ok(_) => Response::created(),
                    _ => Response::not_found(),
                }
            }
        };

        response
    } else {
        Response::not_found()
    };

    Ok(response)
}
