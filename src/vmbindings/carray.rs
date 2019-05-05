use std::ptr::null_mut;

#[repr(C)]
pub struct CArray<T> {
    data: *mut T, // NOTE: I don't free this because hopefully the
                  // VM should automatically call array_free
    _len: usize,
    capacity: usize,
}

impl<T> CArray<T> {
    pub fn new_nil() -> CArray<T> {
        CArray::<T> {
            data: null_mut(),
            _len: 0,
            capacity: 0
        }
    }

    pub fn push(&mut self, val : T) {
        use std::mem::size_of;
        unsafe {
            if self._len == self.capacity {
                self.capacity *= 2;
                self.data = libc::realloc(self.data as *mut libc::c_void,
                    size_of::<T>()*self.capacity) as *mut T;
            }
            std::ptr::write(self.data.add(self._len*size_of::<T>()), val);
            self._len += 1;
        }
    }

    pub fn len(&self) -> usize {
        self._len
    }

    pub fn top(&self) -> &T {
        use std::mem::size_of;
        unsafe {
            &(*self.data.add((self._len-1)*size_of::<T>()))
        }
    }
}

impl<T> std::ops::Index<usize> for CArray<T> {
    type Output = T;

    fn index(&self, idx : usize) -> &T {
        if idx >= self._len {
            panic!("accessing outside of index!");
        }
        use std::mem::size_of;
        unsafe {
            &(*self.data.add((self._len-1)*size_of::<T>()))
        }
    }
}
