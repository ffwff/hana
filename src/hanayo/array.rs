use std::cmp::Ordering;

use crate::vmbindings::vm::Vm;
use crate::vmbindings::carray::CArray;
use crate::vmbindings::cnativeval::{_valueType, NativeValue};
use super::{malloc, drop, pin_start, pin_end};
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

    let p = pin_start();

    let nargs = nargs as usize;
    let mut array : CArray<NativeValue> = CArray::reserve(nargs);
    for i in 0..nargs {
        let val = vm.stack.top();
        val.pin();
        array[i] = val.clone();
        vm.stack.pop();
    }
    vm.stack.push(Value::Array(unsafe {
                 &*malloc(array, alloc_free) }).wrap());

    pin_end(p);
}

#[hana_function()]
fn length(array: Value::Array) -> Value {
    Value::Int(array.len() as i64)
}

#[hana_function()]
fn insert_(array: Value::mut_Array, pos: Value::Int, elem: Value::Any) -> Value {
    array.insert(pos as usize, elem.wrap());
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

extern "C" {
fn value_gt(result: *mut NativeValue, left: NativeValue, right: NativeValue);
fn value_lt(result: *mut NativeValue, left: NativeValue, right: NativeValue);
fn value_is_true(left: NativeValue) -> bool;
}

// sorting
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
    let p = pin_start();
    for val in array.iter() {
        val.pin();
    }
    let slice = new_array.as_slice_mut();
    slice.sort_by(value_cmp);
    let arr = Value::Array(unsafe {
                 &*malloc(new_array, alloc_free) });
    pin_end(p);
    arr
}
#[hana_function()]
fn sort_(array: Value::mut_Array) -> Value {
    let slice = array.as_slice_mut();
    slice.sort_by(value_cmp);
    Value::Array(array)
}

// functional
pub extern fn map(cvm : *mut Vm, nargs : u16) {
    assert_eq!(nargs, 2);
    let vm = unsafe { &mut *cvm };

    let p = pin_start();

    let array = vm.stack.top().pin().unwrap().array();
    vm.stack.pop();

    let fun = vm.stack.top().pin().unwrap();
    vm.stack.pop();

    let mut new_array : CArray<NativeValue> = CArray::reserve(array.len());
    match fun {
        Value::Fn(_) | Value::Record(_) => {
            let mut args = CArray::reserve(1);
            let mut i = 0;
            for val in array.iter() {
                args[0] = val.clone();
                new_array[i] = vm.call(fun.wrap(), args.clone());
                i += 1;
            }
            args.drop();
        },
        Value::NativeFn(_) => {
            unimplemented!();
        }
        _ => panic!("expected fn/record/native fn")
    };

    vm.stack.push(Value::Array(unsafe { &*malloc(new_array, alloc_free) }).wrap());

    pin_end(p);
}

pub extern fn filter(cvm : *mut Vm, nargs : u16) {
    assert_eq!(nargs, 2);
    let vm = unsafe { &mut *cvm };

    let p = pin_start();

    let array = vm.stack.top().pin().unwrap().array();
    vm.stack.pop();

    let fun = vm.stack.top().pin().unwrap();
    vm.stack.pop();

    let mut new_array : CArray<NativeValue> = CArray::new();
    match fun {
        Value::Fn(_) | Value::Record(_) => {
            let mut args = CArray::reserve(1);
            for val in array.iter() {
                args[0] = val.clone();
                unsafe {
                    if value_is_true(vm.call(fun.wrap(), args.clone())) {
                        new_array.push(val.clone());
                    }
                }
            }
            args.drop();
        },
        Value::NativeFn(_) => {
            unimplemented!();
        }
        _ => panic!("expected fn/record/native fn")
    };

    vm.stack.push(Value::Array(unsafe { &*malloc(new_array, alloc_free) }).wrap());

    pin_end(p);
}

pub extern fn reduce(cvm : *mut Vm, nargs : u16) {
    assert_eq!(nargs, 3);
    let vm = unsafe { &mut *cvm };

    let p = pin_start();

    let array = vm.stack.top().pin().unwrap().array();
    vm.stack.pop();

    let fun = vm.stack.top().pin().unwrap();
    vm.stack.pop();

    let mut acc = vm.stack.top().pin().unwrap();
    vm.stack.pop();

    match fun {
        Value::Fn(_) | Value::Record(_) => {
            let mut args = CArray::reserve(2);
            for val in array.iter() {
                args[0] = acc.wrap().clone();
                args[1] = val.clone();
                unsafe {
                    acc = vm.call(fun.wrap(), args.clone()).unwrap();
                }
            }
            args.drop();
        },
        Value::NativeFn(_) => {
            unimplemented!();
        }
        _ => panic!("expected fn/record/native fn")
    };

    vm.stack.push(acc.wrap());

    pin_end(p);
}

// search
extern "C" {
fn value_eq(result: *mut NativeValue, left: NativeValue, right: NativeValue);
}
#[hana_function()]
fn index(array: Value::Array, elem: Value::Any) -> Value {
    for i in 0..array.len() {
        let mut val = NativeValue { data: 0, r#type: _valueType::TYPE_NIL };
        unsafe { value_eq(&mut val, array[i], elem.wrap()); }
        if val.data == 1 {
            return Value::Int(i as i64);
        }
    }
    Value::Int(-1)
}

// strings
#[hana_function()]
fn join(array: Value::Array, delim: Value::Str) -> Value {
    let mut s = String::new();
    if array.len() > 0 {
        s += format!("{:?}", array[0].unwrap()).as_str();
    }
    if array.len() > 1 {
        let mut i = 1;
        while i < array.len() {
            s += delim;
            s += format!("{:?}", array[i].unwrap()).as_str();
            i += 1;
        }
    }
    Value::Str(unsafe { &*malloc(s, |ptr| drop::<String>(ptr)) })
}