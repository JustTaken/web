use crate::{err, alloc};

pub struct Array<T> {
    ptr: *mut T,
    capacity: usize,
    len: usize,
}

pub struct HashMap<K, V> {
    keys: Array<K>,
    values: Array<V>,
}

impl<T> Array<T> {
    pub fn new(capacity: usize, allocator: &mut alloc::Allocator) -> Result<Array<T>, err::Error> {
        Ok(Array::<T> {
            ptr: allocator.alloc(capacity)?,
            capacity,
            len: 0,
        })
    }

    pub fn copy(&mut self, dst: &[T]) -> Result<(), err::Error> where T: Copy {
        if self.len + dst.len() > self.capacity {
            Err(err::Error::OutOfBounds)
        } else {
            let src = unsafe { std::slice::from_raw_parts_mut(self.ptr.add(self.len), dst.len())};
            src.copy_from_slice(dst);

            self.len += dst.len();

            Ok(())
        }
    }

    pub fn push(&mut self, item: T) -> Result<(), err::Error> {
        if self.len >= self.capacity {
            Err(err::Error::OutOfBounds)
        } else {
            unsafe { self.ptr.add(self.len).write(item) };
            self.len += 1;

            Ok(())
        }
    }

    pub fn append_slice(&mut self, items: &[T]) -> Result<(), err::Error> where T: Copy {
        if self.len + items.len() > self.capacity {
            Err(err::Error::OutOfBounds)
        } else {
            for i in self.len..items.len() + self.len {
                unsafe { self.ptr.add(i).write(items[i - self.len]) };
            }

            self.len += items.len();
            Ok(())
        }
    }

    pub fn insert(&mut self, index: usize, item: T) -> Result<(), err::Error> {
        if index >= self.capacity {
            Err(err::Error::OutOfBounds)
        } else {
            Ok(unsafe { self.ptr.add(index).write(item) })
        }
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            None
        } else {
            self.len -= 1;
            Some(unsafe { self.ptr.add(self.len).read() })
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn cap(&self) -> usize {
        self.capacity
    }

    pub fn zero(&mut self) {
        unsafe { self.ptr.write_bytes(0, self.capacity) };
        self.len = self.capacity;
    }

    pub fn at(&self, index: usize) -> Result<T, err::Error> {
        if index >= self.capacity {
            Err(err::Error::OutOfBounds)
        } else {
            Ok(unsafe { self.ptr.add(index).read() })
        }
    }

    pub fn eq(&self, other: &Array<T>) -> bool where T: Eq {
        let self_slice = self.slice();
        let other_slice = other.slice();
        let length = self_slice.len();

        if length != other_slice.len() {
            false
        } else {
            for i in 0..length {
                if self_slice[i] != other_slice[i] {
                    return false;
                }
            }

            true
        }
    }

    pub fn slice(&self) -> &[T] {
        unsafe { std::slice::from_raw_parts(self.ptr, self.len) }
    }

    pub fn is_null(&self) -> bool {
        self.ptr.is_null()
    }

}

impl Array<u8> {
    pub fn parse(&mut self, value: usize) -> Result<(), err::Error> {
        let mut n = value;

        while n > 0 {
            let rest = n % 10;
            n /= 10;
            self.push(rest as u8)?;
        }

        Ok(())
    }
}

pub trait Hash {
    fn hash(&self) -> usize;
    fn eq(&self, other: &Self) -> bool;
    fn is_zero(&self) -> bool;
}

impl<K: Hash, V> HashMap<K, V> {
    pub fn new(capacity: usize, allocator: &mut alloc::Allocator) -> Result<HashMap<K, V>, err::Error> {
        let mut keys = Array::new(capacity, allocator)?;
        let mut values = Array::new(capacity, allocator)?;

        keys.zero();
        values.zero();

        Ok(HashMap::<K, V> {
            keys,
            values,
        })
    }

    pub fn insert(&mut self, key: K, value: V) -> Result<(), err::Error> {
        let len = self.keys.len();
        let hash = key.hash();
        let pos = hash % len;

        let mut index = pos;

        for i in 0..len {
            index = (i + pos) % len;

            if self.keys.at(index)?.is_zero() {
                break;
            }
        }

        if (index + 1) % len == pos {
            Err(err::Error::OutOfBounds)
        } else {
            self.keys.insert(index, key)?;
            self.values.insert(index, value)?;

            Ok(())
        }
    }

    pub fn get(&mut self, key: &K) -> Option<V> {
        let len = self.keys.len();

        let hash = key.hash();
        let pos = hash % len;

        for i in 0..len {
            let index = (i + pos) % len;
            let at = self.keys.at(index).unwrap();

            if at.is_zero() {
                return None;
            } if at.eq(&key) {
                return Some(self.values.at(index).unwrap());
            }
        }

        None
    }
}
