use crate::{request, response, collection, alloc, err};
use std::{future::Future, io::Read, io::Write};

type Mapping = fn (&mut alloc::Allocator) -> Result<response::HttpResponse, err::Error>;

pub struct Manager {
    waiting: collection::Array<std::pin::Pin<Box<RequestHandler>>>,
    queue: collection::Array<std::pin::Pin<Box<RequestHandler>>>,
    mappings: collection::HashMap<request::RequestHeader, Mapping>,
    _context: std::sync::Arc<Context>,
    allocator: alloc::Allocator,
    //waker: std::task::Waker,
}

struct Context { }

pub struct RequestHandler {
    stream: std::sync::Arc<std::sync::Mutex<std::net::TcpStream>>,
}

impl RequestHandler {
    pub fn new(stream: std::net::TcpStream) -> RequestHandler {
        stream.set_read_timeout(Some(std::time::Duration::from_micros(1))).unwrap();

        RequestHandler {
            stream: std::sync::Arc::new(std::sync::Mutex::new(stream))
        }
    }
}

impl Future for RequestHandler {
    type Output = request::RequestHeader;

    fn poll(self: std::pin::Pin<&mut Self>, ctx: &mut std::task::Context) -> std::task::Poll<Self::Output> {
        if let Ok(mut stream) = self.stream.lock() {
            let allocator: &mut alloc::Allocator = unsafe { std::mem::transmute(ctx.waker().data() as *mut alloc::Allocator) };
            let data: *mut u8 = allocator.alloc(1024).unwrap();

            let buffer = unsafe { std::slice::from_raw_parts_mut(data, 1024) };
            match stream.read(buffer) {
                Ok(n) => {
                    let bytes = unsafe { std::slice::from_raw_parts_mut(buffer.as_mut_ptr(), n) };

                    std::task::Poll::Ready(request::RequestHeader::from_bytes(bytes, allocator).unwrap())
                },
                Err(_) => {
                    allocator.dealloc(data, 1024);
                    std::task::Poll::Pending
                }
            }
        } else {
            std::task::Poll::Pending
        }
    }
}

impl Manager {
    pub fn new(parent_allocator: &mut alloc::Allocator) -> Result<Manager, err::Error> {
        let mut allocator = parent_allocator.child(4 * 4096)?;
        let mut mappings = collection::HashMap::new(20, &mut allocator)?;

        mappings.insert(request::RequestHeader::new(request::HttpMethod::Get, b"/hello", 1, &mut allocator), hello as Mapping)?;
        mappings.insert(request::RequestHeader::new(request::HttpMethod::Get, b"/", 1, &mut allocator), root as Mapping)?;

        Ok(Manager {
            waiting: collection::Array::new(20, &mut allocator)?,
            queue: collection::Array::new(20, &mut allocator)?,
            mappings,
            _context: std::sync::Arc::new(Context {}),
            allocator,
            //waker,
        })
    }

    pub fn next(&mut self) -> bool {
        if let Some(mut f) = self.queue.pop() {
            let mut allocator = self.allocator.child(4096).unwrap();

            let waker = unsafe { std::task::Waker::from_raw(pointer(&mut allocator)) };
            let mut context = std::task::Context::from_waker(&waker);

            match f.as_mut().poll(&mut context) {
                std::task::Poll::Pending => {
                    self.waiting.push(f).unwrap();
                },

                std::task::Poll::Ready(header) => {
                    if let Some(mapping) = self.mappings.get(&header) {
                        let res = mapping(&mut allocator).unwrap();
                        f.stream.lock().unwrap().write(res.body()).unwrap();
                    } else {
                        println!("Did not find anything for: {}", header);
                    }
                }
            }

            self.allocator.dealloc(allocator.bytes(), allocator.capacity());

            true
        } else {
            false
            //self.swap()
        }
    }

    pub fn swap(&mut self) {
        std::mem::swap(&mut self.waiting, &mut self.queue);
    }

    pub fn has_waiting(&self) -> bool {
        self.waiting.len() > 0
    }

    pub fn append(&mut self, fut: RequestHandler) {
        self.waiting.push(Box::pin(fut)).unwrap();
    }
}

fn pointer<T>(data: *mut T) -> std::task::RawWaker {
    std::task::RawWaker::new((data as *const T) as *const (), &VTABLE)
}

static VTABLE: std::task::RawWakerVTable = std::task::RawWakerVTable::new(clone_fn, wake_fn, wake_by_ref_fn, drop_fn);
const unsafe fn clone_fn(data: *const ()) -> std::task::RawWaker {
    std::task::RawWaker::new(data, &VTABLE)
}

const unsafe fn wake_fn(_: *const ()) {
    //let data = &*(data as *const DataType);
}

const unsafe fn wake_by_ref_fn(_: *const ()) {
    // println!("Wake by ref fn");
}

const unsafe fn drop_fn(_: *const  ()) {
    // println!("Drop fn");
}

fn root(allocator: &mut alloc::Allocator) -> Result<response::HttpResponse, err::Error> {
    let res = b"
        <!DOCTYPE html>
        <h1>Hello world</h1>
    ";
    Ok(response::HttpResponse::new(1, response::HttpStatus::Error, res, allocator)?)
}

fn hello(allocator: &mut alloc::Allocator) -> Result<response::HttpResponse, err::Error> {
    let res = b"
        <!DOCTYPE html>
        <h1>Hello world</h1>
    ";
    Ok(response::HttpResponse::new(1, response::HttpStatus::Ok, res, allocator)?)
}
