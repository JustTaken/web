use crate::{http, request, response, collection, alloc, err};
use std::{future::Future, io::{Read, Seek, Write}};

type Mapping = fn (&mut Context) -> Result<response::HttpResponse, err::Error>;

pub struct Manager {
    waiting: collection::Array<std::pin::Pin<Box<RequestHandler>>>,
    queue: collection::Array<std::pin::Pin<Box<RequestHandler>>>,
    context: std::sync::Arc<std::sync::Mutex<Context>>,
}

struct Context {
    mappings: collection::HashMap<request::RequestHeader, Mapping>,
    files: collection::Array<collection::Array<u8>>,
    allocator: alloc::Allocator,
}

pub struct RequestHandler {
    stream: std::sync::Arc<std::sync::Mutex<std::net::TcpStream>>,
}

impl RequestHandler {
    pub fn new(stream: std::net::TcpStream) -> Result<RequestHandler, err::Error> {
        stream.set_nonblocking(true).map_err(|_| err::Error::Connect)?;

        Ok(RequestHandler {
            stream: std::sync::Arc::new(std::sync::Mutex::new(stream))
        })
    }
}

impl Future for RequestHandler {
    type Output = ();

    fn poll(self: std::pin::Pin<&mut Self>, ctx: &mut std::task::Context) -> std::task::Poll<Self::Output> {
        if let Ok(mut stream) = self.stream.lock() {
            println!("Trying to pool this shit");
            let context: &mut std::sync::Arc<std::sync::Mutex<Context>> = unsafe { std::mem::transmute(ctx.waker().data() as *mut std::sync::Arc<std::sync::Mutex<Context>>) };
            let mut context = context.lock().unwrap();

            let data: *mut u8 = context.allocator.alloc(1024).unwrap();

            let buffer = unsafe { std::slice::from_raw_parts_mut(data, 1024) };
            match stream.read(buffer) {
                Ok(n) => {
                    let bytes = unsafe { std::slice::from_raw_parts_mut(buffer.as_mut_ptr(), n) };
                    let header = request::RequestHeader::from_bytes(bytes, &mut context.allocator).unwrap();

                    if let Some(mapping) = context.mappings.get(&header) {
                        let res = mapping(&mut context).unwrap();
                        stream.write(res.body()).unwrap();
                    } else {
                        let res = error(&mut context).unwrap();
                        stream.write(res.body()).unwrap();
                    }

                    std::task::Poll::Ready(())
                },
                Err(_) => {
                    context.allocator.dealloc(data, 1024);
                    std::task::Poll::Pending
                }
            }
        } else {
            std::task::Poll::Pending
        }
    }
}

impl Context {
    fn new(parent_allocator: &mut alloc::Allocator) -> Result<Context, err::Error> {
        let mut allocator = parent_allocator.child(4 * 4096)?;
        let mut files = collection::Array::new(20, &mut allocator)?;
        let mut mappings = collection::HashMap::new(20, &mut allocator)?;

        files.push(read_file("assets/hello.htmx".into(), &mut allocator)?)?;
        files.push(read_file("assets/error.htmx".into(), &mut allocator)?)?;

        mappings.insert(request::RequestHeader::new(http::Method::Get, b"/hello", http::Version::OneOne, &mut allocator), hello as Mapping)?;
        mappings.insert(request::RequestHeader::new(http::Method::Get, b"/", http::Version::OneOne, &mut allocator), root as Mapping)?;

        Ok(Context {
            mappings,
            allocator,
            files,
        })
    }
}

impl Manager {
    pub fn new(allocator: &mut alloc::Allocator) -> Result<Manager, err::Error> {
        let context = std::sync::Arc::new(std::sync::Mutex::new(Context::new(allocator)?));

        Ok(Manager {
            waiting: collection::Array::new(20, allocator)?,
            queue: collection::Array::new(20, allocator)?,
            context,
        })
    }

    pub fn next(&mut self) -> bool {
        if let Some(mut f) = self.queue.pop() {
            let waker = unsafe { std::task::Waker::from_raw(pointer(&mut self.context.clone())) };
            let mut context = std::task::Context::from_waker(&waker);

            if f.as_mut().poll(&mut context).is_pending() {
                self.waiting.push(f).unwrap();
            }

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

fn pointer<T>(data: &mut T) -> std::task::RawWaker {
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

fn root(context: &mut Context) -> Result<response::HttpResponse, err::Error> {
    Ok(response::HttpResponse::new(http::Version::OneOne, response::HttpStatus::Ok, http::Content::Html(context.files.at(0)?.slice()), &mut context.allocator)?)
}

fn hello(context: &mut Context) -> Result<response::HttpResponse, err::Error> {
    Ok(response::HttpResponse::new(http::Version::OneOne, response::HttpStatus::Ok, http::Content::Html(context.files.at(0)?.slice()), &mut context.allocator)?)
}

fn error(context: &mut Context) -> Result<response::HttpResponse, err::Error> {
    Ok(response::HttpResponse::new(http::Version::OneOne, response::HttpStatus::Error, http::Content::Html(context.files.at(1)?.slice()), &mut context.allocator)?)
}

fn read_file(path: std::path::PathBuf, allocator: &mut alloc::Allocator) -> Result<collection::Array<u8>, err::Error> {
    let mut file = std::fs::File::open(path).map_err(|_| err::Error::FileNotFound)?;
    file.seek(std::io::SeekFrom::End(0)).map_err(|_| err::Error::OutOfBounds)?;
    let size = file.stream_position().map_err(|_| err::Error::OutOfBounds)?;
    file.seek(std::io::SeekFrom::Start(0)).map_err(|_| err::Error::OutOfBounds)?;

    let mut bytes = collection::Array::new(size as usize, allocator)?;
    bytes.zero();

    let slice = bytes.slice_mut();
    let total = file.read(slice).map_err(|_| err::Error::OutOfBounds)?;

    if total != size as usize {
        Err(err::Error::OutOfBounds)
    } else {
        Ok(bytes)
    }
}
