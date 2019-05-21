use std::process::Command;
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

// output
#[hana_function()]
fn out(cmd: Value::mut_Record) -> Value {
    // stdout as string
    let field = cmd.native_field.as_mut().unwrap();
    let cmd = field.downcast_mut::<Command>().unwrap();
    let out = cmd.output().unwrap();
    Value::Str(unsafe { &*malloc(String::from_utf8(out.stdout)
        .unwrap_or_else(|e| panic!("error decoding stdout: {:?}", e)),
        |ptr| drop::<String>(ptr)) })
}

#[hana_function()]
fn err(cmd: Value::mut_Record) -> Value {
    // stderr as string
    let field = cmd.native_field.as_mut().unwrap();
    let cmd = field.downcast_mut::<Command>().unwrap();
    let out = cmd.output().unwrap();
    Value::Str(unsafe { &*malloc(String::from_utf8(out.stderr)
        .unwrap_or_else(|e| panic!("error decoding stderr: {:?}", e)),
        |ptr| drop::<String>(ptr)) })
}

#[hana_function()]
fn outputs(cmd: Value::mut_Record) -> Value {
    // stderr as string
    let field = cmd.native_field.as_mut().unwrap();
    let cmd = field.downcast_mut::<Command>().unwrap();
    let out = cmd.output().unwrap();
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