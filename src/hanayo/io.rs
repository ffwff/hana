use std::io::Write;

use crate::vmbindings::vm::Vm;
use crate::vm::Value;

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
    let mut buffer = String::new();
    std::io::stdin().read_line(&mut buffer).unwrap();
    buffer.pop(); // remove newline
    unsafe { Value::Str(&*Box::into_raw(Box::new(buffer))) }
}