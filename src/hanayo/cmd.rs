use std::process::{Command, Child, Stdio, Output};
use std::io::Write;
use crate::vmbindings::vm::Vm;
use crate::vmbindings::carray::CArray;
use crate::vmbindings::record::Record;
use crate::vmbindings::value::Value;

#[hana_function()]
fn constructor(val: Value::Any) -> Value {
    let cmd : Command = match val {
        Value::Array(arr) => {
            let arr = arr.as_ref();
            if arr.len() == 0 { panic!("expected array with at least 1 elem!"); }
            let mut cmd = Command::new(match arr[0].unwrap() {
                              Value::Str(s) => s.as_ref().clone(),
                              _ => unimplemented!()
                          });
            if arr.len() > 1 {
                let slice = &arr.as_slice()[1..];
                for val in slice {
                    match val.unwrap() {
                        Value::Str(s) => { cmd.arg(s.as_ref().clone()); },
                        _ => { unimplemented!(); }
                    }
                }
            }
            cmd
        },
        Value::Str(scmd) => {
            let mut cmd = Command::new("sh");
            cmd.arg("-c").arg(scmd.as_ref().clone());
            cmd
        },
        _ => panic!("expected val to be string or array")
    };
    // cmd object
    let rec = vm.malloc(Record::new());
    // store native cmd
    rec.as_mut().native_field = Some(Box::new(cmd));
    rec.as_mut().insert("prototype",
        Value::Record(vm.stdlib.as_ref().unwrap().cmd_rec.clone()).wrap());
    Value::Record(rec)
}

// inputs
#[hana_function()]
fn in_(cmd: Value::Record, input: Value::Str) -> Value {
    cmd.as_mut().insert("input_buffer", Value::Str(input).wrap());
    Value::Record(cmd)
}

// outputs
// helper class
enum OutputResult {
    Process(Child),
    Output(Result<Output, std::io::Error>)
}

impl OutputResult {

    fn as_process(self) -> Child {
        match self {
            OutputResult::Process(x) => x,
            _ => unreachable!()
        }
    }

    fn as_output(self) -> Result<Output, std::io::Error> {
        match self {
            OutputResult::Output(x) => x,
            _ => unreachable!()
        }
    }

}

fn get_output(cmd: &mut Record, wait: bool) -> OutputResult {
    let field = cmd.native_field.as_mut().unwrap();
    let mut p = field.downcast_mut::<Command>().unwrap()
        .stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped())
        .spawn().unwrap();
    if let Some(val) = cmd.get(&"input_buffer".to_string()) {
        match val.unwrap() {
            Value::Str(s) => {
                p.stdin.as_mut().unwrap().write_all(s.as_ref().as_bytes());
            },
            _ => unimplemented!()
        }
    }
    if wait { OutputResult::Output(p.wait_with_output()) }
    else { OutputResult::Process(p) }
}

// impls
#[hana_function()]
fn out(cmd: Value::Record) -> Value {
    // stdout as string
    let out = get_output(cmd.as_mut(), true).as_output().unwrap();
    Value::Str(vm.malloc(String::from_utf8(out.stdout)
        .unwrap_or_else(|e| panic!("error decoding stdout: {:?}", e))))
}

#[hana_function()]
fn err(cmd: Value::Record) -> Value {
    // stderr as string
    let out = get_output(cmd.as_mut(), true).as_output().unwrap();
    Value::Str(vm.malloc(String::from_utf8(out.stderr)
        .unwrap_or_else(|e| panic!("error decoding stdout: {:?}", e))))
}

#[hana_function()]
fn outputs(cmd: Value::Record) -> Value {
    // array of [stdout, stderr] outputs
    let out = get_output(cmd.as_mut(), true).as_output().unwrap();
    let arr = vm.malloc(CArray::new());
    arr.as_mut().push(Value::Str(vm.malloc(String::from_utf8(out.stdout)
        .unwrap_or_else(|e| panic!("error decoding stdout: {:?}", e)))).wrap());
    arr.as_mut().push(Value::Str(vm.malloc(String::from_utf8(out.stderr)
        .unwrap_or_else(|e| panic!("error decoding stderr: {:?}", e)))).wrap());
    Value::Array(arr)
}

// spawn
#[hana_function()]
fn spawn(cmd: Value::Record) -> Value {
    let p = get_output(cmd.as_mut(), false).as_process();
    let prec = vm.malloc(Record::new());
    prec.as_mut().native_field = Some(Box::new(p));
    prec.as_mut().insert("prototype",
        Value::Record(vm.stdlib.as_ref().unwrap().proc_rec.clone()).wrap());
    Value::Record(prec)
}