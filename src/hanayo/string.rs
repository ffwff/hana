use crate::vmbindings::vm::Vm;
use crate::vmbindings::carray::CArray;
use crate::vmbindings::cnativeval::NativeValue;
use crate::vm::Value;
use super::{malloc, drop};

pub extern fn constructor(cvm : *mut Vm, nargs : u16) {
    let vm = unsafe { &mut *cvm };
    if nargs == 0 {
        vm.stack.push(unsafe { Value::Str(&*malloc(String::new(), |ptr| drop::<String>(ptr))) }.wrap());
        return;
    } else {
        assert_eq!(nargs, 1);
        let arg = vm.stack.top().clone().unwrap();
        vm.stack.pop();
        vm.stack.push(unsafe { Value::Str(&*malloc(format!("{:?}", arg).to_string(), |ptr| drop::<String>(ptr))) }.wrap());
    }
}

// length
#[hana_function()]
fn length(s: Value::Str) -> Value {
    Value::Int(0)
}
#[hana_function()]
fn bytesize(s: Value::Str) -> Value {
    Value::Int(0)
}

// check
#[hana_function()]
fn startswith(s: Value::Str) -> Value {
    Value::False
}
#[hana_function()]
fn endswith(s: Value::Str) -> Value {
    Value::False
}

// basic manip
#[hana_function()]
fn delete(s: Value::Str) -> Value {
    Value::False
}
#[hana_function()]
fn copy(s: Value::Str) -> Value {
    Value::False
}
#[hana_function()]
fn insert(s: Value::Str) -> Value {
    Value::False
}
#[hana_function()]
fn index(s: Value::Str) -> Value {
    Value::False
}

// other
#[hana_function()]
fn split(s: Value::Str) -> Value {
    Value::False
}