use crate::vmbindings::vm::Vm;
use crate::vmbindings::vm::VmOpcode;
use crate::ast;
use crate::vm::Value;
use crate::compiler::Compiler;
use std::cell::RefCell;
use std::borrow::Borrow;
use std::borrow::BorrowMut;

#[hana_function()]
fn eval(s: Value::Str) -> Value {
    if let Ok(prog) = ast::grammar::start(&s) {
        if let Some(_c) = &mut vm.compiler {
            let _crc = _c.upgrade().unwrap();
            let c : &mut Compiler = &mut *_crc.as_ref().borrow_mut();
            c.files.push("[eval]".to_string());
            for stmt in prog {
                stmt.emit(c);
            }
            c.vm.code.push(VmOpcode::OP_HALT);
            c.vm.execute();
            return Value::True;
        }
    }
    Value::False
}