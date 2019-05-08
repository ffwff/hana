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
        let fun = Function {
            ip: ip,
            nargs: nargs,
            bound: Box::into_raw(Box::new(
                    if env == null_mut() { Env::new(None, 0, None, nargs) }
                    else { Env::copy(&*env) }
                ))
        };
        fun
    }

}

impl std::ops::Drop for Function {

    fn drop(&mut self) {
        unimplemented!()
        /* unsafe {
            if self.bound.nargs > 0 {
                env_free(&mut self.bound);
            }
        } */
    }

}