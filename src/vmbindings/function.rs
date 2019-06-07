//! Provides a function value in Hana

use super::env::Env;
use crate::vmbindings::gc::{push_gray_body, GcNode, GcTraceable};
use std::ptr::null_mut;

#[repr(C)]
#[derive(Clone)]
pub struct Function {
    /// Starting instruction pointer of the function
    pub ip: u32,
    /// Number of args the function takes in
    pub nargs: u16,

    // internal rust properties:
    /// Represents the current local environment
    /// at the time the function is declared.
    ///
    /// This will be COPIED into another struct env whenever OP_CALL is issued.
    ///
    /// Wwe use this to implement closures.
    pub bound: Env,
}

impl Function {
    pub unsafe fn new(ip: u32, nargs: u16, env: *const Env) -> Function {
        Function {
            ip: ip,
            nargs: nargs,
            bound: if env.is_null() {
                Env::new(0, null_mut(), nargs)
            } else {
                Env::copy(&*env)
            },
        }
    }

    pub unsafe fn get_bound_ptr(&self) -> *const Env {
        &self.bound
    }
}

// gc traceable
impl GcTraceable for Function {
    unsafe fn trace(&self, gray_nodes: &mut Vec<*mut GcNode>) {
        for val in self.bound.slots.iter() {
            if let Some(ptr) = val.as_gc_pointer() {
                push_gray_body(gray_nodes, ptr);
            }
        }
    }
}
