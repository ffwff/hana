use std::ptr::{null_mut, drop_in_place};
use std::alloc::{alloc_zeroed, dealloc, Layout};
pub use libc::c_void;
use super::vm::Vm;

// node
struct GcNode {
    prev: *mut GcNode,
    next: *mut GcNode,
    size: usize,
    unreachable: bool, // by default this is false
    // if the node is unreachable, it will be pruned (free'd)
    finalizer: GenericFinalizer
}

impl GcNode {

    pub fn alloc_size<T: Sized>() -> usize {
        // number of bytes needed to allocate node for <T>
        use std::mem::size_of;
        size_of::<GcNode>() + size_of::<T>()
    }

}

// finalizer
// gets called with a pointer (represented as *void) to
// the data that's about to be freed
type GenericFinalizer = fn(*mut c_void);

// manager
const INITIAL_THRESHOLD: usize = 100;
const USED_SPACE_RATIO: f64 = 0.7;
struct GcManager {
    first_node: *mut GcNode,
    last_node: *mut GcNode,
    root: *mut Vm,
    bytes_allocated: usize,
    threshold: usize,
    enabled: bool
}

impl GcManager {

    pub fn new() -> GcManager {
        GcManager {
            first_node: null_mut(),
            last_node: null_mut(),
            root: null_mut(),
            bytes_allocated: 0,
            threshold: INITIAL_THRESHOLD,
            enabled: true
        }
    }

    pub unsafe fn malloc<T: Sized>
        (&mut self, x: T, finalizer: GenericFinalizer) -> *mut T {
        // free up if over threshold
        /* if self.bytes_allocated > self.threshold {
            self.collect();
            // we didn't collect enough, grow the ratio
            if ((self.bytes_allocated as f64) / (self.threshold as f64)) > USED_SPACE_RATIO {
                self.threshold = (self.bytes_allocated as f64 / USED_SPACE_RATIO) as usize;
            }
        } */
        // tfw no qt malloc function
        let layout = Layout::from_size_align(GcNode::alloc_size::<T>(), 2).unwrap();
        let bytes : *mut GcNode = alloc_zeroed(layout) as *mut GcNode;
        // append node
        if self.first_node.is_null() {
            self.first_node = bytes;
            self.last_node = bytes;
            (*bytes).prev = null_mut();
            (*bytes).next = null_mut();
        } else {
            (*self.last_node).next = bytes;
            (*bytes).prev = self.last_node;
            (*bytes).next = null_mut();
            self.last_node = bytes;
        }
        (*bytes).finalizer = finalizer;
        (*bytes).size = GcNode::alloc_size::<T>();
        self.bytes_allocated += (*bytes).size;
        // return the body aka (start byte + sizeof(GCNode))
        std::mem::replace(&mut *(bytes.add(1) as *mut T), x);
        bytes.add(1) as *mut T
    }

    pub unsafe fn free<T: Sized>(&mut self, x: *mut T) {
        // => start byte
        let node : *mut GcNode = (x as *mut GcNode).sub(1);

        if (*node).prev.is_null() { self.first_node = (*node).next; }
        else { (*(*node).prev).next = (*node).next; }

        if (*node).next.is_null() { self.last_node = (*node).prev; }
        else { (*(*node).next).prev = (*node).prev; }

        self.bytes_allocated -= (*node).size;

        // call finalizer
        let finalizer = (*node).finalizer;
        finalizer(x as *mut c_void);
        // free memory
        let layout = Layout::from_size_align((*node).size, 2).unwrap();
        dealloc(node as *mut u8, layout);
    }

    // roots
    pub fn set_root(&mut self, root: *mut Vm) {
        self.root = root;
    }

    // state
    pub fn enable(&mut self) { self.enabled = true; }
    pub fn disable(&mut self) { self.enabled = false; }

    // gc algorithm
    pub fn collect(&mut self) {
        if !self.enabled { return; }

        // mark phase:
        unsafe { // reset all nodes
            let mut node : *mut GcNode = self.first_node;
            while !node.is_null() {
                let next : *mut GcNode = (*node).next;
                (*node).unreachable = true;
                node = next;
            }
        }
        let vm = unsafe { &mut *self.root };
        vm.mark();
        // sweep phase:
        unsafe {
            let mut node : *mut GcNode = self.first_node;
            while !node.is_null() {
                let next : *mut GcNode = (*node).next;
                if (*node).unreachable {
                    let body = node.add(1);

                    // remove from ll
                    if (*node).prev.is_null() { self.first_node = (*node).next; }
                    else { (*(*node).prev).next = (*node).next; }

                    if (*node).next.is_null() { self.last_node = (*node).prev; }
                    else { (*(*node).next).prev = (*node).prev; }

                    self.bytes_allocated -= (*node).size;

                    // call finalizer
                    let finalizer = (*node).finalizer;
                    finalizer(body as *mut c_void);

                    // free memory
                    let layout = Layout::from_size_align((*node).size, 2).unwrap();
                    dealloc(node as *mut u8, layout);
                }
                node = next;
            }
        }
    }

    // ## marking
    pub unsafe fn mark_reachable(ptr: *mut c_void) -> bool {
        // => start byte
        if ptr.is_null() { return false; }
        let node : *mut GcNode = (ptr as *mut GcNode).sub(1);
        if !(*node).unreachable { return false; }
        (*node).unreachable = false;
        true
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

// static allocator
use std::cell::RefCell;
thread_local! {
    static GC_MANAGER : RefCell<GcManager> =
        RefCell::new(GcManager::new());
}

// general
pub unsafe fn malloc<T: Sized>(x: T, finalizer: GenericFinalizer) -> *mut T {
    let mut alloced: *mut T = null_mut();
    GC_MANAGER.with(|gc_manager| {
        let mut gc_manager = gc_manager.borrow_mut();
        alloced = gc_manager.malloc(x, finalizer);
    });
    alloced
}
pub unsafe fn free<T: Sized>(x: *mut T) {
    GC_MANAGER.with(|gc_manager| {
        let mut gc_manager = gc_manager.borrow_mut();
        gc_manager.free(x);
    });
}

// roots
pub fn set_root(vm: *mut Vm) {
    GC_MANAGER.with(|gc_manager| {
        let mut gc_manager = gc_manager.borrow_mut();
        gc_manager.set_root(vm);
    });
}

// state
pub fn enable() {
    GC_MANAGER.with(|gc_manager| {
        let mut gc_manager = gc_manager.borrow_mut();
        gc_manager.enable();
    });
}
pub fn disable() {
    GC_MANAGER.with(|gc_manager| {
        let mut gc_manager = gc_manager.borrow_mut();
        gc_manager.disable();
    });
}

// collect
pub fn collect() {
    GC_MANAGER.with(|gc_manager| {
        let mut gc_manager = gc_manager.borrow_mut();
        gc_manager.collect();
    });
}
pub unsafe fn mark_reachable(ptr: *mut c_void) -> bool {
    GcManager::mark_reachable(ptr)
}

// helpers
pub fn no_finalizer(_ : *mut c_void) {}
pub unsafe fn drop<T>(ptr: *mut c_void) {
    drop_in_place::<T>(ptr as *mut T);
}