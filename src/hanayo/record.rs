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
    let mut array : CArray<NativeValue> = CArray::new();
    let p = pin_start();
    for (key, _) in rec.iter() {
        let s = Value::Str(Gc::new(key.clone()));
        array.push(s.wrap().pin());
    }
    Value::Array(Gc::new(array))
}