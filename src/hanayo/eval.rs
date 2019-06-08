//! Provides eval function for dynamically evaluating source code
use std::rc::Rc;
use std::cell::RefCell;
use crate::ast;
use crate::compiler::{Compiler, ModulesInfo};
use crate::vmbindings::value::Value;
use crate::vmbindings::vm::Vm;
use crate::vmbindings::vm::VmOpcode;

#[hana_function()]
fn eval(s: Value::Str) -> Value {
    let s = s.as_ref();
    if let Ok(prog) = ast::grammar::start(&s) {
        let target_ip = vm.code.as_ref().unwrap().len() as u32;
        let mut c = Compiler::new_append(vm.code.take().unwrap(), Rc::new(RefCell::new(ModulesInfo::new())));
        // generate code
        for stmt in prog {
            if stmt.emit(&mut c).is_err() {
                return Value::False;
            }
        }
        c.cpushop(VmOpcode::OP_HALT);
        vm.code = Some(c.into_code());
        // save current evaluation context
        let ctx = vm.new_exec_ctx();
        vm.jmp(target_ip);
        vm.execute();
        vm.restore_exec_ctx(ctx);
        return Value::True;
    }
    Value::False
}
