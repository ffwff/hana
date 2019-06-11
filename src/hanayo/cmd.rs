//! Provides Cmd record for executing and handling commands
use crate::vmbindings::record::Record;
use crate::vmbindings::value::Value;
use crate::vmbindings::vm::Vm;
use crate::vmbindings::vmerror::VmError;
use std::borrow::Borrow;
use std::io::Write;
use std::process::{Child, Command, Output, Stdio};

#[hana_function()]
fn constructor(val: Value::Any) -> Value {
    let cmd: Command = match val {
        Value::Array(arr) => {
            let arr = arr.as_ref();
            if arr.len() == 0 {
                let rec = vm.malloc(Record::new());
                rec.as_mut().insert(
                    "prototype",
                    Value::Record(vm.stdlib.as_ref().unwrap().invalid_argument_error.clone())
                        .wrap(),
                );
                rec.as_mut().insert(
                    "why",
                    Value::Str(
                        vm.malloc(
                            "Expected argument array to have at least 1 member"
                                .to_string()
                                .into(),
                        ),
                    )
                    .wrap(),
                );
                rec.as_mut().insert("where", Value::Int(0).wrap());
                hana_raise!(vm, Value::Record(rec));
            }
            let mut cmd = Command::new(match unsafe { arr[0].unwrap() } {
                Value::Str(s) => (s.as_ref().borrow() as &String).clone(),
                _ => {
                    let rec = vm.malloc(Record::new());
                    rec.as_mut().insert(
                        "prototype",
                        Value::Record(vm.stdlib.as_ref().unwrap().invalid_argument_error.clone())
                            .wrap(),
                    );
                    rec.as_mut().insert(
                        "why",
                        Value::Str(
                            vm.malloc("Expected command to be of string type".to_string().into()),
                        )
                        .wrap(),
                    );
                    rec.as_mut().insert("where", Value::Int(0).wrap());
                    hana_raise!(vm, Value::Record(rec));
                }
            });
            if arr.len() > 1 {
                let slice = &arr.as_slice()[1..];
                for val in slice {
                    match unsafe { val.unwrap() } {
                        Value::Str(s) => cmd.arg((s.as_ref().borrow() as &String).clone()),
                        _ => {
                            let rec = vm.malloc(Record::new());
                            rec.as_mut().insert(
                                "prototype",
                                Value::Record(
                                    vm.stdlib.as_ref().unwrap().invalid_argument_error.clone(),
                                )
                                .wrap(),
                            );
                            rec.as_mut().insert(
                                "why",
                                Value::Str(vm.malloc(
                                    "Expected argument to be of string type".to_string().into(),
                                ))
                                .wrap(),
                            );
                            rec.as_mut().insert("where", Value::Int(0).wrap());
                            hana_raise!(vm, Value::Record(rec));
                        }
                    };
                }
            }
            cmd
        }
        Value::Str(scmd) => {
            let mut cmd = Command::new("sh");
            cmd.arg("-c")
                .arg((scmd.as_ref().borrow() as &String).clone());
            cmd
        }
        _ => {
            let rec = vm.malloc(Record::new());
            rec.as_mut().insert(
                "prototype",
                Value::Record(vm.stdlib.as_ref().unwrap().invalid_argument_error.clone()).wrap(),
            );
            rec.as_mut().insert(
                "why",
                Value::Str(
                    vm.malloc(
                        "Expected argument to be of string or array type"
                            .to_string()
                            .into(),
                    ),
                )
                .wrap(),
            );
            rec.as_mut().insert("where", Value::Int(0).wrap());
            hana_raise!(vm, Value::Record(rec));
        }
    };
    // cmd object
    let rec = vm.malloc(Record::new());
    // store native cmd
    rec.as_mut().native_field = Some(Box::new(cmd));
    rec.as_mut().insert(
        "prototype",
        Value::Record(vm.stdlib.as_ref().unwrap().cmd_rec.clone()).wrap(),
    );
    Value::Record(rec)
}

// inputs
#[hana_function()]
fn in_(cmd: Value::Record, input: Value::Str) -> Value {
    cmd.as_mut()
        .insert("input_buffer", Value::Str(input).wrap());
    Value::Record(cmd)
}

// outputs
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

// helper class
enum OutputResult {
    Process(Child),
    Output(Result<Output, std::io::Error>),
}

impl OutputResult {
    fn as_process(self) -> Child {
        match self {
            OutputResult::Process(x) => x,
            _ => panic!("calling with wrong object, expected process"),
        }
    }

    fn as_output(self) -> Result<Output, std::io::Error> {
        match self {
            OutputResult::Output(x) => x,
            _ => panic!("calling with wrong object, expected output"),
        }
    }
}

fn get_output(cmd: &mut Record, wait: bool) -> OutputResult {
    let field = cmd.native_field.as_mut().unwrap();
    let mut p = field
        .downcast_mut::<Command>()
        .unwrap()
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    if let Some(val) = cmd.get(&"input_buffer".to_string()) {
        match unsafe { val.unwrap() } {
            Value::Str(s) => {
                p.stdin.as_mut().unwrap().write_all(s.as_ref().as_bytes());
            }
            _ => unimplemented!(),
        }
    }
    if wait {
        OutputResult::Output(p.wait_with_output())
    } else {
        OutputResult::Process(p)
    }
}

// impls
#[hana_function()]
fn out(cmd: Value::Record) -> Value {
    // stdout as string
    let out = get_output(cmd.as_mut(), true).as_output().unwrap();
    match String::from_utf8(out.stdout) {
        Ok(s) => Value::Str(vm.malloc(s.into())),
        Err(err) => {
            hana_raise!(vm, utf8_decoding_error(err, vm));
        }
    }
}

#[hana_function()]
fn err(cmd: Value::Record) -> Value {
    // stderr as string
    let out = get_output(cmd.as_mut(), true).as_output().unwrap();
    match String::from_utf8(out.stderr) {
        Ok(s) => Value::Str(vm.malloc(s.into())),
        Err(err) => {
            hana_raise!(vm, utf8_decoding_error(err, vm));
        }
    }
}

#[hana_function()]
fn outputs(cmd: Value::Record) -> Value {
    // array of [stdout, stderr] outputs
    let out = get_output(cmd.as_mut(), true).as_output().unwrap();
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

// spawn
#[hana_function()]
fn spawn(cmd: Value::Record) -> Value {
    let p = get_output(cmd.as_mut(), false).as_process();
    let prec = vm.malloc(Record::new());
    prec.as_mut().native_field = Some(Box::new(p));
    prec.as_mut().insert(
        "prototype",
        Value::Record(vm.stdlib.as_ref().unwrap().proc_rec.clone()).wrap(),
    );
    Value::Record(prec)
}
