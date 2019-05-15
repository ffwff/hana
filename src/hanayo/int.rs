use std::str::FromStr;
use crate::vmbindings::vm::Vm;
use crate::vm::Value;

#[hana_function()]
fn constructor(val: Value::Any) -> Value {
    match val {
        Value::Int(n) => Value::Int(n),
        Value::Float(n) => Value::Int(n as i64),
        Value::Str(s) => Value::Int(i64::from_str(s)
            .unwrap_or_else(|_| panic!("cant convert to integer"))),
        _ => panic!("cant convert to integer")
    }
}