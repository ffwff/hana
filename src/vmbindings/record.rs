use std::ptr::null;
use super::chmap::CHashMap;
use super::cnativeval::NativeValue;
use super::value::Value;

#[repr(C)]
pub struct Record {
    data: CHashMap,
    prototype: *const Record
}

impl Record {

    pub fn new() -> Record {
        Record {
            data: std::collections::HashMap::new(),
            prototype: null()
        }
    }

    pub fn get(&self, k: &String) -> Option<&NativeValue> {
        if let Some(v) = self.data.get(k) {
            return Some(v);
        } else if self.prototype != null() {
            let prototype = unsafe{ &*self.prototype };
            return prototype.get(k);
        }
        None
    }

    pub fn insert(&mut self, k: String, v: NativeValue) {
        if k == "prototype" {
            self.prototype = match v.unwrap_mut() {
                Value::mut_Record(x) => x,
                _ => panic!("unk prototype")
            };
        }
        self.data.insert(k, v);
    }

    pub fn iter(&self) -> std::collections::hash_map::Iter<String, NativeValue> {
        self.data.iter()
    }

}