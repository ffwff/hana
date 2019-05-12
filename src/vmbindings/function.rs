use super::cnativeval::NativeValue;
use super::env::Env;
use std::ptr::{null, null_mut};

// functions
#[repr(C)]
pub struct Function {
    pub ip: u32, // instruction pointer
    pub nargs : u16, // number of args
    pub bound: *mut Env,
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
            bound: Box::into_raw(Box::new(
                    if env.is_null() { Env::new(0, null_mut(), nargs) }
                    else { Env::copy(&*env) }
                ))
        }
    }

    pub fn drop(&mut self) { // must be called by the gc
        unsafe { Box::from_raw(self.bound); }
    }

    pub fn mark(&self) {
        unsafe {
            for i in 0..(*self.bound).nslots {
                (*self.bound).get(i).mark();
            }
        }
    }

}