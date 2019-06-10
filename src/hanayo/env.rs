//! Provides Env record for getting and setting environment variables
use crate::vmbindings::record::Record;
use crate::vmbindings::value::Value;
use crate::vmbindings::vm::Vm;
use std::borrow::Borrow;
use std::env;

#[hana_function()]
fn get(key: Value::Str) -> Value {
    match env::var(key.as_ref().borrow() as &String) {
        Ok(value) => Value::Str(vm.malloc(value.into())),
        Err(_) => Value::Nil,
    }
}

#[hana_function()]
fn set(key: Value::Str, val: Value::Str) -> Value {
    env::set_var(
        key.as_ref().borrow() as &String,
        val.as_ref().borrow() as &String,
    );
    Value::Nil
}

#[hana_function()]
fn vars() -> Value {
    let record = vm.malloc(Record::new());
    for (key, value) in env::vars() {
        record
            .as_mut()
            .insert(key, Value::Str(vm.malloc(value.into())).wrap());
    }
    Value::Record(record)
}
