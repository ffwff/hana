//! Provides built-in math functions
use crate::vmbindings::vm::Vm;
use crate::vmbindings::value::Value;

#[hana_function()]
fn sqrt(val: Value::Float) -> Value {
    Value::Float(val.sqrt())
}