use web::{ http, alloc };

fn main() {
    let mut allocator = alloc::Allocator::new(20 * 4096);
    let mut connection = http::Connection::new("127.0.0.1:8080").unwrap();

    connection.handle_connections(&mut allocator).unwrap();
}
