//! Provides a stable dynamic array interface that can be used with C

use crate::vmbindings::cnativeval::NativeValue;
use crate::vmbindings::gc::GcTraceable;
use std::alloc::{alloc_zeroed, dealloc, realloc, Layout};
use std::ptr::NonNull;

/// A stable dynamic array interface that can be used with C
#[repr(C)]
pub struct CArray<T> {
    data: NonNull<T>,
    len: usize,
    capacity: usize,
}

impl<T> CArray<T> {
    /// Creates an empty CArray
    pub fn new() -> CArray<T> {
        CArray::<T> {
            data: unsafe {
                let layout = Layout::array::<T>(2).unwrap();
                NonNull::new(alloc_zeroed(layout) as *mut T).unwrap()
            },
            len: 0,
            capacity: 2,
        }
    }

    /// Converts the array into an immutable slice.
    pub fn as_slice(&self) -> &[T] {
        unsafe { std::slice::from_raw_parts(self.as_ptr(), self.len) }
    }
    /// Converts the array into an mutable slice.
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe { std::slice::from_raw_parts_mut(self.as_mut_ptr(), self.len) }
    }

    /// Converts the array into an immutable slice of raw bytes
    pub fn as_bytes(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(
                self.as_ptr() as *const u8,
                std::mem::size_of::<T>() * self.len)
        }
    }
    /// Converts the array into a mutable slice of raw bytes
    pub fn as_mut_bytes(&mut self) -> &mut [u8] {
        unsafe {
            std::slice::from_raw_parts_mut(
                self.as_mut_ptr() as *mut u8,
                std::mem::size_of::<T>() * self.len,
            )
        }
    }

    /// Retrieves the immutable data pointer
    pub fn as_ptr(&self) -> *const T {
        self.data.as_ptr()
    }
    /// Retrieves the mutable data pointer
    pub fn as_mut_ptr(&mut self) -> *mut T {
        self.data.as_ptr()
    }

    /// Retrieves the length of the array
    pub fn len(&self) -> usize {
        self.len
    }

    /// Reserves a zero-initialised array
    ///
    /// This is an internal function, you shouldn't really use this.
    pub fn reserve(n: usize) -> CArray<T> {
        CArray::<T> {
            data: unsafe {
                let layout = Layout::array::<T>(n).unwrap();
                NonNull::new(alloc_zeroed(layout) as *mut T).unwrap()
            },
            len: n,
            capacity: n,
        }
    }

    /// Pushes a value onto the array, increasing its length by 1.
    pub fn push(&mut self, val: T) {
        unsafe {
            use std::mem::size_of;
            if self.len == self.capacity {
                self.capacity *= 2;
                let layout = Layout::array::<T>(self.capacity).unwrap();
                self.data = NonNull::new(realloc(self.as_mut_ptr() as *mut u8, layout, size_of::<T>() * self.capacity)
                    as *mut T).unwrap();
            }
            std::ptr::write(self.as_mut_ptr().add(self.len), val);
        }
        self.len += 1;
    }

    /// Pops a value from the array, decreasing its length by 1.
    pub fn pop(&mut self) {
        if self.len == 0 {
            panic!("popping unbounded!");
        }
        unsafe {
            std::ptr::drop_in_place(self.as_mut_ptr().add(self.len - 1));
        }
        self.len -= 1;
    }

    /// Retrieves an immutable reference to the top value from the stack
    pub fn top(&self) -> &T {
        if self.len == 0 {
            panic!("accessing unbounded!");
        }
        &self[self.len - 1]
    }
    /// Retrieves a mutable reference to the top value from the stack
    pub fn top_mut(&mut self) -> &mut T {
        if self.len == 0 {
            panic!("accessing unbounded!");
        }
        let idx = self.len - 1;
        &mut self[idx]
    }

    /// Gets the iterator for the array
    pub fn iter(&self) -> ArrayIter<T> {
        ArrayIter {
            array: self,
            idx: 0,
        }
    }

    /// Inserts an element to an array at position `pos`
    pub fn insert(&mut self, pos: usize, elem: T) {
        assert!(pos + 1 < self.len);
        unsafe {
            use std::mem::size_of;
            if self.len + 1 >= self.capacity {
                self.capacity *= 2;
                let layout = Layout::array::<T>(self.capacity).unwrap();
                self.data = NonNull::new(realloc(self.as_ptr() as *mut u8, layout, size_of::<T>() * self.capacity)
                    as *mut T).unwrap();
            }
            std::ptr::copy(self.as_ptr().add(pos), self.as_mut_ptr().add(pos + 1), self.len - pos);
            std::ptr::copy(&elem, self.as_mut_ptr().add(pos), 1);
        }
        self.len += 1;
    }

    /// Deletes `nelems` elements starting from position `pos`
    pub fn delete(&mut self, from_pos: usize, nelems: usize) {
        assert!(from_pos + nelems < self.len());
        let remaining = self.len - from_pos - nelems;
        unsafe {
            for i in from_pos..(from_pos + nelems) {
                std::ptr::drop_in_place(self.as_mut_ptr().add(i));
            }
            std::ptr::copy(
                self.as_ptr().add(from_pos + nelems),
                self.as_mut_ptr().add(from_pos),
                remaining,
            );
        }
        self.len -= nelems;
    }
}

impl<T> std::ops::Drop for CArray<T> {
    fn drop(&mut self) {
        unsafe {
            for i in 0..self.len {
                std::ptr::drop_in_place(self.as_mut_ptr().add(i));
            }
            let layout = Layout::array::<T>(self.capacity).unwrap();
            dealloc(self.as_mut_ptr() as *mut u8, layout);
        }
    }
}

impl<T> Clone for CArray<T> {
    fn clone(&self) -> Self {
        CArray::<T> {
            data: unsafe {
                let layout = Layout::array::<T>(self.len).unwrap();
                let d = alloc_zeroed(layout) as *mut T;
                std::ptr::copy(self.as_ptr(), d, self.len);
                NonNull::new(d).unwrap()
            },
            len: self.len,
            capacity: self.len,
        }
    }
}

// index
impl<T> std::ops::Index<usize> for CArray<T> {
    type Output = T;

    fn index(&self, idx: usize) -> &T {
        &self.as_slice()[idx]
    }
}

impl<T> std::ops::IndexMut<usize> for CArray<T> {
    fn index_mut(&mut self, idx: usize) -> &mut T {
        &mut self.as_mut_slice()[idx]
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
        if self.idx >= self.array.len {
            return None;
        }
        let ret = Some(&self.array[self.idx]);
        self.idx += 1;
        ret
    }
}

// gc traceable
impl GcTraceable for CArray<NativeValue> {
    fn trace(ptr: *mut libc::c_void) {
        unsafe {
            let self_ = &*(ptr as *mut Self);
            for val in self_.iter() {
                val.trace();
            }
        }
    }
}

// display
use std::fmt::Debug;
impl<T: Debug> Debug for CArray<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "[")?;
        for val in self.iter() {
            val.fmt(f)?;
            write!(f, ", ")?;
        }
        write!(f, "]")
    }
}
