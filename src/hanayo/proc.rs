use std::process::{Child};
use std::io::Write;
use crate::vmbindings::carray::CArray;

use crate::vmbindings::vm::Vm;
use crate::vmbindings::value::Value;

// inputs
#[hana_function()]
fn in_(process: Value::Record, input: Value::Str) -> Value {
    let field = process.as_mut().native_field.as_mut().unwrap();
    let p = field.downcast_mut::<Child>().unwrap();
    p.stdin.as_mut().unwrap().write_all(input.as_ref().as_bytes());
    Value::Record(process)
}

// outs
#[hana_function()]
fn out(process: Value::Record) -> Value {
    // stdout as string
    let p = *process.as_mut().native_field.take().unwrap().downcast::<Child>().unwrap();
    let out = p.wait_with_output().unwrap();
    Value::Str(vm.malloc(String::from_utf8(out.stdout)
        .unwrap_or_else(|e| panic!("error decoding stdout: {:?}", e))))
}

#[hana_function()]
fn err(process: Value::Record) -> Value {
    // stderr as string
    let p = *process.as_mut().native_field.take().unwrap().downcast::<Child>().unwrap();
    let out = p.wait_with_output().unwrap();
    Value::Str(vm.malloc(String::from_utf8(out.stderr)
        .unwrap_or_else(|e| panic!("error decoding stderr: {:?}", e))))
}

#[hana_function()]
fn outputs(process: Value::Record) -> Value {
    // array of [stdout, stderr] outputs
    let p = *process.as_mut().native_field.take().unwrap().downcast::<Child>().unwrap();
    let out = p.wait_with_output().unwrap();
    let arr = vm.malloc(CArray::new());
    arr.as_mut().push(Value::Str(vm.malloc(String::from_utf8(out.stdout)
        .unwrap_or_else(|e| panic!("error decoding stdout: {:?}", e)))).wrap());
    arr.as_mut().push(Value::Str(vm.malloc(String::from_utf8(out.stderr)
        .unwrap_or_else(|e| panic!("error decoding stderr: {:?}", e)))).wrap());
    Value::Array(arr)
}

// other
#[hana_function()]
fn wait(process: Value::Record) -> Value {
    let field = process.as_mut().native_field.as_mut().unwrap();
    let p = field.downcast_mut::<Child>().unwrap();
    match p.wait() {
        Ok(e) =>
            if let Some(code) = e.code() { Value::Int(code as i64) }
            else { Value::Int(0) },
        Err(_) => Value::Nil
    }
}

#[hana_function()]
fn kill(process: Value::Record) -> Value {
    let field = process.as_mut().native_field.as_mut().unwrap();
    let p = field.downcast_mut::<Child>().unwrap();
    match p.kill() {
        Ok(()) => Value::Int(1),
        Err(_) => Value::Int(0)
    }
}