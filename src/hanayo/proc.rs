//! Provides Proc record for handling child process spawned by Cmd
use std::io::Write;
use std::process::Child;

use crate::vmbindings::record::Record;
use crate::vmbindings::value::Value;
use crate::vmbindings::vm::Vm;
use crate::vmbindings::vmerror::VmError;

// inputs
#[hana_function()]
fn in_(process: Value::Record, input: Value::Str) -> Value {
    let field = process.as_mut().native_field.as_mut().unwrap();
    let p = field.downcast_mut::<Child>().unwrap();
    p.stdin
        .as_mut()
        .unwrap()
        .write_all(input.as_ref().as_bytes());
    Value::Record(process)
}

fn utf8_decoding_error(err: std::string::FromUtf8Error, vm: &Vm) -> Value {
    let rec = vm.malloc(Record::new());
    rec.as_mut().insert(
        "prototype",
        Value::Record(vm.stdlib.as_ref().unwrap().utf8_decoding_error.clone()).wrap(),
    );
    rec.as_mut().insert(
        "why",
        Value::Str(vm.malloc(format!("{:?}", err).into())).wrap(),
    );
    rec.as_mut().insert("where", Value::Int(0).wrap());
    Value::Record(rec)
}

// outs
#[hana_function()]
fn out(process: Value::Record) -> Value {
    // stdout as string
    let p = *process
        .as_mut()
        .native_field
        .take()
        .unwrap()
        .downcast::<Child>()
        .unwrap();
    let out = p.wait_with_output().unwrap();
    match String::from_utf8(out.stdout) {
        Ok(s) => Value::Str(vm.malloc(s.into())),
        Err(err) => {
            hana_raise!(vm, utf8_decoding_error(err, vm));
        }
    }
}

#[hana_function()]
fn err(process: Value::Record) -> Value {
    // stderr as string
    let p = *process
        .as_mut()
        .native_field
        .take()
        .unwrap()
        .downcast::<Child>()
        .unwrap();
    let out = p.wait_with_output().unwrap();
    match String::from_utf8(out.stderr) {
        Ok(s) => Value::Str(vm.malloc(s.into())),
        Err(err) => {
            hana_raise!(vm, utf8_decoding_error(err, vm));
        }
    }
}

#[hana_function()]
fn outputs(process: Value::Record) -> Value {
    // array of [stdout, stderr] outputs
    let p = *process
        .as_mut()
        .native_field
        .take()
        .unwrap()
        .downcast::<Child>()
        .unwrap();
    let out = p.wait_with_output().unwrap();
    let arr = vm.malloc(Vec::new());
    match String::from_utf8(out.stdout) {
        Ok(s) => arr.as_mut().push(Value::Str(vm.malloc(s.into())).wrap()),
        Err(err) => {
            hana_raise!(vm, utf8_decoding_error(err, vm));
        }
    }
    match String::from_utf8(out.stderr) {
        Ok(s) => arr.as_mut().push(Value::Str(vm.malloc(s.into())).wrap()),
        Err(err) => {
            hana_raise!(vm, utf8_decoding_error(err, vm));
        }
    }
    Value::Array(arr)
}

// other
#[hana_function()]
fn wait(process: Value::Record) -> Value {
    let field = process.as_mut().native_field.as_mut().unwrap();
    let p = field.downcast_mut::<Child>().unwrap();
    match p.wait() {
        Ok(e) => {
            if let Some(code) = e.code() {
                Value::Int(code as i64)
            } else {
                Value::Int(0)
            }
        }
        Err(_) => Value::Nil,
    }
}

#[hana_function()]
fn kill(process: Value::Record) -> Value {
    let field = process.as_mut().native_field.as_mut().unwrap();
    let p = field.downcast_mut::<Child>().unwrap();
    match p.kill() {
        Ok(()) => Value::Int(1),
        Err(_) => Value::Int(0),
    }
}
