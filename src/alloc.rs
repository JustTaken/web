use crate::err;

pub struct Allocator {
    bytes: *mut u8,
    capacity: usize,
    end: usize,
}

impl Allocator {
    pub fn new(capacity: usize) -> Allocator {
        let bytes = unsafe {std::alloc::alloc(std::alloc::Layout::from_size_align_unchecked(capacity, 1)) };

        Allocator {
            bytes,
            capacity,
            end: 0,
        }
    }

    pub fn child(&mut self, capacity: usize) -> Result<Allocator, err::Error> {
        Ok(Allocator {
            bytes: self.alloc(capacity)?,
            capacity,
            end: 0,
        })
    }

    pub fn alloc<T>(&mut self, count: usize) -> Result<*mut T, err::Error> {
        let layout = std::alloc::Layout::from_size_align(count * std::mem::size_of::<T>(), std::mem::align_of::<T>()).map_err(|_| err::Error::Allocation)?;
        let size = layout.size();
        let align = layout.align();

        if self.end + size > self.capacity {
            return Err(err::Error::Allocation);
        }

        let ptr = unsafe { self.bytes.add(self.end) };
        let offset = ptr.align_offset(align);
        self.end += offset + size;

        Ok(unsafe {ptr.add(offset) as *mut T})
    }

    pub fn dealloc<T>(&mut self, ptr: *mut T, count: usize) {
        let layout = unsafe { std::alloc::Layout::from_size_align_unchecked(count * std::mem::size_of::<T>(), std::mem::align_of::<T>()) };
        let size = layout.size();
        let bytes = ptr as *mut u8;

        if unsafe {self.bytes.add(self.end) == bytes.add(size) } {
            self.end -= size;
        }
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn bytes(&self) -> *mut u8 {
        self.bytes
    }

    pub fn free_size(&self) -> usize {
        self.capacity - self.end
    }

    pub fn clear(&mut self) {
        self.end = 0;
    }
}

