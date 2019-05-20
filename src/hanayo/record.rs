use crate::vmbindings::vm::Vm;
use crate::vmbindings::record::Record;
use crate::vmbindings::carray::CArray;
use crate::vmbindings::cnativeval::NativeValue;
use super::{malloc, drop};
use crate::vm::Value;

#[hana_function()]
fn constructor() -> Value {
    Value::Record(unsafe{ &*malloc(Record::new(), |ptr| drop::<Record>(ptr)) })
}

#[hana_function()]
fn keys(rec: Value::Record) -> Value {
    let mut array : CArray<NativeValue> = CArray::new();
    let p = pin_start();
    for (key, _) in rec.iter() {
        let s = Value::Str(unsafe {
                &*malloc(key.clone(), |ptr| drop::<String>(ptr)) });
        array.push(s.wrap().pin());
    }
    let ret = Value::Array(unsafe { &*malloc(array, |ptr|
        drop::<CArray<NativeValue>>(ptr)) });
    pin_end(p);
    ret
}