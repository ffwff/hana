use super::record::Record;
use super::carray::CArray;
use super::function::Function;
use super::vm::Vm;
use super::cnativeval::{NativeValue, _valueType};
use super::gc::*;
extern crate libc;

pub type NativeFnData = extern fn(*mut Vm, u16);

#[derive(Clone)]
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
    Fn(&'static Function),
    Str(&'static String),
    Record(&'static Record),
    Array(&'static CArray<NativeValue>),
    NativeObj,

    // mut wrappers
    mut_Fn(*mut Function),
    mut_Str(*mut String),
    mut_Record(*mut Record),
    mut_Array(*mut CArray<NativeValue>),
}

impl Value {

    pub fn string(&self) -> &'static String {
        match self {
            Value::Str(s) => s,
            _ => { panic!("Expected string"); }
        }
    }

    pub fn array(&self) -> &'static CArray<NativeValue> {
        match self {
            Value::Array(s) => s,
            _ => { panic!("Expected string"); }
        }
    }

    //
    pub fn wrap(&self) -> NativeValue {
        use std::mem::transmute;
        #[allow(non_camel_case_types)]
        unsafe { match &self {
            Value::Nil         => NativeValue { r#type: _valueType::TYPE_NIL, data: 0       },
            Value::True        => NativeValue { r#type: _valueType::TYPE_INT, data: 1       },
            Value::False       => NativeValue { r#type: _valueType::TYPE_INT, data: 0       },
            Value::Int(n)      => NativeValue { r#type: _valueType::TYPE_INT,
                                                data: transmute::<i64, u64>(*n)             },
            Value::Float(n)    => NativeValue { r#type: _valueType::TYPE_FLOAT,
                                                data: transmute::<f64, u64>(*n)             },
            Value::NativeFn(f) => NativeValue { r#type: _valueType::TYPE_NATIVE_FN,
                                                data: transmute::<NativeFnData, u64>(*f) },
            Value::Fn(_)       => NativeValue { r#type: _valueType::TYPE_FN,
                                                data: transmute::<*const Function, u64>(*s) },
            Value::Str(s)      => NativeValue { r#type: _valueType::TYPE_STR,
                                                data: transmute::<*const String, u64>(*s) },
            Value::Record(_)     => NativeValue { r#type: _valueType::TYPE_DICT , data: 0     },
            Value::Array(a)    => NativeValue { r#type: _valueType::TYPE_ARRAY,
                                                data: transmute::<*const CArray<NativeValue>, u64>(*a) },
            Value::NativeObj   => NativeValue { r#type: _valueType::TYPE_NATIVE_OBJ, data: 0},
            _ => unimplemented!()
        } }
    }

    // gc
    pub fn mark(&self) {
        match &self {
            Value::Fn(f)       => {
                f.mark();
            },
            Value::Record(d)   => {
                for (_, val) in d.iter() {
                    val.mark();
                }
            },
            Value::Array(a)    => {
                for val in a.iter() {
                    val.mark();
                }
            },
            _ => {}
        }
    }

}

// NOTE: you can't implement a trait for a specific variant
// in rust, neither can you implement a trait for a type alias, so I have
// to implement PartialEq by hand instead of deriving it for Value
use std::cmp::PartialEq;
impl PartialEq for Value {
    fn eq(&self, other: &Value) -> bool {
        match (self, other) {
            (Value::Nil,       Value::Nil)           => true,
            (Value::Int(x),    Value::Int(y))        => x == y,
            (Value::Float(x),  Value::Float(y))      => x == y,
            (Value::NativeFn(x), Value::NativeFn(y)) => std::ptr::eq(x, y),
            (Value::Fn(x),     Value::Fn(y))         => std::ptr::eq(x, y),
            (Value::Str(x),    Value::Str(y))        => x == y,
            (Value::Record(_), Value::Record(_))     => false,
            (Value::Array(_),  Value::Array(_))      => false,
            (Value::NativeObj, Value::NativeObj)     => false,
            _ => false
        }
    }
}

use std::fmt;
impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Nil         => write!(f, "[nil]"),
            Value::Int(n)      => write!(f, "{}", n),
            Value::Float(n)    => write!(f, "{}", n),
            Value::NativeFn(_) => write!(f, "[native fn]"),
            Value::Fn(x)       => write!(f, "[fn]"),
            Value::Str(s)      => write!(f, "{}", s),
            Value::Record(_)   => write!(f, "[record]"),
            Value::Array(_)    => write!(f, "[array]"),
            Value::NativeObj   => write!(f, "[native obj]"),
            _ => write!(f, "[unk]")
        }
    }
}