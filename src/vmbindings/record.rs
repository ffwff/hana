use std::any::Any;
use std::boxed::Box;
use super::chmap::CHashMap;
use super::cnativeval::NativeValue;
use super::gc::Gc;
use super::value::Value;

#[repr(C)]
pub struct Record {
    data: CHashMap,
    prototype: Option<Gc<Record>>,
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
        } else if let Some(prototype) = &self.prototype {
            return prototype.as_ref().get(k);
        }
        None
    }

    pub fn insert(&mut self, k: String, v: NativeValue) {
        if k == "prototype" {
            self.prototype = match v.unwrap() {
                Value::Record(x) => Some(x),
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