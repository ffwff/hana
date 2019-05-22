use crate::vmbindings::vm::Vm;
use crate::vmbindings::record::Record;
use crate::vmbindings::carray::CArray;
use super::Gc;
use crate::vm::Value;

#[hana_function()]
fn constructor() -> Value {
    Value::Record(Gc::new(Record::new()))
}

#[hana_function()]
fn keys(rec: Value::Record) -> Value {
    let array = Gc::new(CArray::new());
    for (key, _) in rec.as_ref().iter() {
        array.as_mut().push(Value::Str(Gc::new(key.clone())).wrap());
    }
    Value::Array(array)
}