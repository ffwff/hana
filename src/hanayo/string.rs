use crate::vmbindings::vm::Vm;
use crate::vmbindings::carray::CArray;
use crate::vmbindings::cnativeval::NativeValue;
use crate::vm::Value;
use super::Gc;

pub extern fn constructor(cvm : *mut Vm, nargs : u16) {
    let vm = unsafe { &mut *cvm };
    if nargs == 0 {
        vm.stack.push(Value::Str(Gc::new(String::from())).wrap());
        return;
    } else {
        assert_eq!(nargs, 1);
        let arg = vm.stack.top().clone().unwrap();
        vm.stack.pop();
        vm.stack.push(Value::Str(Gc::new(format!("{:?}", arg).to_string())).wrap());
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
    Value::Str(Gc::new(new_s))
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
    Value::Str(Gc::new(s[from_pos_u..from_pos_u+nchars_u].to_string()))
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
    let mut array = Gc::new(CArray::new());
    let sarray = s.split(delim).map(|ss| Value::Str(Gc::new(ss.clone().to_string())));
    for s in sarray {
        array.as_mut_ref().push(s.unwrap());
    }
    array
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
    for ss in s.chars() {
        let val = Value::Str(unsafe {
                &*malloc(ss.clone().to_string(), alloc_free) });
        array.push(val.wrap().pin());
    }
    let ret = Value::Array(unsafe { &*malloc(array, |ptr|
        drop::<CArray<NativeValue>>(ptr)) });
    ret
}

#[hana_function()]
fn ord(s: Value::Str) -> Value {
    if let Some(ch) = s.chars().next() {
        Value::Int(ch as i64)
    } else {
        Value::Int(0)
    }
}