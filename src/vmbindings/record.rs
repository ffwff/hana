//! Provides a record value in Hana

use super::gc::{push_gray_body, GcNode, GcTraceable};
use super::hmap::HaruHashMap;
use super::string::HaruString;
use super::value::Value;
use std::any::Any;
use std::borrow::Borrow;
use std::boxed::Box;
use std::hash::Hash;

/// A record value in Hana
pub struct Record {
    data: HaruHashMap,
    prototype: Option<&'static Record>,
    // it says static but it lasts as long as Record, see below!
    /// Dynamic field for use in native functions
    pub native_field: Option<Box<Any>>,
}

impl Record {
    pub fn new() -> Record {
        Record {
            data: HaruHashMap::new(),
            prototype: None,
            native_field: None,
        }
    }

    pub fn with_capacity(n: usize) -> Record {
        Record {
            data: HaruHashMap::with_capacity(n),
            prototype: None,
            native_field: None,
        }
    }

    pub fn get<T: ?Sized>(&self, k: &T) -> Option<&Value>
    where
        HaruString: Borrow<T>,
        T: Hash + Eq,
    {
        if let Some(v) = self.data.get(k) {
            return Some(v);
        } else if let Some(prototype) = self.prototype {
            return prototype.get(k);
        }
        None
    }

    pub fn insert<K>(&mut self, k: K, v: Value)
    where
        K: Into<HaruString> + Hash + Eq,
    {
        let k = k.into();
        if (k.borrow() as &String) == "prototype" {
            self.prototype = unsafe {
                match &v {
                    // since the borrow checker doesn't know that self.prototype
                    // can last as long as self, we'll have to use unsafe
                    Value::Record(x) => Some(&*x.to_raw()),
                    _ => None,
                }
            };
        }
        self.data.insert(k, v);
    }

    pub fn iter(&self) -> hashbrown::hash_map::Iter<HaruString, Value> {
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
