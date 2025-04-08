use crate::{http, alloc, collection, err};

pub struct RequestHeader {
    method: http::Method,
    end_point: EndPoint,
    version: http::Version,
}

pub struct EndPoint(collection::Array<u8>);

impl RequestHeader {
    pub fn new(method: http::Method, end: &[u8], protocol: http::Version, allocator: &mut alloc::Allocator) -> RequestHeader {
        let end_point = EndPoint::from_bytes(Some(end), allocator).unwrap();

        RequestHeader {
            method, end_point, version: protocol,
        }
    }

    pub fn from_bytes(bytes: &[u8], allocator: &mut alloc::Allocator) -> Result<RequestHeader, err::Error> {
        let mut iter = bytes.split(|&b| b == b' ' || b == b'\r');

        Ok(RequestHeader {
            method: http::Method::from_bytes(iter.next())?,
            end_point: EndPoint::from_bytes(iter.next(), allocator)?,
            version: http::Version::from_bytes(iter.next())?,
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

        let v = self.version as usize;
        let version: usize = v << 30;
        count |= version;

        count
    }

    fn is_zero(&self) -> bool {
        self.end_point.0.cap() == 0
    }
}

impl EndPoint {
    fn from_bytes(opt: Option<&[u8]>, allocator: &mut alloc::Allocator) -> Result<EndPoint, err::Error> {
        let Some(bytes) = opt else {
            return Err(err::Error::Parsing);
        };

        let mut string = collection::Array::new(bytes.len(), allocator)?;

        string.copy(bytes)?;

        Ok(EndPoint(string))
    }
}

impl std::fmt::Display for RequestHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Method: {:?}, ProtocolVersion: {:?}, EndPoint: {}", self.method, self.version, std::str::from_utf8(self.end_point.0.slice()).unwrap())
    }
}
