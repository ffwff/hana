use crate::vmbindings::vm::Vm;
use crate::vmbindings::record::Record;
use crate::vmbindings::carray::CArray;
use crate::vmbindings::cnativeval::NativeValue;
use super::Gc;
use crate::vm::Value;

#[hana_function()]
fn constructor() -> Value {
    Value::Record(Gc::new(Record::new()))
}

#[hana_function()]
fn keys(rec: Value::Record) -> Value {
    let mut array = Gc::new(CArray::new());
    let keys = rec.as_ref().iter().map(|(key, _)| Value::Str(Gc::new(key.clone()))).collect();
    for key in keys {
        array.as_mut().push(keys.wrap());
    }
    Value::Array(array)
}