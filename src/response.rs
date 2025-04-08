use crate::{http, collection, alloc, err};

#[derive(Debug)]
pub enum HttpStatus {
    Ok,
    Error,
}

pub struct HttpResponse {
    status: HttpStatus,
    version: http::Version,
    body: collection::Array<u8>,
}

impl HttpResponse {
    pub fn new(version: http::Version, status: HttpStatus, content: http::Content, allocator: &mut alloc::Allocator) -> Result<HttpResponse, err::Error> {
        let mut body: collection::Array<u8> = collection::Array::new(1024, allocator)?;
        body.append_slice(b"HTTP/")?;

        match version {
            http::Version::OneOne => body.append_slice(b"1.1 ")?,
            _ => return Err(err::Error::HttpVersion),
        }

        match status {
            HttpStatus::Ok => body.append_slice(b"200 OK")?,
            HttpStatus::Error => body.append_slice(b"404 ERROR")?,
        }

        if let Some(b) = content.bytes() {
            body.append_slice(b"\r\n")?;

            match content {
                http::Content::Html(_) => {
                    body.append_slice(b"Content-Type: text/html\r\n")?;
                }
                _ => return Err(err::Error::Parsing),
            }

            body.append_slice(b"Content-Length: ")?;
            body.parse(b.len())?;
            body.append_slice(b"\r\n\r\n")?;
            body.append_slice(b)?;

            println!("body: {}", std::str::from_utf8(body.slice()).unwrap());
        }


        Ok(HttpResponse {
            status,
            version,
            body,
        })
    }

    pub fn body(&self) -> &[u8] {
        self.body.slice()
    }
}

impl std::fmt::Display for HttpResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Status: {:?}, ProtocolVersion: {:?}, {}", self.status, self.version, std::str::from_utf8(self.body.slice()).unwrap())
    }
}
