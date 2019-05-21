use super::env::Env;
use std::ptr::null_mut;
use crate::vmbindings::cnativeval::NativeValue;
use crate::vmbindings::gc::GcTraceable;

// functions
#[repr(C)]
#[derive(Clone)]
pub struct Function {
    pub ip: u32, // instruction pointer
    pub nargs : u16, // number of args

    // internal rust properties:
    pub bound: Env,
    // represents the current local environment
    // at the time the function is declared, this will be
    // COPIED into another struct env whenever OP_CALL is issued
    // (we use this to implement closures)
}

impl Function {

    pub unsafe fn new(ip: u32, nargs: u16, env: *const Env) -> Function {
        Function {
            ip: ip,
            nargs: nargs,
            bound: if env.is_null() { Env::new(0, null_mut(), nargs) }
                   else { Env::copy(&*env) }
        }
    }

    pub unsafe fn get_bound_ptr(&mut self) -> *mut Env {
        &mut self.bound
    }

}

// gc traceable
impl GcTraceable for Function {

    fn trace(ptr: *mut libc::c_void) {
        let self_ = unsafe{ &*(ptr as *mut NativeValue) };
        unimplemented!()
        /* pub fn mark(&self) {
            for val in self.bound.slots.iter() {
                val.mark();
            }
        } */
    }

}