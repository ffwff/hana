use super::cnativeval::NativeValue;
use super::env::Env;
use std::ptr::{null, null_mut};

// functions
#[repr(C)]
pub struct Function {
    ip: u32,
    nargs : u16,
    bound: std::boxed::Box<Env>
}

impl Function {

    pub unsafe fn new(ip: u32, nargs: u16, env: *const Env) -> Function {
        Function {
            ip: ip,
            nargs: nargs,
            bound: Box::new(
                    if env == null_mut() {
                        Env::new(null_mut(), 0, null_mut(), nargs) }
                    else { Env::copy(&*env) }
                )
        }
    }

    pub fn mark(&self) {
        /* if self.bound.nslots == 0 { return; }
        for i in 0..self.bound.nslots {
            self.bound.get(i).mark();
        } */
    }

}