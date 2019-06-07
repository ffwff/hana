//! Provides a record value in Hana

use super::chmap::CHashMap;
use super::cnativeval::NativeValue;
use super::gc::{push_gray_body, GcNode, GcTraceable};
use super::value::Value;
use std::any::Any;
use std::borrow::Borrow;
use std::boxed::Box;
use std::hash::Hash;

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

    pub fn with_capacity(n: usize) -> Record {
        Record {
            data: std::collections::HashMap::with_capacity(n),
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
            self.prototype = unsafe{ match &v.unwrap() {
                // since the borrow checker doesn't know that self.prototype
                // can last as long as self, we'll have to use unsafe
                Value::Record(x) => Some(&*x.to_raw()),
                _ => None,
            } };
        }
        self.data.insert(k, v);
    }

    pub fn iter(&self) -> std::collections::hash_map::Iter<String, NativeValue> {
        self.data.iter()
    }

    pub fn is_prototype_of(&self, other: &Record) -> bool {
        let mut prototype = self.prototype.clone();
        while prototype.is_some() {
            let proto = prototype.unwrap();
            if proto as *const _ == other as *const _ {
                return true;
            }
            prototype = proto.prototype;
        }
        false
    }
}

impl GcTraceable for Record {
    unsafe fn trace(&self, gray_nodes: &mut Vec<*mut GcNode>) {
        for (_, val) in self.iter() {
            if let Some(ptr) = val.as_gc_pointer() {
                push_gray_body(gray_nodes, ptr);
            }
        }
    }
}
