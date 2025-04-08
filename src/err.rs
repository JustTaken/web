#[derive(Debug)]
pub enum Error {
    Allocation,
    OutOfBounds,
    FileNotFound,
    Connect,
    Parsing,
    HttpMethod,
    Protocol,
    HttpVersion,
}
