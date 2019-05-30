//! Provides Float record for handling floating point numbers
use std::str::FromStr;

use crate::vmbindings::record::Record;
use crate::vmbindings::value::Value;
use crate::vmbindings::vm::Vm;
use crate::vmbindings::vmerror::VmError;

#[hana_function()]
fn constructor(val: Value::Any) -> Value {
    match val {
        Value::Int(n) => Value::Float(n as f64),
        Value::Float(n) => Value::Float(n),
        Value::Str(s) => match f64::from_str(s.as_ref()) {
            Ok(n) => Value::Float(n),
            Err(_) => {
                hana_raise!(vm, {
                    let rec = vm.malloc(Record::new());
                    rec.as_mut().insert(
                        "prototype",
                        Value::Record(vm.stdlib.as_ref().unwrap().invalid_argument_error.clone())
                            .wrap(),
                    );
                    rec.as_mut().insert(
                        "why",
                        Value::Str(vm.malloc("Can't convert string to float".to_string())).wrap(),
                    );
                    rec.as_mut().insert("where", Value::Int(0).wrap());
                    Value::Record(rec)
                });
            }
        },
        _ => {
            hana_raise!(vm, {
                let rec = vm.malloc(Record::new());
                rec.as_mut().insert(
                    "prototype",
                    Value::Record(vm.stdlib.as_ref().unwrap().invalid_argument_error.clone())
                        .wrap(),
                );
                rec.as_mut().insert(
                    "why",
                    Value::Str(vm.malloc("Can't convert value to float".to_string())).wrap(),
                );
                rec.as_mut().insert("where", Value::Int(0).wrap());
                Value::Record(rec)
            });
        }
    }
}
