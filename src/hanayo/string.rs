//! Provides String record for handling UTF-8 strings
extern crate unicode_segmentation;
use unicode_segmentation::UnicodeSegmentation;

use crate::vmbindings::valuearray::ValueArray;
use crate::vmbindings::value::Value;
use crate::vmbindings::vm::Vm;

pub extern "C" fn constructor(cvm: *mut Vm, nargs: u16) {
    let vm = unsafe { &mut *cvm };
    if nargs == 0 {
        vm.stack.push(Value::Str(vm.malloc(String::new())).wrap());
        return;
    } else {
        assert_eq!(nargs, 1);
        let arg = vm.stack.top().clone().unwrap();
        vm.stack.pop();
        vm.stack
            .push(Value::Str(vm.malloc(format!("{:?}", arg).to_string())).wrap());
    }
}

// length
#[hana_function()]
fn length(s: Value::Str) -> Value {
    Value::Int(s.as_ref().graphemes(true).count() as i64)
}
#[hana_function()]
fn bytesize(s: Value::Str) -> Value {
    Value::Int(s.as_ref().len() as i64)
}

// check
#[hana_function()]
fn startswith(s: Value::Str, left: Value::Str) -> Value {
    Value::Int(s.as_ref().starts_with(left.as_ref()) as i64)
}
#[hana_function()]
fn endswith(s: Value::Str, left: Value::Str) -> Value {
    Value::Int(s.as_ref().ends_with(left.as_ref()) as i64)
}

// basic manip
#[hana_function()]
fn delete(s: Value::Str, from_pos: Value::Int, nchars: Value::Int) -> Value {
    let from_pos = from_pos as usize;
    let nchars = nchars as usize;
    let ss = s
        .as_ref()
        .graphemes(true)
        .enumerate()
        .filter_map(|(i, ch)| {
            if (from_pos..(from_pos + nchars)).contains(&i) {
                None
            } else {
                Some(ch)
            }
        })
        .collect::<String>();
    Value::Str(vm.malloc(ss))
}

#[hana_function()]
fn delete_(s: Value::Str, from_pos: Value::Int, nchars: Value::Int) -> Value {
    let from_pos = from_pos as usize;
    let nchars = nchars as usize;
    let it = s.as_ref().grapheme_indices(true).skip(from_pos);
    if let Some((i, _)) = it.clone().take(1).next() {
        if let Some((j, _)) = it.skip(nchars).take(1).next() {
            s.as_mut().replace_range(i..j, "");
        } else {
            s.as_mut().remove(i);
        }
    }
    Value::Str(s)
}

#[hana_function()]
fn copy(s: Value::Str, from_pos: Value::Int, nchars: Value::Int) -> Value {
    let from_pos = from_pos as usize;
    let nchars = nchars as usize;
    let ss = s
        .as_ref()
        .graphemes(true)
        .enumerate()
        .filter_map(|(i, ch)| {
            if (from_pos..(from_pos + nchars)).contains(&i) {
                Some(ch)
            } else {
                None
            }
        })
        .collect::<String>();
    Value::Str(vm.malloc(ss))
}

#[hana_function()]
fn insert_(dst: Value::Str, from_pos: Value::Int, src: Value::Str) -> Value {
    let from_pos = from_pos as usize;
    if let Some((i, _)) = dst.as_ref().grapheme_indices(true).skip(from_pos).next() {
        dst.as_mut().insert_str(i, src.as_ref().as_str());
    }
    Value::Str(dst)
}

// other
#[hana_function()]
fn split(s: Value::Str, delim: Value::Str) -> Value {
    let array = vm.malloc(CArray::new());
    for ss in s.as_ref().split(delim.as_ref()) {
        array
            .as_mut()
            .push(Value::Str(vm.malloc(ss.clone().to_string())).wrap());
    }
    Value::Array(array)
}

#[hana_function()]
fn index(s: Value::Str, needle: Value::Str) -> Value {
    let s = s.as_ref();
    match s.find(needle.as_ref()) {
        Some(x) => Value::Int({
            let mut idx_grapheme = 0;
            if let Some(_) = s
                .grapheme_indices(true)
                .filter(|(i, _)| {
                    idx_grapheme += 1;
                    *i == x
                })
                .next()
            {
                (idx_grapheme - 1) as i64
            } else {
                -1
            }
        }),
        None => Value::Int(-1),
    }
}

#[hana_function()]
fn chars(s: Value::Str) -> Value {
    let array = ValueArray::malloc(vm);
    for ch in s.as_ref().graphemes(true) {
        array.push(Value::Str(vm.malloc(ch.to_string())));
    }
    Value::Array(array)
}

#[hana_function()]
fn ord(s: Value::Str) -> Value {
    let s = s.as_ref();
    if let Some(ch) = s.chars().next() {
        Value::Int(ch as i64)
    } else {
        Value::Int(0)
    }
}
