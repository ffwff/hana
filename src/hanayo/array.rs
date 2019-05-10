use crate::vmbindings::vm::Vm;
use crate::vmbindings::carray::CArray;
use crate::vmbindings::cnativeval::NativeValue;
use super::malloc;
use crate::vm::Value;

pub extern fn constructor(cvm : *mut Vm, nargs : u16) {
    fn alloc_free(ptr: *mut libc::c_void) {
        let array = unsafe { &mut *(ptr as *mut CArray<NativeValue>) };
        array.drop();
    }

    let vm = unsafe { &mut *cvm };
    if nargs == 0 {
        let array : CArray<NativeValue> = CArray::new();
        vm.stack.push(unsafe { Value::Array(&*malloc(array, alloc_free)) }.wrap());
        return;
    }
    let nargs = nargs as usize;
    let mut array : CArray<NativeValue> = CArray::reserve(nargs);
    for i in 0..nargs {
        let val = vm.stack.top();
        array[i] = val.clone();
        vm.stack.pop();
    }
    vm.stack.push(unsafe { Value::Array(&*malloc(array, alloc_free)) }.wrap());
}

#[hana_function()]
fn length(array: Value::Array) -> Value {
    Value::Int(array.len() as i64)
}

#[hana_function()]
fn delete_(array: Value::mut_Array, from_pos: Value::Int, nelems: Value::Int) -> Value {
    array.delete(from_pos as usize, nelems as usize);
    Value::Int(array.len() as i64)
}

// stack manipulation
#[hana_function()]
fn push(array: Value::mut_Array, elem: Value::Any) -> Value {
    array.push(elem.wrap());
    Value::Nil
}

#[hana_function()]
fn pop(array: Value::mut_Array) -> Value {
    let el = array.top().clone();
    array.pop();
    el.unwrap()
}

// sorting
#[hana_function()]
fn sort(array: Value::Array) -> Value {
    unimplemented!()
}
#[hana_function()]
fn sort_(array: Value::Array) -> Value {
    unimplemented!()
}

// functional
#[hana_function()]
fn map(array: Value::Array) -> Value {
    unimplemented!()
}
#[hana_function()]
fn filter(array: Value::Array) -> Value {
    unimplemented!()
}
#[hana_function()]
fn reduce(array: Value::Array) -> Value {
    unimplemented!()
}
#[hana_function()]
fn index(array: Value::Array, elem: Value::Any) -> Value {
    unimplemented!()
}

// strings
#[hana_function()]
fn join(array: Value::Array) -> Value {
    unimplemented!()
}