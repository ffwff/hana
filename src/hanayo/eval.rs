//! Provides eval function for dynamically evaluating source code
use crate::ast;
use crate::compiler::Compiler;
use crate::vmbindings::value::Value;
use crate::vmbindings::vm::Vm;
use crate::vmbindings::vm::VmOpcode;

#[hana_function()]
fn eval(s: Value::Str) -> Value {
    let s = s.as_ref();
    if let Ok(prog) = ast::grammar::start(&s) {
        let target_ip = vm.code.as_ref().unwrap().len() as u32;
        let mut c = Compiler::new_append(vm.code.take().unwrap());
        // generate code
        for stmt in prog {
            stmt.emit(&mut c);
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
