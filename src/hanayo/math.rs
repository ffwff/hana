//! Provides built-in math functions
use crate::vmbindings::value::Value;
use crate::vmbindings::vm::Vm;

#[hana_function()]
fn sqrt(val: Value::Float) -> Value {
    Value::Float(val.sqrt())
}
