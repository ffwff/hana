use super::cnativeval::NativeValue;
use super::env::Env;
use std::ptr::{null, null_mut};

// functions
#[repr(C)]
pub struct Function {
    ip: u32,
    nargs : u16,
    bound: *mut Env
}

impl Function {

    pub unsafe fn new(ip: u32, nargs: u16, env: *const Env) -> Function {
        Function {
            ip: ip,
            nargs: nargs,
            bound: Box::into_raw(Box::new(
                    if env == null_mut() {
                        Env::new(null_mut(), 0, None, nargs) }
                    else { Env::copy(&*env) }
                ))
        }
    }

}

impl std::ops::Drop for Function {

    fn drop(&mut self) {
        // TODO free bound
    }

}