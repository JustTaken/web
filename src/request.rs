use crate::{alloc, collection, err::Error};

pub struct RequestHeader {
    method: HttpMethod,
    end_point: EndPoint,
    version: HttpProtocolVersion,
}

#[derive(Debug, Clone, Copy)]
pub enum HttpMethod {
    Get,
    Post,
}

pub struct EndPoint(collection::Array<u8>);
pub struct HttpProtocolVersion(u32);

impl RequestHeader {
    pub fn new(method: HttpMethod, end: &[u8], version: u32, allocator: &mut alloc::Allocator) -> RequestHeader {
        let end_point = EndPoint::from_bytes(Some(end), allocator).unwrap();
        let protocol = HttpProtocolVersion(version);

        RequestHeader {
            method, end_point, version: protocol,
        }
    }
    pub fn from_bytes(bytes: &[u8], allocator: &mut alloc::Allocator) -> Result<RequestHeader, Error> {
        let mut iter = bytes.split(|&b| b == b' ' || b == b'\r');

        Ok(RequestHeader {
            method: HttpMethod::from_bytes(iter.next())?,
            end_point: EndPoint::from_bytes(iter.next(), allocator)?,
            version: HttpProtocolVersion::from_bytes(iter.next())?,
        })
    }
}

impl collection::Hash for RequestHeader {
    fn eq(&self, other: &RequestHeader) -> bool {
        self.method.eq(&other.method) && self.end_point.0.eq(&other.end_point.0)
    }

    fn hash(&self) -> usize {
        let slice = self.end_point.0.slice();

        let mut count: usize = 0;
        for s in slice {
            count += *s as usize;
        }

        let version = self.version.0 << 30;
        count |= version as usize;

        count
    }

    fn is_zero(&self) -> bool {
        self.end_point.0.cap() == 0
    }
}

impl HttpMethod {
    fn from_bytes(opt: Option<&[u8]>) -> Result<HttpMethod, Error> {
        let Some(bytes) = opt else {
            return Err(Error::Parsing);
        };

        match bytes {
            b"GET" => Ok(HttpMethod::Get),
            b"POST" => Ok(HttpMethod::Post),
            _ => Err(Error::HttpMethod),
        }
    }

    fn eq(&self, other: &HttpMethod) -> bool {
        *self as u32 == *other as u32
    }
}

impl EndPoint {
    fn from_bytes(opt: Option<&[u8]>, allocator: &mut alloc::Allocator) -> Result<EndPoint, Error> {
        let Some(bytes) = opt else {
            return Err(Error::Parsing);
        };

        let mut string = collection::Array::new(bytes.len(), allocator)?;

        string.copy(bytes)?;

        Ok(EndPoint(string))
    }
}

impl HttpProtocolVersion {
    fn from_bytes(opt: Option<&[u8]>) -> Result<HttpProtocolVersion, Error> {
        let Some(bytes) = opt else {
            return Err(Error::Parsing);
        };

        let mut iter = bytes.split(|&b| b == b'/');

        let Some(http_string) = iter.next() else {
            return Err(Error::Parsing);
        };

        let Some(version) = iter.next() else {
            return Err(Error::Parsing);
        };

        if http_string != b"HTTP" {
            return Err(Error::Protocol);
        }

        match version {
            b"1.1" => Ok(HttpProtocolVersion(1)),
            b"1.0" => Ok(HttpProtocolVersion(1)),
            _ => Err(Error::HttpVersion),
        }
    }
}

impl std::fmt::Display for RequestHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Method: {:?}, ProtocolVersion: {}, EndPoint: {}", self.method, self.version.0, std::str::from_utf8(self.end_point.0.slice()).unwrap())
    }
}
