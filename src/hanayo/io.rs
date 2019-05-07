use crate::vmbindings::vm::Vm;
use crate::vm::Value;
use std::io::Write;

pub extern fn print(cvm : *mut Vm, nargs : u16) {
    let vm = unsafe { &mut *cvm };
    for _ in 0..nargs {
        let val = vm.stack.top().unwrap();
        std::print!("{:?}", val);
        std::io::stdout().flush().unwrap();
        vm.stack.pop();
    }
    vm.stack.push(Value::Nil.wrap());
}

#[hana_function()]
fn input() -> Value {
    // TODO
    Value::Nil
}