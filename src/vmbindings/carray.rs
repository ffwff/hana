use std::ptr::null_mut;

#[repr(C)]
pub struct CArray<T> {
    data: *mut T, // NOTE: this won't be freed inside rust because
                  // the VM should automatically call array_free
    len: usize,
    capacity: usize,
}

impl<T> CArray<T> {
    pub fn new_nil() -> CArray<T> {
        CArray::<T> {
            data: null_mut(),
            len: 0,
            capacity: 0
        }
    }

    pub fn new(n : usize) -> CArray<T> {
        use std::mem::size_of;
        CArray::<T> {
            data: unsafe{ libc::malloc(size_of::<T>()*n) as *mut T },
            len: n,
            capacity: n
        }
    }

    pub fn push(&mut self, val : T) {
        use std::mem::size_of;
        unsafe {
            if self.len == self.capacity {
                self.capacity *= 2;
                self.data = libc::realloc(self.data as *mut libc::c_void,
                    size_of::<T>()*self.capacity) as *mut T;
            }
            std::ptr::write(self.data.add(self.len*size_of::<T>()), val);
            self.len += 1;
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn top(&self) -> &T {
        if self.len == 0 { panic!("accessing unbounded!"); }
        unsafe { &(*self.data.add(self.len-1)) }
    }
    pub fn pop(&mut self) {
        if self.len == 0 { panic!("popping unbounded!"); }
        self.len -= 1;
    }
}

impl<T> std::ops::Index<usize> for CArray<T> {
    type Output = T;

    fn index(&self, idx : usize) -> &T {
        if idx >= self.len {
            panic!("accessing outside of index!");
        }
        unsafe { &(*self.data.add(idx)) }
    }
}