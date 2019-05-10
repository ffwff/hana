use crate::vmbindings::vm::Vm;
use crate::vmbindings::carray::CArray;
use crate::vmbindings::cnativeval::NativeValue;
use crate::vm::Value;
use super::{malloc, drop};

pub extern fn constructor(cvm : *mut Vm, nargs : u16) {
    let vm = unsafe { &mut *cvm };
    if nargs == 0 {
        vm.stack.push(unsafe { Value::Str(&*malloc(String::new, |ptr| drop::<String>(ptr))) }.wrap());
        return;
    } else {
        assert_eq!(nargs, 1);
        let arg = vm.stack.top().clone();
        vm.stack.pop();
        vm.stack.push(unsafe { Value::Str(&*malloc(format!("{}", arg).to_string(), |ptr| drop::<String>(ptr))) }.wrap());
    }
}