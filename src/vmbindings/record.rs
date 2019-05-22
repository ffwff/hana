use std::any::Any;
use std::boxed::Box;
use super::chmap::CHashMap;
use super::cnativeval::NativeValue;
use super::gc::GcTraceable;
use super::value::Value;

#[repr(C)]
pub struct Record {
    data: CHashMap,
    prototype: Option<&'static Record>,
    // it says static but it lasts as long as Record, see below!
    pub native_field: Option<Box<Any>>,
}

impl Record {

    pub fn new() -> Record {
        Record {
            data: std::collections::HashMap::new(),
            prototype: None,
            native_field: None
        }
    }

    pub fn get(&self, k: &String) -> Option<&NativeValue> {
        if let Some(v) = self.data.get(k) {
            return Some(v);
        } else if let Some(prototype) = self.prototype {
            return prototype.get(k);
        }
        None
    }

    pub fn insert(&mut self, k: String, v: NativeValue) {
        if k == "prototype" {
            self.prototype = match v.unwrap() {
                // since the borrow checker doesn't know that self.prototype
                // can last as long as self, we'll have to use unsafe
                Value::Record(x) => Some(unsafe{ &*x.to_raw() }),
                _ => None
            };
        }
        self.data.insert(k, v);
    }

    pub fn remove(&mut self, k: &String) -> Option<NativeValue> {
        self.data.remove(k)
    }

    pub fn iter(&self) -> std::collections::hash_map::Iter<String, NativeValue> {
        self.data.iter()
    }

}

impl GcTraceable for Record {

    fn trace(ptr: *mut libc::c_void) {
        let self_ = unsafe{ &*(ptr as *mut Self) };
        for (_, val) in self_.iter() {
            val.trace();
        }
    }

}