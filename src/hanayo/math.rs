use crate::vmbindings::vm::Vm;
use crate::vm::Value;

#[hana_function()]
fn sqrt(val: Value::Float) -> Value {
    Value::Float(val.sqrt())
}