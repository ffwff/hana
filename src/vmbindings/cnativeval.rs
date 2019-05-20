use libc::c_void;
use super::record::Record;
use super::function::Function;
use super::carray::CArray;
use super::value::{Value, NativeFnData};
use super::gc::{mark_reachable, pin};

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
    TYPE_NATIVE_OBJ = 8,
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
                Value::Fn(&*(self.data as *const Function))
            },
        _valueType::TYPE_STR        => unsafe {
                Value::Str(&*(self.data as *const String))
            },
        _valueType::TYPE_DICT       => unsafe {
                Value::Record(&*(self.data as *const Record))
            },
        _valueType::TYPE_ARRAY      => unsafe {
                Value::Array(&*(self.data as *const CArray<NativeValue>))
            },
        _valueType::TYPE_NATIVE_OBJ => {
                Value::NativeObj(self.data as *mut libc::c_void)
            }
        }
    }

    pub fn unwrap_mut(&self) -> Value {
        use std::mem::transmute;
        #[allow(non_camel_case_types)]
        match &self.r#type {
        _valueType::TYPE_NIL        => Value::Nil,
        _valueType::TYPE_INT        =>
                Value::Int(self.data as i64),
        _valueType::TYPE_FLOAT      =>
                Value::Float(f64::from_bits(self.data)),
        _valueType::TYPE_NATIVE_FN  => unsafe {
                Value::NativeFn(transmute::<u64, NativeFnData>(self.data))
            },
        _valueType::TYPE_FN         => Value::mut_Fn(self.data as *mut Function),
        _valueType::TYPE_STR        => Value::mut_Str(self.data as *mut String),
        _valueType::TYPE_DICT       => Value::mut_Record(self.data as *mut Record),
        _valueType::TYPE_ARRAY      => Value::mut_Array(self.data as *mut CArray<NativeValue>),
        _valueType::TYPE_NATIVE_OBJ => Value::NativeObj(self.data as *mut libc::c_void),
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
            _valueType::TYPE_NATIVE_OBJ => unsafe {
                    mark_reachable(self.data as *mut c_void);
                },
            _ => {}
        }
    }

    pub fn pin(&self) -> NativeValue {
        match self.r#type {
            _valueType::TYPE_FN   |
            _valueType::TYPE_STR  |
            _valueType::TYPE_DICT |
            _valueType::TYPE_ARRAY  => {
                if pin(self.data as *mut c_void) {
                    self.unwrap().pin_rec(); }
                },
            _valueType::TYPE_NATIVE_OBJ => {
                    pin(self.data as *mut c_void);
                },
            _ => {}
        }
        *self
    }

}
