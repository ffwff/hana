//! Provides Env record for getting and setting environment variables
use crate::vmbindings::record::Record;
use crate::vmbindings::value::Value;
use crate::vmbindings::vm::Vm;
use std::env;

#[hana_function()]
fn get(key: Value::Str) -> Value {
    match env::var(key.as_ref()) {
        Ok(value) => Value::Str(vm.malloc(value)),
        Err(_) => Value::Nil,
    }
}

#[hana_function()]
fn set(key: Value::Str, val: Value::Str) -> Value {
    env::set_var(key.as_ref(), val.as_ref());
    Value::Nil
}

#[hana_function()]
fn vars() -> Value {
    let record = vm.malloc(Record::new());
    for (key, value) in env::vars() {
        record
            .as_mut()
            .insert(key, Value::Str(vm.malloc(value)).wrap());
    }
    Value::Record(record)
}
