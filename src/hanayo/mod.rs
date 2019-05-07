mod io;
use crate::vmbindings::vm::Vm;
use crate::vmbindings::value::*;

pub fn init(vm : &mut Vm) {
    let globalenv = unsafe { &mut *vm.globalenv };
    globalenv.insert("print".to_string(), Value::NativeFn(io::print).wrap());
}