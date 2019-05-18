use std::collections::HashMap;
use super::function::Function;
use super::record::Record;
use super::value::Value;
use super::vm::Vm;
use super::env::Env;

pub struct ExFrame {
    handlers: HashMap<*const Record, Function>,
    pub unwind_env: *const Env, // rewind target
    pub unwind_stack: usize, // stack rewind target
    pub unwind_native_call_depth: usize, // how many native functions to return until we can call this?
}

impl ExFrame {

    pub fn new(unwind_env: *const Env, unwind_stack: usize, unwind_native_call_depth: usize) -> ExFrame {
        ExFrame {
            handlers: HashMap::new(),
            unwind_env: unwind_env,
            unwind_stack: unwind_stack,
            unwind_native_call_depth: unwind_native_call_depth
        }
    }

    pub fn set_handler(&mut self, rec: *const Record, fun: Function) {
        self.handlers.insert(rec, fun);
    }

    pub fn get_handler(&self, vm: *const Vm, val: &Value)
        -> Option<&Function> {
        let rec = val.get_prototype(vm);
        self.handlers.get(&rec)
    }

}