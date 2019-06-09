//! Provides Record record for handling records
use crate::vmbindings::record::Record;
use crate::vmbindings::value::Value;
use crate::vmbindings::vm::Vm;
use std::borrow::Borrow;

#[hana_function()]
fn constructor() -> Value {
    Value::Record(vm.malloc(Record::new()))
}

#[hana_function()]
fn keys(rec: Value::Record) -> Value {
    let array = vm.malloc(Vec::new());
    for (key, _) in rec.as_ref().iter() {
        array
            .as_mut()
            .push(Value::Str(vm.malloc(key.clone())).wrap());
    }
    Value::Array(array)
}

#[hana_function()]
fn has_key(rec: Value::Record, needle: Value::Str) -> Value {
    for (key, _) in rec.as_ref().iter() {
        if key == needle.as_ref() {
            return Value::True;
        }
    }
    Value::False
}
