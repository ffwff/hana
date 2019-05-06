use super::chmap::CHashMap;

#[derive(Clone)]
pub enum Value {
    Nil,
    Int(i64),
    Float(f64),
    NativeFn,
    Fn,
    Str(&'static String),
    Dict(&'static CHashMap),
    Array,
    NativeObj
}

impl Value {

    pub fn string(&self) -> &'static String {
        match self {
            Value::Str(s) => s,
            _ => { panic!("Expected string"); }
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
            (Value::Nil,       Value::Nil)       => true,
            (Value::Int(x),    Value::Int(y))    => x == y,
            (Value::Float(x),  Value::Float(y))  => x == y,
            (Value::NativeFn,  Value::NativeFn)  => false,
            (Value::Fn,        Value::Fn)        => false,
            (Value::Str(x),    Value::Str(y))    => x == y,
            (Value::Dict(_),   Value::Dict(_))   => false,
            (Value::Array,     Value::Array)     => false,
            (Value::NativeObj, Value::NativeObj) => false,
            _ => false
        }
    }
}

use std::fmt;
impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[Value]")
    }
}