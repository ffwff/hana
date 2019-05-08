use super::cnativeval::{_valueType, NativeValue};
use std::ptr::{null, null_mut};
use std::vec;

#[repr(C)]
pub struct Env {
    pub slots: Vec<NativeValue>,
    pub nargs : u16,
    pub parent : Option<&'static Env>,
    pub lexical_parent : Option<&'static Env>,
    pub retip : u32,
}

impl Env {

    pub fn new(parent: Option<&'static Env>,
              retip: u32,
              lexical_parent: Option<&'static Env>,
              nargs: u16) -> Env {
        Env {
            slots: Vec::with_capacity(nargs as usize),
            nargs: nargs,
            parent: parent,
            lexical_parent: lexical_parent,
            retip: retip
        }
    }

    pub fn copy(e : &Env) -> Env {
        unimplemented!()
    }

    pub fn reserve(&mut self, nargs: u16) {
        self.slots.resize(nargs as usize,
            NativeValue{ data: 0, r#type: _valueType::TYPE_NIL })
    }

}