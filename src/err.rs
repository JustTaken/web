#[derive(Debug)]
pub enum Error {
    Allocation,
    OutOfBounds,
    Connect,
    Parsing,
    HttpMethod,
    Protocol,
    HttpVersion,
}
