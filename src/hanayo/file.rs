use std::fs::{File, OpenOptions};
use std::io::{Read, Write, Seek, SeekFrom};
use std::boxed::Box;

use crate::vmbindings::vm::Vm;
use crate::vmbindings::record::Record;
use super::Gc;
use crate::vm::Value;

#[hana_function()]
fn constructor(path : Value::Str, mode: Value::Str) -> Value {
    // options
    let mut options = OpenOptions::new();
    for ch in mode.as_ref().chars() {
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
    let rec = Gc::new(Record::new());
    // store native file
    rec.as_mut().native_field = Some(Box::new(options.open(path.as_ref()).unwrap()));
    // TODO: maybe not hardcode prototype it like this
    rec.as_mut().insert("prototype".to_string(), *vm.global().get(&"File".to_string()).unwrap());
    rec.as_mut().insert("path".to_string(), Value::Str(path).wrap());
    rec.as_mut().insert("mode".to_string(), Value::Str(mode).wrap());
    Value::Record(rec)
}

// reopen
#[hana_function()]
fn close(file: Value::Record) -> Value {
    file.as_mut().native_field = None;
    Value::Nil
}

// read
#[hana_function()]
fn read(file: Value::Record) -> Value {
    let field = file.as_ref().native_field.as_mut().unwrap();
    let file = field.downcast_mut::<File>().unwrap();
    let s = Gc::new(String::new());
    file.read_to_string(s.as_mut());
    Value::Str(s)
}

#[hana_function()]
fn read_up_to(file: Value::Record, n: Value::Int) -> Value {
    let field = file.as_ref().native_field.as_mut().unwrap();
    let file = field.downcast_mut::<File>().unwrap();
    let mut bytes : Vec<u8> = Vec::new();
    bytes.resize(n as usize, 0);
    if file.read_exact(&mut bytes).is_err() {
        panic!("unable to read exact!");
    }
    Value::Str(Gc::new(String::from_utf8(bytes)
        .unwrap_or_else(|e| panic!("error decoding file: {:?}", e))))
}

// write
#[hana_function()]
fn write(file: Value::Record, buf: Value::Str) -> Value {
    let file = file.as_mut();
    if let Some(field) = file.native_field.as_mut() {
        let file = field.downcast_mut::<File>().unwrap();
        Value::Int(file.write_all(buf.as_ref().as_bytes()).is_ok() as i64)
    } else {
        Value::Int(0)
    }
}

// positioning
#[hana_function()]
fn seek(file: Value::Record, pos: Value::Int) -> Value {
    let file = file.as_mut();
    if let Some(field) = file.native_field.as_mut() {
        let file = field.downcast_mut::<File>().unwrap();
        if let Result::Ok(result) = file.seek(SeekFrom::Current(pos)) {
            Value::Int(result as i64)
        } else {
            Value::Int(-1)
        }
    } else { Value::Int(-1) }
}

#[hana_function()]
fn seek_from_start(file: Value::Record, pos: Value::Int) -> Value {
    let file = file.as_mut();
    if let Some(field) = file.native_field.as_mut() {
        let file = field.downcast_mut::<File>().unwrap();
        if let Result::Ok(result) = file.seek(SeekFrom::Start(pos as u64)) {
            Value::Int(result as i64)
        } else {
            Value::Int(-1)
        }
    } else { Value::Int(-1) }
}

#[hana_function()]
fn seek_from_end(file: Value::Record, pos: Value::Int) -> Value {
    let file = file.as_mut();
    if let Some(field) = file.native_field.as_mut() {
        let file = field.downcast_mut::<File>().unwrap();
        if let Result::Ok(result) = file.seek(SeekFrom::End(pos)) {
            Value::Int(result as i64)
        } else {
            Value::Int(-1)
        }
    } else { Value::Int(-1) }
}