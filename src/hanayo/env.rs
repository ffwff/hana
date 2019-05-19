use std::env;
use crate::vmbindings::record::Record;
use crate::vmbindings::vm::Vm;
use super::{malloc, drop};
use crate::vm::Value;

#[hana_function()]
fn get(key: Value::Str) -> Value {
    match env::var(key) {
        Ok(value) => Value::Str(unsafe{
            &*malloc(value, |ptr| drop::<String>(ptr)) }),
        Err(_) => Value::Nil
    }
}

#[hana_function()]
fn set(key: Value::Str, val: Value::Str) -> Value {
    env::set_var(key, val);
    Value::Nil
}

#[hana_function()]
fn vars() -> Value {
    let mut record = Record::new();
    for (key, value) in env::vars() {
        record.insert(key, Value::Str(unsafe{
            &*malloc(value, |ptr| drop::<String>(ptr)) }).wrap().pin());
    }
    Value::Record(unsafe{ &*malloc(record, |ptr| drop::<Record>(ptr)) })
}