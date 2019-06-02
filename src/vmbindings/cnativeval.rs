//! Provides the native value representation
//! used by the virtual machine

use super::function::Function;
use super::gc::{ref_dec, ref_inc, Gc, GcManager, GcTraceable};
use super::record::Record;
use super::value::{NativeFnData, Value};

#[repr(u8)]
#[allow(non_camel_case_types, dead_code)]
#[derive(Debug, PartialEq, Clone, Copy)]
/// Type of the native value
pub enum _valueType {
    TYPE_NIL = 0,
    TYPE_INT = 1,
    TYPE_FLOAT = 2,
    TYPE_NATIVE_FN = 3,
    TYPE_FN = 4,
    TYPE_STR = 5,
    TYPE_DICT = 6,
    TYPE_ARRAY = 7,
}

#[repr(C, packed)]
#[derive(Debug, PartialEq, Clone, Copy)]
/// Native value representation used by the virtual machine
pub struct NativeValue {
    pub data: u64,
    pub r#type: _valueType,
}

impl NativeValue {
    /// Converts the native value into a wrapped Value.
    pub fn unwrap(&self) -> Value {
        use std::mem::transmute;
        #[allow(non_camel_case_types)]
        match &self.r#type {
            _valueType::TYPE_NIL => Value::Nil,
            _valueType::TYPE_INT => unsafe { Value::Int(transmute::<u64, i64>(self.data)) },
            _valueType::TYPE_FLOAT => Value::Float(f64::from_bits(self.data)),
            _valueType::TYPE_NATIVE_FN => unsafe {
                Value::NativeFn(transmute::<u64, NativeFnData>(self.data))
            },
            _valueType::TYPE_FN => unsafe {
                Value::Fn(Gc::from_raw(self.data as *mut Function))
            },
            _valueType::TYPE_STR => unsafe {
                Value::Str(Gc::from_raw(self.data as *mut String))
            },
            _valueType::TYPE_DICT => unsafe {
                Value::Record(Gc::from_raw(self.data as *mut Record))
            },
            _valueType::TYPE_ARRAY => unsafe {
                Value::Array(Gc::from_raw(self.data as *mut Vec<NativeValue>))
            }
        }
    }

    pub fn as_pointer(&self) -> Option<*mut libc::c_void> {
        #[allow(non_camel_case_types)]
        match self.r#type {
            _valueType::TYPE_FN
            | _valueType::TYPE_STR
            | _valueType::TYPE_DICT
            | _valueType::TYPE_ARRAY => {
                if self.data == 0 { None }
                else { Some(self.data as *mut libc::c_void) }
            },
            _ => None
        }
    }

    // reference counting
    pub unsafe fn ref_inc(&self) {
        #[allow(non_camel_case_types)]
        match self.r#type {
            _valueType::TYPE_FN
            | _valueType::TYPE_STR
            | _valueType::TYPE_DICT
            | _valueType::TYPE_ARRAY => {
                ref_inc(self.data as *mut libc::c_void);
            }
            _ => {}
        }
    }

    pub unsafe fn ref_dec(&self) {
        #[allow(non_camel_case_types)]
        match self.r#type {
            _valueType::TYPE_FN
            | _valueType::TYPE_STR
            | _valueType::TYPE_DICT
            | _valueType::TYPE_ARRAY => {
                ref_dec(self.data as *mut libc::c_void);
            }
            _ => {}
        }
    }
}
