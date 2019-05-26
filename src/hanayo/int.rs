//! Provides Int record for handling integers
use std::str::FromStr;
use crate::vmbindings::vm::Vm;
use crate::vmbindings::value::Value;

#[hana_function()]
fn constructor(val: Value::Any) -> Value {
    match val {
        Value::Int(n) => Value::Int(n),
        Value::Float(n) => Value::Int(n as i64),
        Value::Str(s) => Value::Int(i64::from_str(s.as_ref())
            .unwrap_or_else(|_| panic!("cant convert to integer"))),
        _ => panic!("cant convert to integer")
    }
}

#[hana_function()]
fn chr(i: Value::Int) -> Value {
    if let Some(ch) = std::char::from_u32(i as u32) {
        Value::Str(vm.malloc(ch.to_string()))
    } else {
        Value::Nil
    }
}