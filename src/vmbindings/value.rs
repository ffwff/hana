#[derive(PartialEq, Debug, Clone)]
pub enum Value {
    Nil,
    Int(i64),
    Float(f64),
    NativeFn,
    Fn,
    Str(&'static str),
    Dict,
    Array,
    NativeObj
}