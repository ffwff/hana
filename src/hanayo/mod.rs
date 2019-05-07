mod io;
use crate::vmbindings::vm::Vm;
use crate::vmbindings::value::*;

pub fn init(vm : &mut Vm) {
    let globalenv = unsafe { &mut *vm.globalenv };
    globalenv.insert("true".to_string(), Value::Int(1).wrap());
    globalenv.insert("false".to_string(), Value::Int(0).wrap());
    globalenv.insert("print".to_string(), Value::NativeFn(io::print).wrap());
    globalenv.insert("input".to_string(), Value::NativeFn(io::input).wrap());
}