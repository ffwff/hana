use super::cnativeval::NativeValue;
use std::ptr::{null, null_mut};

#[repr(C)]
pub struct Env {
    pub slots: *mut NativeValue,
    pub nslots : u16,
    nargs : u16,
    pub parent : *mut Env,
    lexical_parent : *mut Env,
    retip : u32,
}

impl Env {

    pub unsafe fn new() -> Env {
        Env {
            slots: null_mut(),
            nslots: 0,
            nargs: 0,
            parent: null_mut(),
            lexical_parent: null_mut(),
            retip: 0
        }
    }

}

// functions
#[repr(C)]
pub struct Function {
    ip: u32,
    nargs : u16,
    pub bound: Env
}

extern "C" {
    fn env_copy(dst: *mut Env, src: *const Env);
    fn env_free(env: *mut Env);
}

impl Function {

    pub unsafe fn new(ip: u32, nargs: u16, env: *const Env) -> Function {
        let mut fun = Function {
            ip: ip,
            nargs: nargs,
            bound: Env::new()
        };
        if env != null() {
            env_copy(&mut fun.bound, env);
        }
        fun
    }

}

impl std::ops::Drop for Function {

    fn drop(&mut self) {
        unsafe {
            if self.bound.nargs > 0 {
                env_free(&mut self.bound);
            }
        }
    }

}