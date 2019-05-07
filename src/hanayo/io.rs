use std::io::Write;
use crate::vmbindings::vm::Vm;
use crate::vm::Value;

pub extern fn print(cvm : *mut Vm, nargs : u16) {
    let vm = unsafe { &mut *cvm };
    let val = vm.stack.top().unwrap();
    std::print!("{:?}", val);
    std::io::stdout().flush().unwrap();
    vm.stack.pop();
    vm.stack.push(Value::Int(10).wrap());
}