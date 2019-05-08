use crate::vmbindings::vm::Vm;
use crate::vmbindings::carray::CArray;
use crate::vmbindings::cnativeval::NativeValue;
use crate::vm::Value;

pub extern fn constructor(cvm : *mut Vm, nargs : u16) {
    let vm = unsafe { &mut *cvm };
    if nargs == 0 {
        let array : CArray<NativeValue> = CArray::new();
        vm.stack.push(unsafe { Value::Array(&*Box::into_raw(Box::new(array))) }.wrap());
        return;
    }
    let nargs = nargs as usize;
    let mut array : CArray<NativeValue> = CArray::reserve(nargs);
    for i in 0..nargs {
        let val = vm.stack.top();
        array[i] = val.clone();
        vm.stack.pop();
    }
    vm.stack.push(unsafe { Value::Array(&*Box::into_raw(Box::new(array))) }.wrap());
}