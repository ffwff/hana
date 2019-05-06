use super::chmap::CHashMap;
use super::cfunction::Function;
use super::value::Value;

#[repr(u8)]
#[allow(non_camel_case_types, dead_code)]
#[derive(PartialEq, Clone)]
enum _valueType {
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

#[repr(C)]
#[derive(Clone)]
pub struct NativeValue {
    data : u64,
    r#type : _valueType
}

impl NativeValue {

    pub fn unwrap(&self) -> Value {
        use std::mem::transmute;
        #[allow(non_camel_case_types)]
        match &self.r#type {
_valueType::TYPE_NIL        => Value::Nil,
_valueType::TYPE_INT        => unsafe { Value::Int(transmute::<u64, i64>(self.data)) },
_valueType::TYPE_FLOAT      => unsafe { Value::Float(transmute::<u64, f64>(self.data)) },
_valueType::TYPE_NATIVE_FN  => Value::NativeFn,
_valueType::TYPE_FN         => unsafe {
        Value::Fn(&*transmute::<u64, *const Function>(self.data))
    },
_valueType::TYPE_STR        => unsafe {
        Value::Str(&*transmute::<u64, *const String>(self.data))
    },
_valueType::TYPE_DICT       => unsafe {
        Value::Dict(&*transmute::<u64, *const CHashMap>(self.data))
    },
_valueType::TYPE_ARRAY      => Value::Array,
_valueType::TYPE_NATIVE_OBJ => Value::NativeObj,
        }
    }

}
