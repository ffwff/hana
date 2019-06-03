//! Basic implementation of a mark and sweep garbage collector

pub use libc::c_void;
use std::alloc::{alloc_zeroed, dealloc, Layout};
use std::ptr::{drop_in_place, null_mut, NonNull};

use super::vm::Vm;

#[derive(Debug, PartialEq)]
enum GcNodeColor {
    White,
    Gray,
    Black,
}

// node
pub struct GcNode {
    next: *mut GcNode,
    size: usize,
    color: GcNodeColor,
    native_refs: usize,
    // tracer gets called on the marking phase
    tracer: GenericTraceFunction,
    /* finalizer gets called with a pointer to
     * the data that's about to be freed */
    finalizer: GenericFunction,
}

impl GcNode {
    pub fn alloc_size<T: Sized>() -> usize {
        // number of bytes needed to allocate node for <T>
        use std::mem::size_of;
        size_of::<GcNode>() + size_of::<T>()
    }
}

type GenericFunction = unsafe fn(*mut c_void);
// a generic function that takes in some pointer
// this might be a finalizer or a tracer function
// TODO maybe replace this with Any

// manager
const INITIAL_THRESHOLD: usize = 128;
const USED_SPACE_RATIO: f64 = 0.8;
pub struct GcManager {
    first_node: *mut GcNode,
    last_node: *mut GcNode,
    bytes_allocated: usize,
    gray_nodes: Vec<*mut GcNode>,
    threshold: usize,
    enabled: bool,
}

impl GcManager {
    pub fn new() -> GcManager {
        GcManager {
            first_node: null_mut(),
            last_node: null_mut(),
            bytes_allocated: 0,
            gray_nodes: Vec::new(),
            threshold: INITIAL_THRESHOLD,
            enabled: false,
        }
    }

    unsafe fn malloc_raw<T: Sized + GcTraceable>(
        &mut self, vm: &Vm, x: T, finalizer: GenericFunction,
    ) -> *mut T {
        self.cycle(vm);
        // tfw no qt malloc function
        let layout = Layout::from_size_align(GcNode::alloc_size::<T>(), 2).unwrap();
        let node: *mut GcNode = alloc_zeroed(layout) as *mut GcNode;
        // append node
        if self.first_node.is_null() {
            self.first_node = node;
            self.last_node = node;
            (*node).next = null_mut();
        } else {
            (*self.last_node).next = node;
            (*node).next = null_mut();
            self.last_node = node;
        }
        (*node).native_refs = 1;
        (*node).tracer = std::mem::transmute(T::trace as *mut c_void);
        (*node).finalizer = finalizer;
        (*node).size = GcNode::alloc_size::<T>();
        self.bytes_allocated += (*node).size;
        // gray out the node
        // TODO: we currently move the write barrier forward rather than backwards
        // this probably is less efficient than setting the newly allocated node
        // to white then resetting its soon-to-be parent to gray (for retracing)
        (*node).color = GcNodeColor::Gray;
        self.gray_nodes.push(node);
        // return the body aka (start byte + sizeof(GCNode))
        std::mem::replace(&mut *(node.add(1) as *mut T), x);
        node.add(1) as *mut T
    }

    pub fn malloc<T: Sized + GcTraceable>(&mut self, vm: &Vm, val: T) -> Gc<T> {
        Gc {
            ptr: NonNull::new(unsafe {
                self.malloc_raw(vm, val, |ptr| drop_in_place::<T>(ptr as *mut T))
            }).unwrap(),
        }
    }

    // state
    pub fn enable(&mut self) {
        self.enabled = true;
    }
    pub fn disable(&mut self) {
        self.enabled = false;
    }

    // gc algorithm
    unsafe fn cycle(&mut self, vm: &Vm) {
        if !self.enabled {
            return;
        }
        //eprintln!("GRAY NODES: {:?}", self.gray_nodes);
        // marking phase
        let gray_nodes = std::mem::replace(&mut self.gray_nodes, Vec::new());
        for node in gray_nodes.iter() {
            let body = node.add(1) as *mut c_void;
            (**node).color = GcNodeColor::Black;
            ((**node).tracer)(body, std::mem::transmute(&mut self.gray_nodes));
        }
        //eprintln!("GRAY NODES MARKING: {:?}, {:?}", gray_nodes, self.gray_nodes);
        // nothing left to traverse, sweeping phase:
        if self.gray_nodes.is_empty() && self.bytes_allocated > self.threshold {
            let mut node: *mut GcNode = self.first_node;
            let mut prev: *mut GcNode = null_mut();
            // don't sweep nodes with at least one native reference
            node = self.first_node;
            while !node.is_null() {
                let next: *mut GcNode = (*node).next;
                let ptr = node.add(1) as *mut c_void;
                if (*node).native_refs > 0 && (*node).color != GcNodeColor::Black {
                    //eprintln!("{:p}", node);
                    // get its children and subchildren
                    let mut children : Vec<*mut GcNode> = Vec::new();
                    ((*node).tracer)(std::mem::transmute(ptr), std::mem::transmute(&mut children));
                    let mut i = 0;
                    while i < children.len() {
                        let node = children[i];
                        ((*node).tracer)(std::mem::transmute(ptr), std::mem::transmute(&mut children));
                        i += 1;
                    }
                    // color them black
                    //eprintln!("children: {:?}", children);
                    for child in children {
                        (*child).color = GcNodeColor::Black;
                    }
                }
                node = next;
            }
            // sweep
            node = self.first_node;
            while !node.is_null() {
                let next: *mut GcNode = (*node).next;
                let mut freed = false;
                if (*node).native_refs == 0 && (*node).color == GcNodeColor::White {
                    freed = true;
                    //eprintln!("free {:p}", node);
                    let body = node.add(1);

                    // remove from ll
                    if prev.is_null() {
                        self.first_node = (*node).next;
                    } else {
                        (*prev).next = (*node).next;
                    }
                    if (*node).next.is_null() {
                        self.last_node = prev;
                    }
                    self.bytes_allocated -= (*node).size;

                    // call finalizer
                    let finalizer = (*node).finalizer;
                    finalizer(body as *mut c_void);

                    // free memory
                    let layout = Layout::from_size_align((*node).size, 2).unwrap();
                    dealloc(node as *mut u8, layout);
                } else {
                    (*node).color = GcNodeColor::White;
                }
                if !freed {
                    prev = node;
                }
                node = next;
            }
            // reset all root nodes to gray
            self.gray_nodes = vm.gc_new_gray_node_stack();

            // we didn't collect enough, grow the ratio
            if ((self.bytes_allocated as f64) / (self.threshold as f64)) > USED_SPACE_RATIO {
                self.threshold = (self.bytes_allocated as f64 / USED_SPACE_RATIO) as usize;
            }
        }
    }
}

unsafe impl std::marker::Send for GcManager {}
unsafe impl std::marker::Sync for GcManager {}

impl std::ops::Drop for GcManager {
    fn drop(&mut self) {
        unsafe {
            let mut node: *mut GcNode = self.first_node;
            while !node.is_null() {
                let next: *mut GcNode = (*node).next;
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

// #region gc struct
#[repr(transparent)]
pub struct Gc<T: Sized + GcTraceable> {
    ptr: NonNull<T>,
}

impl<T: Sized + GcTraceable> Gc<T> {
    // raw
    pub unsafe fn from_raw(ptr: *mut T) -> Gc<T> {
        //println!("from raw");
        // manually color it black & increment ref
        let node: *mut GcNode = (ptr as *mut GcNode).sub(1);
        (*node).color = GcNodeColor::Black;
        (*node).native_refs += 1;
        Gc {
            ptr: NonNull::new(ptr).unwrap(),
        }
    }

    // ptrs
    pub fn to_raw(&self) -> *const T {
        self.ptr.as_ptr()
    }
    pub unsafe fn into_raw(self) -> *mut T {
        self.ptr.as_ptr()
    }

    // refs with interior mutability
    pub fn as_mut(&self) -> &mut T {
        unsafe { &mut *self.ptr.as_ptr() }
    }
}

impl<T: Sized + GcTraceable> std::ops::Drop for Gc<T> {
    fn drop(&mut self) {
        unsafe {
            ref_dec(self.ptr.as_ptr() as *mut libc::c_void);
        }
    }
}

impl<T: Sized + GcTraceable> std::convert::AsRef<T> for Gc<T> {
    fn as_ref(&self) -> &T {
        unsafe { self.ptr.as_ref() }
    }
}

impl<T: Sized + GcTraceable> std::clone::Clone for Gc<T> {
    fn clone(&self) -> Self {
        Gc {
            ptr: unsafe {
                ref_inc(self.ptr.as_ptr() as *mut libc::c_void);
                self.ptr
            },
        }
    }
}

impl<T: Sized + GcTraceable> std::cmp::PartialEq for Gc<T> {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.ptr.as_ptr(), other.ptr.as_ptr())
    }
}
// #endregion

// #region traceable
pub trait GcTraceable {
    unsafe fn trace(&self, manager: &mut Vec<*mut GcNode>);
}

type GenericTraceFunction = unsafe fn(*mut c_void, *mut c_void);

// native traceables
impl GcTraceable for String {
    unsafe fn trace(&self, manager: &mut Vec<*mut GcNode>) {}
}

use super::cnativeval::NativeValue;
impl GcTraceable for Vec<NativeValue> {
    unsafe fn trace(&self, gray_nodes: &mut Vec<*mut GcNode>) {
        for val in self.iter() {
            if let Some(ptr) = val.as_gc_pointer() {
                push_gray_body(gray_nodes, ptr);
            }
        }
    }
}
// #endregion

// collect
pub unsafe fn ref_inc(ptr: *mut c_void) {
    if ptr.is_null() {
        return;
    }
    let node: *mut GcNode = (ptr as *mut GcNode).sub(1);
    (*node).native_refs += 1;
}

pub unsafe fn ref_dec(ptr: *mut c_void) {
    if ptr.is_null() {
        return;
    }
    let node: *mut GcNode = (ptr as *mut GcNode).sub(1);
    (*node).native_refs -= 1;
}

pub unsafe fn push_gray_body(gray_nodes: &mut Vec<*mut GcNode>, ptr: *mut c_void) {
    let node: *mut GcNode = (ptr as *mut GcNode).sub(1);
    if (*node).color == GcNodeColor::Black || (*node).color == GcNodeColor::Gray {
        return;
    }
    (*node).color = GcNodeColor::Gray;
    gray_nodes.push(node);
}