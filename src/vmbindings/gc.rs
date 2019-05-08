use std::ptr::{null_mut, drop_in_place};
use std::alloc::{alloc_zeroed, dealloc, Layout};
use super::vm::Vm;
use super::lazy_static;

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
        return size_of::<GcNode>() + size_of::<T>();
    }

}

// finalizer
// gets called with a pointer (represented as *u8) to itself
type GenericFinalizer = fn(*mut u8);

// manager
struct GcManager {
    first_node: *mut GcNode,
    last_node: *mut GcNode,
    roots: std::vec::Vec<*mut Vm>
}

impl GcManager {

    pub fn new() -> GcManager {
        GcManager {
            first_node: null_mut(),
            last_node: null_mut(),
            roots: Vec::new()
        }
    }

    pub unsafe fn malloc<T: Sized>
        (&mut self, x: T, finalizer: GenericFinalizer) -> *mut T {
        // tfw no qt malloc function
        let layout = Layout::from_size_align(GcNode::alloc_size::<T>(), 2).unwrap();
        let bytes : *mut GcNode = alloc_zeroed(layout) as *mut GcNode;
        // append node
        if self.first_node == null_mut() {
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
        // return (start byte+*struct GCNode)
        std::mem::replace(&mut *(bytes.add(1) as *mut T), x);
        bytes.add(1) as *mut T
    }

    pub unsafe fn free<T: Sized>(&mut self, x: *mut T) {
        // => start byte
        let node : *mut GcNode = (x as *mut GcNode).sub(1);

        if (*node).prev == null_mut() { self.first_node = (*node).next; }
        else { (*(*node).prev).next = (*node).next; }

        if (*node).next == null_mut() { self.last_node = (*node).prev; }
        else { (*(*node).next).prev = (*node).prev; }

        // call finalizer
        let finalizer = (*node).finalizer;
        finalizer(x as *mut u8);
        // free memory
        let layout = Layout::from_size_align((*node).size, 2).unwrap();
        dealloc(node as *mut u8, layout);
    }

    // roots
    pub fn add_root(&mut self, x: *mut Vm) {
        self.roots.push(x);
    }
    pub fn remove_root(&mut self, x: *mut Vm) {
        self.roots.remove_item(&x);
    }

    // gc algorithm
    pub fn collect(&mut self) {
        // mark phase:
        unsafe { // reset all nodes
            let mut node : *mut GcNode = self.first_node;
            while node != null_mut() {
                let next : *mut GcNode = (*node).next;
                (*node).unreachable = true;
                node = next;
            }
        }
        for root in self.roots.iter_mut() {
            let vm = unsafe { &mut **root };
            vm.mark();
        }
        // sweep phase:
        unsafe {
            let mut node : *mut GcNode = self.first_node;
            while node != null_mut() {
                let next : *mut GcNode = (*node).next;
                if (*node).unreachable {
                    let body = node.add(1);

                    // remove from ll
                    if (*node).prev == null_mut() { self.first_node = (*node).next; }
                    else { (*(*node).prev).next = (*node).next; }

                    if (*node).next == null_mut() { self.last_node = (*node).prev; }
                    else { (*(*node).next).prev = (*node).prev; }

                    // call finalizer
                    let finalizer = (*node).finalizer;
                    finalizer(body as *mut u8);

                    // free memory
                    let layout = Layout::from_size_align((*node).size, 2).unwrap();
                    dealloc(node as *mut u8, layout);
                }
                node = next;
            }
        }
    }

    // ## marking
    pub unsafe fn mark_reachable(ptr: *mut u8) {
        // => start byte
        let node : *mut GcNode = (ptr as *mut GcNode).sub(1);
        (*node).unreachable = false;
    }

}

unsafe impl std::marker::Send for GcManager {}
unsafe impl std::marker::Sync for GcManager {}

impl std::ops::Drop for GcManager {

    fn drop(&mut self) {
        unsafe {
            let mut node : *mut GcNode = self.first_node;
            while node != null_mut() {
                let next : *mut GcNode = (*node).next;
                let body = node.add(1);
                // call finalizer
                let finalizer = (*node).finalizer;
                finalizer(body as *mut u8);
                // free memory
                let layout = Layout::from_size_align((*node).size, 2).unwrap();
                dealloc(node as *mut u8, layout);
                node = next;
            }
        }
    }

}

// static allocator
use std::sync::Mutex;
lazy_static! {
    static ref GC_MANAGER_MUT : Mutex<GcManager> = Mutex::new(GcManager::new());
}

// general
pub unsafe fn malloc<T: Sized>(x: T, finalizer: GenericFinalizer) -> *mut T {
    let mut gc_manager = GC_MANAGER_MUT.lock().unwrap();
    gc_manager.collect();
    gc_manager.malloc(x, finalizer)
}
pub unsafe fn free<T: Sized>(x: *mut T) {
    let mut gc_manager = GC_MANAGER_MUT.lock().unwrap();
    gc_manager.free(x);
}

// roots
pub fn add_root(vm: *mut Vm) {
    let mut gc_manager = GC_MANAGER_MUT.lock().unwrap();
    gc_manager.add_root(vm);
}
pub fn remove_root(vm: *mut Vm) {
    let mut gc_manager = GC_MANAGER_MUT.lock().unwrap();
    gc_manager.remove_root(vm);
}

// collect
pub fn collect(vm: *mut Vm) {
    let mut gc_manager = GC_MANAGER_MUT.lock().unwrap();
    gc_manager.collect();
}
pub fn mark_reachable(ptr: *mut u8) {
    unsafe { GcManager::mark_reachable(ptr); }
}

// helpers
pub fn no_finalizer(_ : *mut u8) {}
pub unsafe fn drop<T>(ptr: *mut u8) {
    drop_in_place::<T>(ptr as *mut T);
}