#[allow(unused_variables)]

use super::cnativeval::NativeValue;
use std::ptr::null_mut;
use std::alloc::{alloc_zeroed, dealloc, Layout};

#[repr(C)]
#[derive(Clone)]
pub struct Env {
    pub slots: *mut NativeValue,
    pub nslots: u16,
    // raw slots array. rust doesn't have a non bounds-check index
    // function for arrays so we'll have to use this
    // slot indexes access SHOULD be bounded whenever the script
    // is compiled to bytecode

    pub nargs : u16,
    // cached number of args the function was called with

    pub lexical_parent : *mut Env,
    // lexical parents are the parent of the function's lexical scopes
    // this should be set to (struct function*)->bound

    pub retip : u32, // return ip, where to return to on pop
}

impl Env {

    pub fn new(retip: u32,
              lexical_parent: *mut Env,
              nargs: u16) -> Env {
        Env {
            slots: null_mut(),
            nslots: 0,
            nargs: nargs,
            lexical_parent: lexical_parent,
            retip: retip
        }
    }

    pub fn copy(other : &Env) -> Env {
        let mut env = Env {
            slots: null_mut(),
            nslots: other.nslots,
            nargs: 0,
            lexical_parent: other.lexical_parent,
            retip: std::u32::MAX
        };
        unsafe {
            env.reserve(other.nslots);
            std::ptr::copy(other.slots, env.slots, env.nslots as usize);
        };
        env
    }

    pub unsafe fn get(&self, idx: u16) -> NativeValue {
        (*self.slots.add(idx as usize)).clone()
    }
    pub unsafe fn get_up(&self, up: u16, idx: u16) -> NativeValue {
        let mut env : *mut Env = self.lexical_parent;
        for _ in 1..up {
            env = (*env).lexical_parent;
        }
        { (*env).get(idx) }
    }

    pub unsafe fn set(&mut self, idx: u16, val: NativeValue) {
        debug_assert!(idx <= self.nslots, "expected: {}", self.nslots);
        std::ptr::copy(&val, self.slots.add(idx as usize), 1);
    }
    pub unsafe fn set_up(&mut self, up: u16, idx: u16, val: NativeValue) {
        let mut env : *mut Env = self.lexical_parent;
        for _ in 1..up {
            env = (*env).lexical_parent;
        }
        (*env).set(idx, val)
    }

    pub unsafe fn reserve(&mut self, nslots: u16) {
        if nslots == 0 { self.slots = null_mut(); }
        else {
            if !self.slots.is_null() { // preallocated
                let layout = Layout::array::<NativeValue>(self.nslots as usize).unwrap();
                dealloc(self.slots as *mut u8, layout);
            }
            let layout = Layout::array::<NativeValue>(nslots as usize).unwrap();
            self.slots = alloc_zeroed(layout) as *mut NativeValue;
        }
        self.nslots = nslots;
    }

}

impl std::ops::Drop for Env {

    fn drop(&mut self) {
        if self.nslots == 0 { return; }
        let layout = Layout::array::<NativeValue>(self.nslots as usize).unwrap();
        unsafe { dealloc(self.slots as *mut u8, layout); }
    }

}