use crate::vmbindings::vm::Vm;
use crate::vmbindings::carray::CArray;
use crate::vmbindings::cnativeval::NativeValue;
use crate::vm::Value;
use super::{malloc, drop};

fn alloc_free(ptr: *mut libc::c_void) {
    unsafe { drop::<String>(ptr) };
}

pub extern fn constructor(cvm : *mut Vm, nargs : u16) {
    let vm = unsafe { &mut *cvm };
    if nargs == 0 {
        vm.stack.push(Value::Str(unsafe {
                &*malloc(String::new(), alloc_free) }).wrap());
        return;
    } else {
        assert_eq!(nargs, 1);
        let arg = vm.stack.top().clone().unwrap();
        vm.stack.pop();
        vm.stack.push(Value::Str(unsafe {
                &*malloc(format!("{:?}", arg).to_string(), alloc_free) }).wrap());
    }
}

// length
#[hana_function()]
fn length(s: Value::Str) -> Value {
    // NOTE: this is an O(n) operation due to utf8 decoding
    Value::Int(s.chars().count() as i64)
}
#[hana_function()]
fn bytesize(s: Value::Str) -> Value {
    Value::Int(s.len() as i64)
}

// check
#[hana_function()]
fn startswith(s: Value::Str, left: Value::Str) -> Value {
    Value::Int(s.starts_with(left) as i64)
}
#[hana_function()]
fn endswith(s: Value::Str, left: Value::Str) -> Value {
    Value::Int(s.ends_with(left) as i64)
}

// basic manip
#[hana_function()]
fn delete(s: Value::Str, from_pos: Value::Int, nchars: Value::Int) -> Value {
    let from_pos_u = from_pos as usize;
    let nchars_u = nchars as usize;
    let mut new_s = s.clone();
    new_s.replace_range(from_pos_u..from_pos_u+nchars_u, "");
    Value::Str(unsafe { &*malloc(new_s, alloc_free) })
}
#[hana_function()]
fn delete_(s: Value::mut_Str, from_pos: Value::Int, nchars: Value::Int) -> Value {
    let from_pos_u = from_pos as usize;
    let nchars_u = nchars as usize;
    s.replace_range(from_pos_u..from_pos_u+nchars_u, "");
    Value::Str(s)
}

#[hana_function()]
fn copy(s: Value::Str, from_pos: Value::Int, nchars: Value::Int) -> Value {
    let from_pos_u = from_pos as usize;
    let nchars_u = nchars as usize;
    let new_s = unsafe { &*malloc(s[from_pos_u..from_pos_u+nchars_u].to_string(), alloc_free) };
    Value::Str(new_s)
}

#[hana_function()]
fn insert_(dst: Value::mut_Str, from_pos: Value::Int, src: Value::Str) -> Value {
    let from_pos_u = from_pos as usize;
    dst.insert_str(from_pos_u, src);
    Value::Str(dst)
}

// other
#[hana_function()]
fn split(s: Value::Str, delim: Value::Str) -> Value {
    let mut array : CArray<NativeValue> = CArray::new();
    let p = pin_start();
    for ss in s.split(delim) {
        let val = Value::Str(unsafe {
                &*malloc(ss.clone().to_string(), alloc_free) });
        array.push(val.wrap().pin());
    }
    let ret = Value::Array(unsafe { &*malloc(array, |ptr|
        drop::<CArray<NativeValue>>(ptr)) });
    pin_end(p);
    ret
}

#[hana_function()]
fn index(s: Value::Str, needle: Value::Str) -> Value {
    match s.find(needle) {
        Some(x) => Value::Int(x as i64),
        None => Value::Int(-1)
    }
}

#[hana_function()]
fn chars(s: Value::Str) -> Value {
    let mut array : CArray<NativeValue> = CArray::new();
    let p = pin_start();
    for ss in s.chars() {
        let val = Value::Str(unsafe {
                &*malloc(ss.clone().to_string(), alloc_free) });
        array.push(val.wrap().pin());
    }
    let ret = Value::Array(unsafe { &*malloc(array, |ptr|
        drop::<CArray<NativeValue>>(ptr)) });
    pin_end(p);
    ret
}