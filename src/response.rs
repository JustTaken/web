use crate::{collection, alloc, err};

#[derive(Debug)]
pub enum HttpStatus {
    Ok,
    Error,
}

pub struct HttpProtocolVersion(u32);

pub struct HttpResponse {
    status: HttpStatus,
    version: HttpProtocolVersion,
    body: collection::Array<u8>,
}

impl HttpResponse {
    pub fn new(version: u32, status: HttpStatus, content: &[u8], allocator: &mut alloc::Allocator) -> Result<HttpResponse, err::Error> {
        let mut body: collection::Array<u8> = collection::Array::new(1024, allocator)?;
        body.append_slice(b"HTTP/")?;

        match version {
            1 => body.append_slice(b"1.1 ")?,
            _ => return Err(err::Error::HttpVersion),
        }

        match status {
            HttpStatus::Ok => body.append_slice(b"200 OK")?,
            HttpStatus::Error => body.append_slice(b"404 ERROR")?,
        }

        if content.len() > 0 {
            body.append_slice(b"\r\n")?;
            body.append_slice(b"Content-Length: ")?;
            body.parse(content.len())?;
            body.append_slice(b"\r\n\r\n")?;
            body.append_slice(content)?;
        }

        let version = HttpProtocolVersion(version);

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
        write!(f, "Status: {:?}, ProtocolVersion: {}, {}", self.status, self.version.0, std::str::from_utf8(self.body.slice()).unwrap())
    }
}
