use std::{
    collections::HashMap,
    io::{BufRead, Cursor, Read, Write},
    str::FromStr,
};

#[derive(Debug, PartialEq, Eq)]
pub struct ParseSteamError(String);

#[derive(Default, PartialEq, Debug, Clone, Copy, Eq)]
pub enum StatusCode {
    #[default]
    OK,
    Created,
    NotFound,
}

impl std::fmt::Display for StatusCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let status = match &self {
            StatusCode::OK => "200 OK",
            StatusCode::Created => "201 Created",
            _ => "404 Not Found",
        };
        write!(f, "{status}")
    }
}
#[derive(Debug, Default, Clone)]
pub struct Response {
    status: StatusCode,
    header: HashMap<String, String>,
    body: Vec<u8>,
}
impl Response {
    pub fn ok() -> Response {
        Response {
            status: StatusCode::OK,
            ..Default::default()
        }
    }

    pub fn not_found() -> Response {
        Response {
            status: StatusCode::NotFound,
            ..Default::default()
        }
    }

    pub fn created() -> Response {
        Response {
            status: StatusCode::Created,
            ..Default::default()
        }
    }

    pub fn set_header(mut self, key: &str, value: &str) -> Self {
        self.header.insert(key.to_owned(), value.to_owned());
        self
    }

    pub fn set_body(mut self, body: impl AsRef<[u8]>) -> Self {
        self.body.extend_from_slice(body.as_ref());
        self
    }

    pub fn write(mut self, mut writer: impl Write) -> Result<(), std::io::Error> {
        self.header
            .entry("Content-Length".to_string())
            .or_insert(self.body.len().to_string());
        let status_line = format!("HTTP/1.1 {}\r\n", self.status);
        writer.write_all(status_line.as_bytes())?;
        for (k, v) in self.header.into_iter() {
            writer.write_all(format!("{k}: {v}\r\n").as_bytes())?;
        }

        writer.write_all(b"\r\n")?;
        // body
        writer.write_all(&self.body)?;

        writer.flush()?;

        Ok(())
    }
}
impl FromStr for Method {
    type Err = ParseSteamError;
    fn from_str(method: &str) -> Result<Self, Self::Err> {
        match method {
            "GET" => Ok(Method::GET),
            "POST" => Ok(Method::POST),
            _ => Err(ParseSteamError(format!("Invalid Http Method {}", method))),
        }
    }
}

#[derive(Debug, Default)]
pub enum Method {
    #[default]
    GET,
    POST,
}

#[derive(Debug, Default)]
pub struct Request {
    pub method: Method,
    pub path: String,
    pub header: HashMap<String, String>,
    pub body: Vec<u8>,
}

pub fn parse_request(buf: &[u8]) -> Result<Request, ParseSteamError> {
    let mut http_request: Vec<String> = vec![];
    let mut cursor = Cursor::new(buf);
    for line in cursor.by_ref().lines() {
        let line = line.map_err(|e| ParseSteamError(e.to_string()))?;
        if line.is_empty() {
            break;
        }
        http_request.push(line);
    }

    if http_request.is_empty() {
        return Err(ParseSteamError("Invalid request".to_string()));
    }

    let parts: Vec<_> = http_request[0].split(' ').collect();
    if parts.len() != 3 {
        return Err(ParseSteamError("Invalid request".to_string()));
    }

    let method: Method = parts[0].parse()?;
    let path = parts[1].to_string();
    let header: HashMap<String, String> = http_request[1..]
        .iter()
        .filter_map(|line| {
            line.split_once(": ")
                .map(|(k, v)| (k.to_owned(), v.to_owned()))
        })
        .collect();

    let mut body = Vec::new();
    if let Some(length) = header.get("Content-Length") {
        if let Ok(n) = length.parse() {
            body.resize(n, 0);
            cursor
                .read_exact(&mut body)
                .map_err(|e| ParseSteamError(e.to_string()))?;
        };
    }

    Ok(Request {
        method,
        path,
        header,
        body,
    })
}
