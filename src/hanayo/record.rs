//! Provides Record record for handling records
use crate::vmbindings::vm::Vm;
use crate::vmbindings::record::Record;
use crate::vmbindings::carray::CArray;
use crate::vmbindings::value::Value;

#[hana_function()]
fn constructor() -> Value {
    Value::Record(vm.malloc(Record::new()))
}

#[hana_function()]
fn keys(rec: Value::Record) -> Value {
    let array = vm.malloc(CArray::new());
    for (key, _) in rec.as_ref().iter() {
        array.as_mut().push(Value::Str(vm.malloc(key.clone())).wrap());
    }
    Value::Array(array)
}