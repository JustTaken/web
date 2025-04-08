use crate::{alloc, err, manager};

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
                Ok(stream) => executor.append(manager::RequestHandler::new(stream)),
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

