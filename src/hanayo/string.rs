use crate::vmbindings::vm::Vm;
use crate::vmbindings::carray::CArray;
use crate::vmbindings::cnativeval::NativeValue;
use crate::vm::Value;

#[hana_function()]
fn length(array: Value::Array) -> Value {
    Value::Int(array.len() as i64)
}
