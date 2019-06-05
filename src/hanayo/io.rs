//! Provides print, input and exit functions
use std::io::Write;

use crate::vmbindings::value::Value;
use crate::vmbindings::vm::Vm;

pub extern "C" fn print(cvm: *mut Vm, nargs: u16) {
    let vm = unsafe { &mut *cvm };
    for _ in 0..nargs {
        let val = vm.stack.pop().unwrap().unwrap();
        std::print!("{}", val);
    }
    std::io::stdout().flush().unwrap();
    vm.stack.push(Value::Nil.wrap());
}

#[hana_function()]
fn input() -> Value {
    let buffer = vm.malloc(String::new());
    std::io::stdin().read_line(buffer.as_mut()).unwrap();
    buffer.as_mut().pop(); // remove newline
    Value::Str(buffer)
}

#[hana_function()]
fn exit(code: Value::Int) -> Value {
    std::process::exit(code as i32);
}
