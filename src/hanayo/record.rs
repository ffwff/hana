//! Provides Record record for handling records
use crate::vmbindings::valuearray::ValueArray;
use crate::vmbindings::record::Record;
use crate::vmbindings::value::Value;
use crate::vmbindings::vm::Vm;

#[hana_function()]
fn constructor() -> Value {
    Value::Record(vm.malloc(Record::new()))
}

#[hana_function()]
fn keys(rec: Value::Record) -> Value {
    let array = ValueArray::malloc(vm);
    for (key, _) in rec.as_ref().iter() {
        array.push(Value::Str(vm.malloc(key.clone())));
    }
    Value::Array(array)
}
