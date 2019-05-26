//! Provides Float record for handling floating point numbers
use std::str::FromStr;

use crate::vmbindings::vm::Vm;
use crate::vmbindings::value::Value;

#[hana_function()]
fn constructor(val: Value::Any) -> Value {
    match val {
        Value::Int(n) => Value::Float(n as f64),
        Value::Float(n) => Value::Float(n),
        Value::Str(s) => Value::Float(f64::from_str(s.as_ref())
            .unwrap_or_else(|_| panic!("cant convert to float"))),
        _ => panic!("cant convert to float")
    }
}