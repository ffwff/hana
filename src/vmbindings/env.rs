#[allow(unused_variables)]

use super::cnativeval::NativeValue;
use super::vm::Value;

#[derive(Clone)]
pub struct Env {
    pub slots: Vec<NativeValue>,
    // slot indexes access SHOULD be bounded whenever the script
    // is compiled to bytecode

    pub nargs : u16,
    // cached number of args the function was called with

    pub lexical_parent : *const Env,
    // lexical parents are the parent of the function's lexical scopes
    // this should be set to (struct function*)->bound

    pub retip : u32, // return ip, where to return to on pop
}

impl Env {

    pub fn new(retip: u32,
              lexical_parent: *const Env,
              nargs: u16) -> Env {
        Env {
            slots: Vec::new(),
            nargs: nargs,
            lexical_parent: lexical_parent,
            retip: retip
        }
    }

    pub fn copy(other : &Env) -> Env {
        Env {
            slots: other.slots.clone(),
            nargs: 0,
            lexical_parent: other.lexical_parent,
            retip: std::u32::MAX
        }
    }

    pub unsafe fn get(&self, idx: u16) -> NativeValue {
        self.slots.get_unchecked(idx as usize).clone()
    }
    pub unsafe fn get_up(&self, up: u16, idx: u16) -> NativeValue {
        let mut env = self.lexical_parent;
        for _ in 1..up {
            env = (*env).lexical_parent;
        }
        (*env).get(idx)
    }

    pub unsafe fn set(&mut self, idx: u16, val: NativeValue) {
        let elem = self.slots.get_unchecked_mut(idx as usize);
        *elem = val;
    }

    pub fn reserve(&mut self, nslots: u16) {
        self.slots.resize(nslots as usize, Value::Nil.wrap());
    }

}