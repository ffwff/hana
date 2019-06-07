//! Provides an abstraction for native values

use super::cnativeval::{NativeValue, NativeValueType};
use super::function::Function;
use super::gc::Gc;
use super::record::Record;
use super::vm::Vm;
extern crate libc;

pub type NativeFnData = extern "C" fn(*mut Vm, u16);

#[derive(Clone, PartialEq)]
#[allow(non_camel_case_types, dead_code)]
/// Wrapper for native values
pub enum Value {
    // we don't have control over how rust manages its variant
    // types, so this is a convenient wrapper for (de)serialising
    // hana's values
    Nil,
    True,
    False,

    Int(i64),
    Float(f64),
    NativeFn(NativeFnData),
    Fn(Gc<Function>),
    Str(Gc<String>),
    Record(Gc<Record>),
    Array(Gc<Vec<NativeValue>>),

    PropagateError,
}

#[allow(improper_ctypes)]
extern "C" {
    fn value_get_prototype(vm: *const Vm, val: NativeValue) -> *const Record;
    fn value_is_true(left: NativeValue, vm: *const Vm) -> bool;
}

impl Value {
    // wrapper for native
    pub fn wrap(&self) -> NativeValue {
        use std::mem::transmute;
        #[allow(non_camel_case_types)]
        unsafe {
            match &self {
                Value::Nil => NativeValue {
                    r#type: NativeValueType::TYPE_NIL,
                    data: 0,
                },
                Value::True => NativeValue {
                    r#type: NativeValueType::TYPE_INT,
                    data: 1,
                },
                Value::False => NativeValue {
                    r#type: NativeValueType::TYPE_INT,
                    data: 0,
                },
                Value::Int(n) => NativeValue {
                    r#type: NativeValueType::TYPE_INT,
                    data: transmute::<i64, u64>(*n),
                },
                Value::Float(n) => NativeValue {
                    r#type: NativeValueType::TYPE_FLOAT,
                    data: transmute::<f64, u64>(*n),
                },
                Value::NativeFn(f) => NativeValue {
                    r#type: NativeValueType::TYPE_NATIVE_FN,
                    data: transmute::<NativeFnData, u64>(*f),
                },
                Value::Fn(p) => NativeValue {
                    r#type: NativeValueType::TYPE_FN,
                    data: transmute::<*const Function, u64>(p.to_raw()),
                },
                Value::Str(p) => NativeValue {
                    r#type: NativeValueType::TYPE_STR,
                    data: transmute::<*const String, u64>(p.to_raw()),
                },
                Value::Record(p) => NativeValue {
                    r#type: NativeValueType::TYPE_DICT,
                    data: transmute::<*const Record, u64>(p.to_raw()),
                },
                Value::Array(p) => NativeValue {
                    r#type: NativeValueType::TYPE_ARRAY,
                    data: transmute::<*const Vec<NativeValue>, u64>(p.to_raw()),
                },
                _ => unimplemented!(),
            }
        }
    }

    // prototype
    pub fn get_prototype(&self, vm: *const Vm) -> *const Record {
        unsafe { value_get_prototype(vm, self.wrap()) }
    }

    // bool
    pub fn is_true(&self, vm: *const Vm) -> bool {
        unsafe { value_is_true(self.wrap(), vm) }
    }

    #[cfg_attr(tarpaulin, skip)]
    pub fn type_name(&self) -> &str {
        match self {
            Value::Nil => "nil",
            Value::Int(_) => "Int",
            Value::Float(_) => "Float",
            Value::NativeFn(_) | Value::Fn(_) => "Function",
            Value::Str(_) => "String",
            Value::Record(_) => "Record",
            Value::Array(_) => "Array",
            _ => "unk",
        }
    }
}

use std::fmt;

#[cfg_attr(tarpaulin, skip)]
impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Nil => write!(f, "[nil]"),
            Value::True => write!(f, "1"),
            Value::False => write!(f, "0"),
            Value::Int(n) => write!(f, "{}", n),
            Value::Float(n) => write!(f, "{}", n),
            Value::NativeFn(_) => write!(f, "[native fn]"),
            Value::Fn(_) => write!(f, "[fn]"),
            Value::Str(p) => write!(f, "{}", p.as_ref()),
            Value::Record(p) => write!(f, "[record {:p}]", p.to_raw()),
            Value::Array(p) => write!(f, "[array {:p}]", p.to_raw()),
            _ => unreachable!(),
        }
    }
}

#[cfg_attr(tarpaulin, skip)]
impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Nil => write!(f, "[nil]"),
            Value::Int(n) => write!(f, "{}", n),
            Value::Float(n) => write!(f, "{}", n),
            Value::NativeFn(_) => write!(f, "[native fn]"),
            Value::Fn(_) => write!(f, "[fn]"),
            Value::Str(p) => {
                let mut s = String::new();
                for ch in p.as_ref().chars() {
                    match ch {
                        '\n' => s.push_str("\\n"),
                        '"' => s.push('"'),
                        _ => s.push(ch),
                    }
                }
                write!(f, "\"{}\"", s)
            }
            Value::Record(p) => write!(f, "[record {:p}]", p.to_raw()),
            Value::Array(p) => write!(f, "[array {:p}]", p.to_raw()),
            _ => write!(f, "[unk]"),
        }
    }
}
