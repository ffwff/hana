use std::process::{Command, Stdio, Output};
use std::io::Write;
use crate::vmbindings::vm::Vm;
use crate::vmbindings::carray::CArray;
use crate::vmbindings::cnativeval::NativeValue;
use crate::vmbindings::record::Record;
use crate::vm::Value;
use super::{malloc, drop};

#[hana_function()]
fn constructor(val: Value::Any) -> Value {
    let cmd : Command = match val {
        Value::Array(arr) => {
            if arr.len() == 0 { panic!("expected array with at least 1 elem!"); }
            let mut cmd = Command::new(arr[0].unwrap().string());
            if arr.len() > 1 {
                let slice = &arr.as_slice()[1..];
                for val in slice {
                    cmd.arg(val.unwrap().string());
                }
            }
            cmd
        },
        Value::Str(scmd) => {
            let mut cmd = Command::new("sh");
            cmd.arg("-c").arg(scmd);
            cmd
        },
        _ => panic!("expected val to be string or array")
    };
    // cmd object
    let mut rec = Record::new();
    // store native cmd
    rec.native_field = Some(Box::new(cmd));
    // TODO: maybe not hardcode prototype it like this
    rec.insert("prototype".to_string(), *vm.global().get(&"Cmd".to_string()).unwrap());
    Value::Record(unsafe{ &*malloc(rec, |ptr| drop::<Record>(ptr)) })
}

// inputs
#[hana_function()]
fn in_(cmd: Value::Record, input: Value::Str) -> Value {
    cmd.insert("input_buffer".to_string(), Value::Str(input).wrap());
    Value::Record(cmd)
}

// outputs
fn get_output(cmd: &mut Record) -> Result<Output, std::io::Error> {
    // TODO
    let field = cmd.native_field.as_mut().unwrap();
    let mut p = field.downcast_mut::<Command>().unwrap()
        .stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped())
        .spawn().unwrap();
    if let Some(val) = cmd.get(&"input_buffer".to_string()) {
        p.stdin.as_mut().unwrap().write_all(val.unwrap().string().as_bytes());
    }
    p.wait_with_output()
}

#[hana_function()]
fn out(cmd: Value::Record) -> Value {
    // stdout as string
    let out = get_output(cmd).unwrap();
    Value::Str(unsafe { &*malloc(String::from_utf8(out.stdout)
        .unwrap_or_else(|e| panic!("error decoding stdout: {:?}", e)),
        |ptr| drop::<String>(ptr)) })
}

#[hana_function()]
fn err(cmd: Value::Record) -> Value {
    // stderr as string
    let out = get_output(cmd).unwrap();
    Value::Str(unsafe { &*malloc(String::from_utf8(out.stderr)
        .unwrap_or_else(|e| panic!("error decoding stderr: {:?}", e)),
        |ptr| drop::<String>(ptr)) })
}

#[hana_function()]
fn outputs(cmd: Value::Record) -> Value {
    // stderr as string
    let out = get_output(cmd).unwrap();
    let mut arr = CArray::new();
    arr.push(Value::Str(unsafe { &*malloc(String::from_utf8(out.stdout)
        .unwrap_or_else(|e| panic!("error decoding stdout: {:?}", e)),
        |ptr| drop::<String>(ptr)) }).wrap().pin());
    arr.push(Value::Str(unsafe { &*malloc(String::from_utf8(out.stderr)
        .unwrap_or_else(|e| panic!("error decoding stderr: {:?}", e)),
        |ptr| drop::<String>(ptr)) }).wrap().pin());
    Value::Array(unsafe { &*malloc(arr, |ptr|
        drop::<CArray<NativeValue>>(ptr)) })
}