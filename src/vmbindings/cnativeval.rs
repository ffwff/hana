//! Provides the native value representation
//! used by the virtual machine

use super::carray::CArray;
use super::function::Function;
use super::gc::{mark_reachable, ref_inc, ref_dec, Gc, GcTraceable};
use super::record::Record;
use super::value::{NativeFnData, Value};

#[repr(u8)]
#[allow(non_camel_case_types, dead_code)]
#[derive(Debug, PartialEq, Clone, Copy)]
/// Type of the native value
pub enum _valueType {
    TYPE_INT = 1,
    TYPE_NATIVE_FN = 2,
    TYPE_FN = 3,
    TYPE_STR = 4,
    TYPE_DICT = 5,
    TYPE_ARRAY = 6,
}

#[repr(transparent)]
#[derive(Debug, PartialEq, Clone, Copy)]
/// Native value representation used by the virtual machine
pub struct NativeValue(u64);

const RESERVED_NAN : u64 = 0x7ff;
const RESERVED_NAN_MASK : u64 = 0b01111111_11110000_00000000_00000000_00000000_00000000_00000000_00000000;
const TAG_BIT_MASK      : u64 = 0b00000000_00001111_00000000_00000000_00000000_00000000_00000000_00000000;

impl NativeValue {
    /// Converts the native value into a wrapped Value.
    pub fn unwrap(&self) -> Value {
        use std::mem::transmute;
        unsafe {
            if self.0 & RESERVED_NAN_MASK != RESERVED_NAN {
                return Value::Float(transmute(self.0));
            }
            let type = self.0 & TAG_BIT_MASK;
        }
        /* #[allow(non_camel_case_types)]
        match &self.r#type {
            NativeValueType::TYPE_NIL => Value::Nil,
            NativeValueType::TYPE_INT => unsafe { Value::Int(self.data as i32) },
            NativeValueType::TYPE_FLOAT => Value::Float(f64::from_bits(self.data)),
            NativeValueType::TYPE_NATIVE_FN => unsafe {
                Value::NativeFn(transmute::<u64, NativeFnData>(self.data))
            },
            _valueType::TYPE_FN => Value::Fn(Gc::from_raw(self.data as *mut Function)),
            _valueType::TYPE_STR => Value::Str(Gc::from_raw(self.data as *mut String)),
            _valueType::TYPE_DICT => Value::Record(Gc::from_raw(self.data as *mut Record)),
            _valueType::TYPE_ARRAY => {
                Value::Array(Gc::from_raw(self.data as *mut CArray<NativeValue>))
            }
        } */
    }

    /// Traces the native value recursively for use in the garbage collector.
    pub unsafe fn trace(&self) {
        /* #[allow(non_camel_case_types)]
        match self.r#type {
            _valueType::TYPE_FN => {
                if mark_reachable(self.data as *mut libc::c_void) {
                    Function::trace(self.data as *mut libc::c_void)
                }
            }
            _valueType::TYPE_STR => {
                mark_reachable(self.data as *mut libc::c_void);
            }
            _valueType::TYPE_DICT => {
                if mark_reachable(self.data as *mut libc::c_void) {
                    Record::trace(self.data as *mut libc::c_void)
                }
            }
            _valueType::TYPE_ARRAY => {
                if mark_reachable(self.data as *mut libc::c_void) {
                    CArray::trace(self.data as *mut libc::c_void)
                }
            }
            _ => {}
        } */
    }

    // reference counting
    pub unsafe fn ref_inc(&self) {
        /* #[allow(non_camel_case_types)]
        match self.r#type {
            _valueType::TYPE_FN
            | _valueType::TYPE_STR
            | _valueType::TYPE_DICT
            | _valueType::TYPE_ARRAY
            => {
                ref_inc(self.data as *mut libc::c_void);
            }
            _ => {}
        } */
    }

    pub unsafe fn ref_dec(&self) {
        /* #[allow(non_camel_case_types)]
        match self.r#type {
            _valueType::TYPE_FN
            | _valueType::TYPE_STR
            | _valueType::TYPE_DICT
            | _valueType::TYPE_ARRAY
            => {
                ref_dec(self.data as *mut libc::c_void);
            }
            _ => {}
        } */
    }
}
