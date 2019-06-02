//! Provides Array record for handling arrays
use std::cmp::Ordering;

use crate::vmbindings::cnativeval::{NativeValue, NativeValueType};
use crate::vmbindings::value::Value;
use crate::vmbindings::vm::Vm;

pub extern "C" fn constructor(cvm: *mut Vm, nargs: u16) {
    let vm = unsafe { &mut *cvm };
    if nargs == 0 {
        vm.stack.push(Value::Array(vm.malloc(Vec::new())).wrap());
        return;
    }

    let nargs = nargs as usize;
    let array = vm.malloc(Vec::with_capacity(nargs));
    for _i in 0..nargs {
        let val = vm.stack.pop().unwrap();
        array.as_mut().push(val.clone());
    }
    vm.stack.push(Value::Array(array).wrap());
}

#[hana_function()]
fn length(array: Value::Array) -> Value {
    Value::Int(array.as_ref().len() as i64)
}

#[hana_function()]
fn insert_(array: Value::Array, pos: Value::Int, elem: Value::Any) -> Value {
    array.as_mut().insert(pos as usize, elem.wrap());
    Value::Int(array.as_ref().len() as i64)
}

#[hana_function()]
fn delete_(array: Value::Array, from_pos: Value::Int, nelems: Value::Int) -> Value {
    array
        .as_mut()
        .drain((from_pos as usize)..((nelems as usize) + 1));
    Value::Int(array.as_ref().len() as i64)
}

// stack manipulation
#[hana_function()]
fn push(array: Value::Array, elem: Value::Any) -> Value {
    array.as_mut().push(elem.wrap());
    Value::Nil
}

#[hana_function()]
fn pop(array: Value::Array) -> Value {
    array.as_mut().pop().unwrap().unwrap()
}

extern "C" {
    fn value_gt(result: *mut NativeValue, left: NativeValue, right: NativeValue);
    fn value_lt(result: *mut NativeValue, left: NativeValue, right: NativeValue);
}

// sorting
fn value_cmp(left: &NativeValue, right: &NativeValue) -> Ordering {
    let left = left.clone();
    let right = right.clone();
    let mut val = NativeValue {
        data: 0,
        r#type: NativeValueType::TYPE_NIL,
    };

    unsafe {
        value_gt(&mut val, left, right);
    }
    if val.data == 1 {
        return Ordering::Greater;
    }

    unsafe {
        value_lt(&mut val, left, right);
    }
    if val.data == 1 {
        return Ordering::Less;
    }

    Ordering::Equal
}

#[hana_function()]
fn sort(array: Value::Array) -> Value {
    let new_array = vm.malloc(array.as_ref().clone());
    let slice = new_array.as_mut().as_mut_slice();
    slice.sort_by(value_cmp);
    Value::Array(new_array)
}
#[hana_function()]
fn sort_(array: Value::Array) -> Value {
    let slice = array.as_mut().as_mut_slice();
    slice.sort_by(value_cmp);
    Value::Array(array)
}

// functional
#[hana_function()]
fn map(array: Value::Array, fun: Value::Any) -> Value {
    let new_array = vm.malloc(Vec::with_capacity(array.as_ref().len()));
    let mut args = Vec::with_capacity(1);
    for val in array.as_ref().iter() {
        args.clear();
        args.push(val.clone());
        if let Some(val) = vm.call(fun.wrap(), &args) {
            new_array.as_mut().push(val);
        } else {
            return Value::PropagateError;
        }
    }
    Value::Array(new_array)
}

#[hana_function()]
fn filter(array: Value::Array, fun: Value::Any) -> Value {
    let new_array = vm.malloc(Vec::new());
    let mut args = Vec::with_capacity(1);
    for val in array.as_ref().iter() {
        args.clear();
        args.push(val.clone());
        if let Some(filter) = vm.call(fun.wrap(), &args) {
            if filter.unwrap().is_true(vm) {
                new_array.as_mut().push(val.clone());
            }
        } else {
            return Value::PropagateError;
        }
    }
    Value::Array(new_array)
}

#[hana_function()]
fn reduce(array: Value::Array, fun: Value::Any, acc_: Value::Any) -> Value {
    let mut acc = acc_.clone();
    let mut args = Vec::with_capacity(2);
    for val in array.as_ref().iter() {
        args.clear();
        args.push(acc.wrap().clone());
        args.push(val.clone());
        if let Some(val) = vm.call(fun.wrap(), &args) {
            acc = val.unwrap();
        } else {
            return Value::PropagateError;
        }
    }
    acc
}

// search
extern "C" {
    fn value_eq(result: *mut NativeValue, left: NativeValue, right: NativeValue);
}
#[hana_function()]
fn index(array: Value::Array, elem: Value::Any) -> Value {
    let array = array.as_ref();
    for i in 0..(array.len() - 1) {
        let mut val = NativeValue {
            data: 0,
            r#type: NativeValueType::TYPE_NIL,
        };
        unsafe {
            value_eq(&mut val, array[i], elem.wrap());
        }
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
    let array = array.as_ref();
    if array.len() > 0 {
        s += format!("{}", array[0].unwrap()).as_str();
    }
    if array.len() > 1 {
        let mut i = 1;
        while i < array.len() {
            s += delim.as_ref();
            s += format!("{}", array[i].unwrap()).as_str();
            i += 1;
        }
    }
    Value::Str(vm.malloc(s))
}
