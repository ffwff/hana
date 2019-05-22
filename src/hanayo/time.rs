use std::time::*;
use crate::vmbindings::vm::Vm;
use crate::vm::Value;
use crate::vmbindings::record::Record;
use super::Gc;

fn duration_to_record(vm: &mut Vm, duration: Duration) -> Value {
    let rec = Gc::new(Record::new());
    rec.as_mut().native_field = Some(Box::new(duration));
    // TODO: maybe not hardcode prototype it like this
    rec.as_mut().insert("prototype".to_string(), *vm.global().get(&"Time".to_string()).unwrap());
    Value::Record(rec)
}

#[hana_function()]
fn constructor() -> Value {
    duration_to_record(vm, SystemTime::now().duration_since(UNIX_EPOCH).unwrap())
}

// since
#[hana_function()]
fn since(left: Value::Record, right: Value::Record) -> Value {
    let lfield = left.as_ref().native_field.as_ref().unwrap();
    let left_duration = lfield.downcast_ref::<Duration>().unwrap();
    let rfield = right.as_ref().native_field.as_ref().unwrap();
    let right_duration = rfield.downcast_ref::<Duration>().unwrap();
    duration_to_record(vm, left_duration.clone()
            .checked_sub(right_duration.clone()).unwrap())
}

// accessors
#[hana_function()]
fn secs(time: Value::Record) -> Value {
    let tref = time.as_ref().native_field.as_ref().unwrap();
    let time = tref.downcast_ref::<Duration>().unwrap();
    Value::Int(time.as_secs() as i64)
}
#[hana_function()]
fn millis(time: Value::Record) -> Value {
    let tref = time.as_ref().native_field.as_ref().unwrap();
    let time = tref.downcast_ref::<Duration>().unwrap();
    Value::Int(time.as_millis() as i64)
}
#[hana_function()]
fn micros(time: Value::Record) -> Value {
    let tref = time.as_ref().native_field.as_ref().unwrap();
    let time = tref.downcast_ref::<Duration>().unwrap();
    Value::Int(time.as_micros() as i64)
}
#[hana_function()]
fn nanos(time: Value::Record) -> Value {
    let tref = time.as_ref().native_field.as_ref().unwrap();
    let time = tref.downcast_ref::<Duration>().unwrap();
    Value::Int(time.as_nanos() as i64)
}