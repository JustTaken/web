use crate::{alloc, err, manager};

#[derive(Debug, Clone, Copy)]
pub enum Version {
    One,
    OneOne,
}

#[derive(Debug, Clone, Copy)]
pub enum Method {
    Get,
    Post,
}

pub enum Content<'a> {
    None,
    Html(&'a [u8]),
    Json(&'a [u8]),
}

impl<'a> Content<'a> {
    pub fn bytes(&self) -> Option<&'a [u8]> {
        match self {
            Content::Html(b) => Some(b),
            Content::Json(b) => Some(b),
            Content::None => None
        }
    }
}

pub struct Connection {
    listener: std::net::TcpListener,
}

impl Connection {
    pub fn new(addr: &str) -> Result<Connection, err::Error> {
        let listener = std::net::TcpListener::bind(addr).map_err(|_| err::Error::Connect)?;
        listener.set_nonblocking(true).map_err(|_| err::Error::Connect)?;

        Ok(Connection {
            listener,
        })
    }

    pub fn handle_connections(&mut self, allocator: &mut alloc::Allocator) -> Result<(), err::Error> {
        let mut executor = manager::Manager::new(allocator)?;

        for stream in self.listener.incoming() {
            match stream {
                Ok(stream) => executor.append(manager::RequestHandler::new(stream)?),
                Err(e) => {
                    if let std::io::ErrorKind::WouldBlock = e.kind() {
                        if !executor.has_waiting() {
                            std::thread::sleep(std::time::Duration::from_millis(10));
                            continue;
                        }
                    } else {
                        return Err(err::Error::Parsing);
                    }
                },
            }

            executor.swap();
            while executor.next() {}
        }

        Ok(())
    }
}

impl Version {
    pub fn from_bytes(opt: Option<&[u8]>) -> Result<Version, err::Error> {
        let Some(bytes) = opt else {
            return Err(err::Error::Parsing);
        };

        let mut iter = bytes.split(|&b| b == b'/');

        let Some(http_string) = iter.next() else {
            return Err(err::Error::Parsing);
        };

        let Some(version) = iter.next() else {
            return Err(err::Error::Parsing);
        };

        if http_string != b"HTTP" {
            return Err(err::Error::Protocol);
        }

        match version {
            b"1.1" => Ok(Version::OneOne),
            b"1.0" => Ok(Version::One),
            _ => Err(err::Error::HttpVersion),
        }
    }
}

impl Method {
    pub fn from_bytes(opt: Option<&[u8]>) -> Result<Method, err::Error> {
        let Some(bytes) = opt else {
            return Err(err::Error::Parsing);
        };

        match bytes {
            b"GET" => Ok(Method::Get),
            b"POST" => Ok(Method::Post),
            _ => Err(err::Error::HttpMethod),
        }
    }

    pub fn eq(&self, other: &Method) -> bool {
        *self as u32 == *other as u32
    }
}

