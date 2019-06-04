//! Provides Int record for handling integers
use crate::vmbindings::record::Record;
use crate::vmbindings::value::Value;
use crate::vmbindings::vm::Vm;
use crate::vmbindings::vmerror::VmError;
use std::path::Path;

struct Dir {
    path: Path,
}

impl Dir {

    fn new(path: &String) -> Dir {
        Dir {
            path: Path::new(path)
        }
    }

}

#[hana_function()]
fn constructor(path: Value::Str) -> Value {
    let rec = vm.malloc(Record::new());
    rec.as_mut().native_field = Some(Dir::new(Path::new(path.as_ref())));
    Value::Record(rec)
}

#[hana_function()]
fn ls(path: Value::Record) -> Value {
    unimplemented!()
}