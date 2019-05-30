//! Provides Int record for handling integers
use crate::vmbindings::value::Value;
use crate::vmbindings::record::Record;
use crate::vmbindings::vmerror::VmError;
use crate::vmbindings::vm::Vm;
use std::str::FromStr;

#[hana_function()]
fn constructor(val: Value::Any) -> Value {
    match val {
        Value::Int(n) => Value::Int(n),
        Value::Float(n) => Value::Int(n as i64),
        Value::Str(s) => match i64::from_str(s.as_ref()) {
            Ok(n) => Value::Int(n),
            Err(_) => {
                hana_raise!(vm, {
                    let rec = vm.malloc(Record::new());
                    rec.as_mut().insert("prototype", Value::Record(vm.stdlib.as_ref().unwrap().invalid_argument_error.clone()).wrap());
                    rec.as_mut().insert("why", Value::Str(vm.malloc("Can't convert string to integer".to_string())).wrap());
                    rec.as_mut().insert("where", Value::Int(0).wrap());
                    Value::Record(rec)
                });
            }
        },
        _ => {
            hana_raise!(vm, {
                let rec = vm.malloc(Record::new());
                rec.as_mut().insert("prototype", Value::Record(vm.stdlib.as_ref().unwrap().invalid_argument_error.clone()).wrap());
                rec.as_mut().insert("why", Value::Str(vm.malloc("Can't convert value to integer".to_string())).wrap());
                rec.as_mut().insert("where", Value::Int(0).wrap());
                Value::Record(rec)
            });
        }
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

#[hana_function()]
fn hex(i: Value::Int) -> Value {
    Value::Str(vm.malloc(format!("0x{:x}", i)))
}
