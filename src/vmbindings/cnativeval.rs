use libc::c_void;
use super::record::Record;
use super::function::Function;
use super::carray::CArray;
use super::value::{Value, NativeFnData};
use super::gc::{mark_reachable, pin, Gc};

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
        _valueType::TYPE_FN         => unsafe {
                Value::Fn(Gc::from_raw(self.data as *mut Function))
            },
        _valueType::TYPE_STR        => unsafe {
                Value::Str(Gc::from_raw(self.data as *mut String))
            },
        _valueType::TYPE_DICT       => unsafe {
                Value::Record(Gc::from_raw(self.data as *mut Record))
            },
        _valueType::TYPE_ARRAY      => unsafe {
                Value::Array(Gc::from_raw(self.data as *mut CArray<NativeValue>))
            },
        }
    }

    pub fn mark(&self) {
        match self.r#type {
            _valueType::TYPE_FN   |
            _valueType::TYPE_STR  |
            _valueType::TYPE_DICT |
            _valueType::TYPE_ARRAY  =>
                if unsafe{ mark_reachable(self.data as *mut c_void) } {
                    self.unwrap().mark(); },
            _ => {}
        }
    }

}
