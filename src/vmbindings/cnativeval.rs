use super::record::Record;
use super::function::Function;
use super::carray::CArray;
use super::value::{Value, NativeFnData};
use super::gc::{mark_reachable, Gc, GcTraceable};

#[repr(u8)]
#[allow(non_camel_case_types, dead_code)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum _valueType {
    TYPE_NIL        = 0,
    TYPE_INT        = 1,
    TYPE_FLOAT      = 2,
    TYPE_NATIVE_FN  = 3,
    TYPE_FN         = 4,
    TYPE_STR        = 5,
    TYPE_DICT       = 6,
    TYPE_ARRAY      = 7,
}

#[repr(C, packed)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct NativeValue {
    pub data : u64,
    pub r#type : _valueType,
}

impl NativeValue {

    pub fn unwrap(&self) -> Value {
        use std::mem::transmute;
        #[allow(non_camel_case_types)]
        match &self.r#type {
            _valueType::TYPE_NIL        => Value::Nil,
            _valueType::TYPE_INT        => unsafe {
                Value::Int(transmute::<u64, i64>(self.data))
            },
            _valueType::TYPE_FLOAT      =>
                Value::Float(f64::from_bits(self.data)),
            _valueType::TYPE_NATIVE_FN  => unsafe {
                Value::NativeFn(transmute::<u64, NativeFnData>(self.data))
            },
            _valueType::TYPE_FN         =>
                Value::Fn(Gc::from_raw(self.data as *mut Function)),
            _valueType::TYPE_STR        =>
                Value::Str(Gc::from_raw(self.data as *mut String)),
            _valueType::TYPE_DICT       =>
                Value::Record(Gc::from_raw(self.data as *mut Record)),
            _valueType::TYPE_ARRAY      =>
                Value::Array(Gc::from_raw(self.data as *mut CArray<NativeValue>)),
        }
    }

    pub unsafe fn trace(&self) {
        if self.data == 0 { return; } // uninitialized memory
        #[allow(non_camel_case_types)]
        match self.r#type {
            _valueType::TYPE_FN         => {
                if mark_reachable(self.data as *mut libc::c_void) {
                    Function::trace(self.data as *mut libc::c_void)
                }
            },
            _valueType::TYPE_STR        => {
                mark_reachable(self.data as *mut libc::c_void);
            },
            _valueType::TYPE_DICT       => {
                if mark_reachable(self.data as *mut libc::c_void) {
                    Record::trace(self.data as *mut libc::c_void)
                }
            },
            _valueType::TYPE_ARRAY      => {
                if mark_reachable(self.data as *mut libc::c_void) {
                    CArray::trace(self.data as *mut libc::c_void)
                }
            },
            _ => {}
        }
    }

}