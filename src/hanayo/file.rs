use std::fs::{File, OpenOptions};
use std::os::unix::io::{IntoRawFd, FromRawFd};
use std::io::{Read, Write, Seek, SeekFrom};
use std::mem::ManuallyDrop;
use std::borrow::BorrowMut;

use crate::vmbindings::vm::Vm;
use crate::vmbindings::record::Record;
use super::{malloc, drop};
use crate::vm::Value;

// manually drop because we don't want rust's RAII to automatically close the fd
// the gc should be doing it in the record's finaliser!
fn get_file_from_obj(rec: &Record) -> Option<ManuallyDrop<File>> {
    if let Some(fptr) = rec.get(&"fptr!".to_string()) {
        Some(ManuallyDrop::new(unsafe {
            File::from_raw_fd(fptr.unwrap().int() as i32) }))
    } else {
        None
    }
}

#[hana_function()]
fn constructor(path : Value::Str, mode: Value::Str) -> Value {
    // options
    let mut options = OpenOptions::new();
    for ch in mode.chars() {
        match ch {
            'r' => { options.read(true);       },
            'w' => { options.write(true);      },
            'c' => { options.create(true);     },
            'n' => { options.create_new(true); },
            'a' => { options.append(true);     },
            't' => { options.truncate(true);   },
            _ => panic!("expected options")
        }
    }

    // file object
    let mut rec = Record::new();
    // TODO: maybe not hardcode prototype it like this
    rec.insert("prototype".to_string(), *vm.global().get(&"File".to_string()).unwrap());
    rec.insert("path".to_string(), Value::Str(path).wrap());
    rec.insert("mode".to_string(), Value::Str(mode).wrap());
    rec.insert("fptr!".to_string(), Value::Int(options.open(path).unwrap().into_raw_fd() as i64).wrap());
    Value::Record(unsafe{ &*malloc(rec, |ptr| {
        // cleanup file descriptor
        let rec = &mut *(ptr as *mut Record);
        ManuallyDrop::into_inner(get_file_from_obj(rec).unwrap());
        // cleanup record
        drop::<Record>(ptr);
    }) })
}

// reopen
#[hana_function()]
fn close(rec: Value::mut_Record) -> Value {
    let file = get_file_from_obj(rec).unwrap();
    ManuallyDrop::into_inner(file);
    rec.insert("fptr!".to_string(), Value::Nil.wrap());
    Value::Nil
}

// read
#[hana_function()]
fn read(file: Value::Record) -> Value {
    let mut file = get_file_from_obj(file).unwrap();
    let mut s = String::new();
    file.borrow_mut().read_to_string(&mut s).unwrap_or(0);
    Value::Str(unsafe { &*malloc(s, |ptr| drop::<String>(ptr)) })
}

#[hana_function()]
fn read_up_to(file: Value::Record, n: Value::Int) -> Value {
    let mut file = get_file_from_obj(file).unwrap();
    let mut bytes : Vec<u8> = Vec::with_capacity(n as usize);
    file.borrow_mut().read_exact(bytes.as_mut_slice());
    Value::Str(unsafe { &*malloc(String::from_utf8(bytes)
        .unwrap_or_else(|e| panic!("error decoding file: {:?}", e)),
        |ptr| drop::<String>(ptr)) })
}

// write
#[hana_function()]
fn write(file: Value::Record, buf: Value::Str) -> Value {
    let mut file = get_file_from_obj(file).unwrap();
    Value::Int(file.borrow_mut().write_all(buf.as_bytes()).is_ok() as i64)
}

// positioning
#[hana_function()]
fn seek(file: Value::Record, pos: Value::Int) -> Value {
    let mut file = get_file_from_obj(file).unwrap();
    if let Result::Ok(result) = file.borrow_mut().seek(SeekFrom::Current(pos)) {
        Value::Int(result as i64)
    } else {
        Value::Int(-1)
    }
}

#[hana_function()]
fn seek_from_start(file: Value::Record, pos: Value::Int) -> Value {
    let mut file = get_file_from_obj(file).unwrap();
    if let Result::Ok(result) = file.borrow_mut().seek(SeekFrom::Start(pos as u64)) {
        Value::Int(result as i64)
    } else {
        Value::Int(-1)
    }
}

#[hana_function()]
fn seek_from_end(file: Value::Record, pos: Value::Int) -> Value {
    let mut file = get_file_from_obj(file).unwrap();
    if let Result::Ok(result) = file.borrow_mut().seek(SeekFrom::End(pos)) {
        Value::Int(result as i64)
    } else {
        Value::Int(-1)
    }
}