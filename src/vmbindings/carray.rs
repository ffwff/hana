use std::ptr::null_mut;

#[repr(C)]
pub struct CArray<T> {
    data: *mut T,
    // NOTE: this won't be freed using drop because
    // the owner SHOULD automatically call array_free

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

    // constructor/destructor
    pub fn new() -> CArray<T> {
        use std::mem::size_of;
        CArray::<T> {
            data: unsafe{ libc::malloc(size_of::<T>()*2) as *mut T },
            len: 0,
            capacity: 2
        }
    }

    pub fn new_static(arr: &mut [T]) -> CArray<T> {
        // convenient wrapper for statically allocated
        // (stack-allocated) arrays, use this for storing function
        // arguments that will be passed to Vm::call
        CArray::<T> {
            data: &mut arr[0] as *mut T,
            len: arr.len(),
            capacity: arr.len()
        }
    }

    pub fn drop(&mut self) { // must be called manually
        // this function MUST BE called by its owner
        // for example from array_obj::drop function
        unsafe{ libc::free(self.data as *mut libc::c_void) };
        self.data = null_mut();
        self.len = 0;
        self.capacity = 0;
    }

    // to slice
    pub fn as_slice(&self) -> &[T] {
        unsafe{ std::slice::from_raw_parts(self.data, self.len) }
    }
    pub fn as_slice_mut(&self) -> &mut [T] {
        unsafe { std::slice::from_raw_parts_mut(self.data, self.len) }
    }

    // clone
    pub fn clone(&self) -> CArray<T> {
        use std::mem::size_of;
        CArray::<T> {
            data: unsafe {
                let d = libc::malloc(size_of::<T>()*self.len) as *mut T;
                std::ptr::copy(self.data, d, self.len);
                d
            },
            len: self.len,
            capacity: self.len
        }

    }

    // length
    pub fn len(&self) -> usize {
        self.len
    }

    pub fn reserve(n : usize) -> CArray<T> {
        use std::mem::size_of;
        CArray::<T> {
            data: unsafe{ libc::calloc(n, size_of::<T>()) as *mut T },
            len: n,
            capacity: n
        }
    }

    // stack
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

    pub fn pop(&mut self) {
        if self.len == 0 { panic!("popping unbounded!"); }
        self.len -= 1;
    }

    pub fn top(&self) -> &T {
        if self.len == 0 { panic!("accessing unbounded!"); }
        unsafe { &(*self.data.add(self.len-1)) }
    }

    // iterator
    pub fn iter(&self) -> ArrayIter<T> {
        ArrayIter {
            array: self,
            idx: 0
        }
    }

    // other
    pub fn delete(&mut self, from_pos : usize, nelems : usize) {
        assert!(from_pos + nelems < self.len());
        let remaining = self.len - from_pos - nelems;
        unsafe {
            std::ptr::copy(self.data.add(from_pos+nelems),
                self.data.add(from_pos), remaining);
        }
        self.len = remaining;
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