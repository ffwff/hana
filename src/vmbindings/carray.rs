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

    pub fn new() -> CArray<T> {
        use std::mem::size_of;
        CArray::<T> {
            data: unsafe{ libc::malloc(size_of::<T>()*2) as *mut T },
            len: 0,
            capacity: 2
        }
    }

    pub fn reserve(n : usize) -> CArray<T> {
        use std::mem::size_of;
        CArray::<T> {
            data: unsafe{ libc::calloc(n, size_of::<T>()) as *mut T },
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

    pub fn iter(&self) -> ArrayIter<T> {
        ArrayIter {
            array: self,
            idx: 0
        }
    }
}

// index
impl<T> std::ops::Index<usize> for CArray<T> {
    type Output = T;

    fn index(&self, idx : usize) -> &T {
        if idx >= self.len {
            panic!("accessing outside of index!");
        }
        unsafe { &(*self.data.add(idx)) }
    }
}

impl<T> std::ops::IndexMut<usize> for CArray<T> {
    fn index_mut(&mut self, idx : usize) -> &mut T {
        if idx >= self.len {
            panic!("accessing outside of index!");
        }
        unsafe { &mut (*self.data.add(idx)) }
    }
}

// iter
pub struct ArrayIter<'a, T> {
    array: &'a CArray<T>,
    idx: usize,
}

impl<'a, T> std::iter::Iterator for ArrayIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<&'a T> {
        if self.idx >= self.array.len { return None; }
        let ret = Some(&self.array[self.idx]);
        self.idx += 1;
        ret
    }
}