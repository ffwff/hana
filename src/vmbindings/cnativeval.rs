//! Provides the native value representation
//! used by the virtual machine

extern crate num_derive;
use num_traits::cast::{FromPrimitive, ToPrimitive};

use super::function::Function;
use super::gc::{ref_dec, ref_inc, Gc, GcManager, GcTraceable};
use super::record::Record;
use super::value::{NativeFnData, Value};

#[repr(u8)]
#[allow(non_camel_case_types, dead_code)]
#[derive(Debug, PartialEq, Clone, Copy, FromPrimitive, ToPrimitive)]
/// Type of the native value
pub enum NativeValueType {
    TYPE_FLOAT = 0,
    TYPE_INT = 1,
    TYPE_NATIVE_FN = 2,
    TYPE_FN = 3,
    TYPE_STR = 4,
    TYPE_DICT = 5,
    TYPE_ARRAY = 6,
    TYPE_INTERPRETER_ERROR = 7,
}

#[repr(transparent)]
#[derive(Debug, PartialEq, Clone, Copy)]
/// Native value representation used by the virtual machine
pub struct NativeValue(u64);


const RESERVED_NAN : u64 = 0x7ff;
const RESERVED_NAN_MASK : u64 = 0b01111111_11110000_00000000_00000000_00000000_00000000_00000000_00000000;
const INT_MASK          : u64 = 0b01111111_11110001_00000000_00000000_00000000_00000000_00000000_00000000;
const TAG_BIT_MASK      : u64 = 0b00000000_00001111_00000000_00000000_00000000_00000000_00000000_00000000;
const LOWER_MASK        : u64 = 0xffffffffffff;

impl NativeValue {
    pub fn tag(&self) -> NativeValueType {
        if !f64::is_nan(unsafe{ std::mem::transmute(self.0) }) {
            return NativeValueType::TYPE_FLOAT;
        }
        NativeValueType::from_u8(((self.0 & TAG_BIT_MASK) >> 48) as u8).unwrap()
    }
    fn get_low48(&self) -> u64 {
        self.0 & LOWER_MASK
    }

    pub fn new_i32(u: i32) -> NativeValue {
        NativeValue(INT_MASK | (u as u64))
    }
    pub fn new_f64(u: f64) -> NativeValue {
        NativeValue(unsafe{ std::mem::transmute(u) })
    }
    pub fn new_tagged_pointer<T>(tag: NativeValueType, ptr : *const T) -> NativeValue {
        let loptr = unsafe { std::mem::transmute::<_, u64>(ptr) & 0xffffffffffff };
        NativeValue((((RESERVED_NAN << 4) | tag.to_u64().unwrap()) << 48) | loptr)
    }

    /// Converts the native value into a wrapped Value.
    pub fn unwrap(&self) -> Value {
        use std::mem::transmute;
        unsafe {
            match self.tag() {
                NativeValueType::TYPE_FLOAT => Value::Float(transmute(self.0)),
                NativeValueType::TYPE_INT => Value::Int((self.0 & 0xffffffff) as i32),
                NativeValueType::TYPE_NATIVE_FN
                    => Value::NativeFn(transmute::<_, NativeFnData>(self.get_low48())),
                NativeValueType::TYPE_FN => Value::Fn(Gc::from_raw(transmute(self.get_low48()))),
                NativeValueType::TYPE_STR => Value::Str(Gc::from_raw(transmute(self.get_low48()))),
                NativeValueType::TYPE_DICT => Value::Record(Gc::from_raw(transmute(self.get_low48()))),
                NativeValueType::TYPE_ARRAY => Value::Array(Gc::from_raw(transmute(self.get_low48()))),
                _ => unimplemented!(),
            }
        }
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
            NativeValueType::TYPE_FN
            | NativeValueType::TYPE_STR
            | NativeValueType::TYPE_DICT
            | NativeValueType::TYPE_ARRAY => {
                ref_inc(self.data as *mut libc::c_void);
            }
            _ => {}
        } */
    }

    pub unsafe fn ref_dec(&self) {
        /* #[allow(non_camel_case_types)]
        match self.r#type {
            NativeValueType::TYPE_FN
            | NativeValueType::TYPE_STR
            | NativeValueType::TYPE_DICT
            | NativeValueType::TYPE_ARRAY => {
                ref_dec(self.data as *mut libc::c_void);
            }
            _ => {}
        } */
    }

    pub unsafe fn as_gc_pointer(&self) -> Option<*mut libc::c_void> {
        None
    }
}
