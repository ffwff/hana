use std::cmp::Ordering;

use crate::vmbindings::vm::Vm;
use crate::vmbindings::carray::CArray;
use crate::vmbindings::cnativeval::{_valueType, NativeValue};
use super::malloc;
use crate::vm::Value;

fn alloc_free(ptr: *mut libc::c_void) {
    let array = unsafe { &mut *(ptr as *mut CArray<NativeValue>) };
    array.drop();
}

pub extern fn constructor(cvm : *mut Vm, nargs : u16) {

    let vm = unsafe { &mut *cvm };
    if nargs == 0 {
        let array : CArray<NativeValue> = CArray::new();
        vm.stack.push(Value::Array(unsafe {
                 &*malloc(array, alloc_free) }).wrap());
        return;
    }
    let nargs = nargs as usize;
    let mut array : CArray<NativeValue> = CArray::reserve(nargs);
    for i in 0..nargs {
        let val = vm.stack.top();
        array[i] = val.clone();
        vm.stack.pop();
    }
    vm.stack.push(Value::Array(unsafe {
                 &*malloc(array, alloc_free) }).wrap());
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
extern "C" {
fn value_gt(result: *mut NativeValue, left: NativeValue, right: NativeValue);
fn value_lt(result: *mut NativeValue, left: NativeValue, right: NativeValue);
}
fn value_cmp(left: &NativeValue, right: &NativeValue) -> Ordering {
    let left = left.clone();
    let right = right.clone();
    let mut val = NativeValue { data: 0, r#type: _valueType::TYPE_NIL };

    unsafe { value_gt(&mut val, left, right); }
    if val.data == 1 { return Ordering::Greater; }

    unsafe { value_lt(&mut val, left, right); }
    if val.data == 1 { return Ordering::Less; }

    Ordering::Equal
}

#[hana_function()]
fn sort(array: Value::Array) -> Value {
    let new_array = array.clone();
    let slice = new_array.as_slice_mut();
    slice.sort_by(value_cmp);
    Value::Array(unsafe {
                 &*malloc(new_array, alloc_free) })
}
#[hana_function()]
fn sort_(array: Value::mut_Array) -> Value {
    let slice = array.as_slice_mut();
    slice.sort_by(value_cmp);
    Value::Array(array)
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