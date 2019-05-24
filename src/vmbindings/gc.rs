use std::ptr::{null_mut, drop_in_place};
use std::alloc::{alloc_zeroed, dealloc, Layout};
use std::rc::Weak;
use std::cell::RefCell;
pub use libc::c_void;
use super::vm::Vm;

// node
struct GcNode {
    next: *mut GcNode,
    size: usize,
    unreachable: bool, // by default this is false
    // if the node is unreachable, it will be pruned (free'd)
    pub native_refs: usize,
    tracer: GenericFunction,
    // tracer gets called sweep phased (FIXME)
    finalizer: GenericFunction,
    // finalizer gets called with a pointer to
    // the data that's about to be freed
}

impl GcNode {

    pub fn alloc_size<T: Sized>() -> usize {
        // number of bytes needed to allocate node for <T>
        use std::mem::size_of;
        size_of::<GcNode>() + size_of::<T>()
    }

}

type GenericFunction = fn(*mut c_void);
// a generic function that takes in some pointer
// this might be a finalizer or a tracer function
// TODO maybe replace this with Any

// manager
const INITIAL_THRESHOLD: usize = 100;
const USED_SPACE_RATIO: f64 = 0.7;
pub struct GcManager {
    first_node: *mut GcNode,
    last_node: *mut GcNode,
    root: Weak<RefCell<Vm>>,
    bytes_allocated: usize,
    threshold: usize,
    enabled: bool
}

impl GcManager {

    pub fn new(root: Weak<RefCell<Vm>>) -> GcManager {
        GcManager {
            first_node: null_mut(),
            last_node: null_mut(),
            root: root,
            bytes_allocated: 0,
            threshold: INITIAL_THRESHOLD,
            enabled: false
        }
    }

    unsafe fn malloc_raw<T: Sized + GcTraceable>
        (&mut self, x: T, finalizer: GenericFunction) -> *mut T {
        // free up if over threshold
        if cfg!(test) {
            self.collect();
        } else if self.bytes_allocated > self.threshold {
            self.collect();
            // we didn't collect enough, grow the ratio
            if ((self.bytes_allocated as f64) / (self.threshold as f64)) > USED_SPACE_RATIO {
                self.threshold = (self.bytes_allocated as f64 / USED_SPACE_RATIO) as usize;
            }
        }
        // tfw no qt malloc function
        let layout = Layout::from_size_align(GcNode::alloc_size::<T>(), 2).unwrap();
        let bytes : *mut GcNode = alloc_zeroed(layout) as *mut GcNode;
        // append node
        if self.first_node.is_null() {
            self.first_node = bytes;
            self.last_node = bytes;
            (*bytes).next = null_mut();
        } else {
            (*self.last_node).next = bytes;
            (*bytes).next = null_mut();
            self.last_node = bytes;
        }
        (*bytes).native_refs = 1;
        (*bytes).tracer = T::trace;
        (*bytes).finalizer = finalizer;
        (*bytes).size = GcNode::alloc_size::<T>();
        self.bytes_allocated += (*bytes).size;
        // return the body aka (start byte + sizeof(GCNode))
        std::mem::replace(&mut *(bytes.add(1) as *mut T), x);
        bytes.add(1) as *mut T
    }

    pub fn malloc<T: Sized + GcTraceable>(&mut self, val: T) -> Gc<T> {
        Gc {
            ptr: unsafe {
                self.malloc_raw(val, |ptr| drop_in_place::<T>(ptr as *mut T))
            }
        }
    }

    // state
    pub fn enable(&mut self) { self.enabled = true; }
    pub fn disable(&mut self) { self.enabled = false; }

    // gc algorithm
    unsafe fn collect(&mut self) {
        if !self.enabled { return; }
        // mark phase:
        let mut node : *mut GcNode = self.first_node;
        // reset all nodes
        while !node.is_null() {
            let next : *mut GcNode = (*node).next;
            (*node).unreachable = true;
            node = next;
        }
        // mark make nodes with at least one native reference
        node = self.first_node;
        while !node.is_null() {
            let next : *mut GcNode = (*node).next;
            if (*node).native_refs > 0 {
                (*node).unreachable = false;
                ((*node).tracer)(node.add(1) as *mut c_void);
            }
            node = next;
        }
        // mark from root
        {
            let rootcell = self.root.upgrade().unwrap();
            rootcell.borrow().mark();
        }
        // sweep phase:
        let mut node : *mut GcNode = self.first_node;
        let mut prev : *mut GcNode = null_mut();
        while !node.is_null() {
            let next : *mut GcNode = (*node).next;
            let mut freed = false;
            if (*node).native_refs == 0 && (*node).unreachable {
                freed = true;
                let body = node.add(1);

                // remove from ll
                if prev.is_null() { self.first_node = (*node).next; }
                else { (*prev).next = (*node).next; }

                if (*node).next.is_null() { self.last_node = prev; }

                self.bytes_allocated -= (*node).size;

                // call finalizer
                let finalizer = (*node).finalizer;
                finalizer(body as *mut c_void);

                // free memory
                let layout = Layout::from_size_align((*node).size, 2).unwrap();
                dealloc(node as *mut u8, layout);
            }
            if !freed { prev = node; }
            node = next;
        }
    }

}

unsafe impl std::marker::Send for GcManager {}
unsafe impl std::marker::Sync for GcManager {}

impl std::ops::Drop for GcManager {

    fn drop(&mut self) {
        unsafe {
            let mut node : *mut GcNode = self.first_node;
            while !node.is_null() {
                let next : *mut GcNode = (*node).next;
                let body = node.add(1);
                // call finalizer
                let finalizer = (*node).finalizer;
                finalizer(body as *mut c_void);
                // free memory
                let layout = Layout::from_size_align((*node).size, 2).unwrap();
                dealloc(node as *mut u8, layout);
                node = next;
            }
        }
    }

}

// gc struct
#[repr(transparent)]
pub struct Gc<T: Sized + GcTraceable> {
    ptr: *mut T,
}

impl<T: Sized + GcTraceable> Gc<T> {
    pub fn new_nil() -> Gc<T> {
        Gc {
            ptr: null_mut()
        }
    }

    // raw
    pub fn from_raw(ptr: *mut T) -> Gc<T> {
        unsafe{ ref_inc(ptr as *mut libc::c_void); }
        Gc {
            ptr: ptr
        }
    }
    pub fn into_raw(self) -> *mut T {
        self.ptr
    }

    // ptrs
    pub fn to_raw(&self) -> *const T {
        self.ptr
    }
    pub fn to_mut_raw(&mut self) -> *mut T {
        self.ptr
    }
    pub fn ptr_eq(&self, right: &Gc<T>) -> bool {
        std::ptr::eq(self.ptr, right.ptr)
    }

    // refs with interior mutability
    pub fn as_mut(&self) -> &mut T {
        unsafe{ &mut *self.ptr }
    }
}

impl<T: Sized + GcTraceable> std::ops::Drop for Gc<T> {
    fn drop(&mut self) {
        unsafe {
            if !self.ptr.is_null() {
                ref_dec(self.ptr as *mut libc::c_void);
            }
        }
    }
}

impl<T: Sized + GcTraceable> std::convert::AsRef<T> for Gc<T> {
    fn as_ref(&self) -> &T {
        unsafe{ &*self.ptr }
    }
}

impl<T: Sized + GcTraceable> std::clone::Clone for Gc<T> {
    fn clone(&self) -> Self {
        Gc {
            ptr: unsafe {
                ref_inc(self.ptr as *mut libc::c_void);
                self.ptr
            }
        }
    }
}

pub trait GcTraceable {
    fn trace(ptr: *mut libc::c_void);
}

// native traceables
impl GcTraceable for String {
    fn trace(_: *mut libc::c_void) {}
}

// collect
pub unsafe fn ref_inc(ptr: *mut c_void) {
    if ptr.is_null() { return; }
    let node : *mut GcNode = (ptr as *mut GcNode).sub(1);
    (*node).native_refs += 1;
}

pub unsafe fn ref_dec(ptr: *mut c_void) {
    if ptr.is_null() { return; }
    let node : *mut GcNode = (ptr as *mut GcNode).sub(1);
    (*node).native_refs -= 1;
}

pub unsafe fn mark_reachable(ptr: *mut c_void) -> bool {
    // => start byte
    if ptr.is_null() { return false; }
    let node : *mut GcNode = (ptr as *mut GcNode).sub(1);
    if !(*node).unreachable { return false; }
    (*node).unreachable = false;
    true
}