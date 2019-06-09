//! Provides Sys record
use crate::vmbindings::value::Value;
use crate::vmbindings::vm::Vm;

#[hana_function()]
fn args() -> Value {
    let array = vm.malloc(Vec::new());
    for arg in std::env::args().skip(1) {
        array
            .as_mut()
            .push(Value::Str(vm.malloc(arg.to_string().into())).wrap());
    }
    Value::Array(array)
}
