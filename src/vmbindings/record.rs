//! Provides a record value in Hana

use super::chmap::CHashMap;
use super::cnativeval::{NativeValue, _valueType};
use super::gc::GcTraceable;
use super::value::Value;
use std::any::Any;
use std::borrow::Borrow;
use std::boxed::Box;
use std::hash::Hash;

#[repr(C)]
/// A record value in Hana
pub struct Record {
    data: CHashMap,
    prototype: Option<&'static Record>,
    // it says static but it lasts as long as Record, see below!
    /// Dynamic field for use in native functions
    pub native_field: Option<Box<Any>>,
}

impl Record {
    pub fn new() -> Record {
        Record {
            data: std::collections::HashMap::new(),
            prototype: None,
            native_field: None,
        }
    }

    pub fn get<T: ?Sized>(&self, k: &T) -> Option<&NativeValue>
    where
        String: Borrow<T>,
        T: Hash + Eq,
    {
        if let Some(v) = self.data.get(k) {
            return Some(v);
        } else if let Some(prototype) = self.prototype {
            return prototype.get(k);
        }
        None
    }

    pub fn insert<K>(&mut self, k: K, v: NativeValue)
    where
        K: std::string::ToString + Hash + Eq,
    {
        let k: String = k.to_string();
        if k == "prototype" {
            self.prototype = match &v.unwrap() {
                // since the borrow checker doesn't know that self.prototype
                // can last as long as self, we'll have to use unsafe
                Value::Record(x) => Some(unsafe { &*x.to_raw() }),
                _ => None,
            };
        }
        self.data.insert(k, v);
    }

    pub fn iter(&self) -> std::collections::hash_map::Iter<String, NativeValue> {
        self.data.iter()
    }
}

impl GcTraceable for Record {
    fn trace(ptr: *mut libc::c_void) {
        unsafe {
            let self_ = &*(ptr as *mut Self);
            for (_, val) in self_.iter() {
                val.trace();
            }
        }
    }
}
