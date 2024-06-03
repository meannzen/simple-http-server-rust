use std::{
    collections::HashMap,
    io::{Read, Write},
    net::TcpStream,
};

#[derive(Debug)]
struct HttpRequst {
    method: String,
    path: String,
    version: String,
    header: HashMap<String, String>,
}

impl HttpRequst {
    fn new() -> Self {
        HttpRequst {
            method: String::new(),
            path: String::new(),
            version: String::new(),
            header: HashMap::new(),
        }
    }

    fn from_raw(raw_data: Vec<&str>) -> Self {
        let mut request = HttpRequst::new();
        if raw_data.len() < 3 {
            return request;
        }
        request.method = raw_data[0].to_string();
        request.path = raw_data[1].to_string();
        request.version = raw_data[2].to_string();
        let mut i = 3;
        while i < raw_data.len() {
            if i + 1 < raw_data.len() {
                let key = raw_data[i].trim_end_matches(':').to_string();
                let value = raw_data[i + 1].to_string();
                request.header.entry(key).or_insert(value);
            }
            i += 2;
        }

        request
    }
}

fn read_stream(mut stream: &TcpStream) -> Option<HttpRequst> {
    let mut buffer = [0; 1025];
    let read_result = stream.read(&mut buffer);
    match read_result {
        Ok(bytes_read) => {
            let cow_buff = String::from_utf8_lossy(&buffer[..bytes_read]);
            let raw_data: Vec<&str> = cow_buff.split_whitespace().collect();
            let http_request = HttpRequst::from_raw(raw_data);
            println!("result:{:?}", http_request);
            Some(http_request)
        }
        _ => None,
    }
}

fn handle_client(stream: &mut TcpStream) -> anyhow::Result<()> {
    let http_request_option = read_stream(stream);
    match http_request_option {
        Some(http_request) => match http_request.path.as_str() {
            "/" => {
                let _ = stream.write("HTTP/1.1 200 OK\r\n\r\n".as_bytes());
            }
            path => {
                if path == "/echo" || path.starts_with("/echo/") {
                    if path == "/echo" {
                        let _ = stream.write("HTTP/1.1 200 OK\r\n\r\n".as_bytes());
                    } else if let Some(content) = path.strip_prefix("/echo/") {
                        let content_lenght = content.len();
                        let response = format!("HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}", content_lenght, content);
                        let _ = stream.write(response.as_bytes());
                    } else {
                        let _ = stream.write("HTTP/1.1 200 OK\r\n\r\n".as_bytes());
                    }
                } else if path == "/user-agent" || path == "/user-agent/" {
                    if let Some(content) = http_request.header.get("User-Agent") {
                        let response = format!("HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}", content.len(), content);
                        let _ = stream.write(response.as_bytes());
                    } else {
                        let _ = stream.write("HTTP/1.1 200 OK\r\n\r\n".as_bytes());
                    }
                } else {
                    let _ = stream.write("HTTP/1.1 404 Not Found\r\n\r\n".as_bytes());
                }
            }
        },
        None => {
            let _ = stream.write("HTTP/1.1 404 Not Found\r\n\r\n".as_bytes());
        }
    }

    Ok(())
}

fn main() -> std::io::Result<()> {
    let listener = std::net::TcpListener::bind("127.0.0.1:4221")?;
    for stream in listener.incoming() {
        match stream {
            Ok(mut s) => {
                let _ = handle_client(&mut s);
            }
            Err(e) => {
                println!("Cannot handle request:{:?}", e);
            }
        }
    }

    Ok(())
}
