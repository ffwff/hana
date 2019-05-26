//! Provides eval function for dynamically evaluating source code
use crate::vmbindings::vm::Vm;
use crate::vmbindings::vm::VmOpcode;
use crate::ast;
use crate::vmbindings::value::Value;
use crate::compiler::Compiler;

#[hana_function()]
fn eval(s: Value::Str) -> Value {
     let s = s.as_ref();
    if let Ok(prog) = ast::grammar::start(&s) {
        let target_ip = vm.code.len() as u32;
        let mut c = unsafe{ Compiler::new_append_vm(vm) };
        // generate code
        for stmt in prog {
            stmt.emit(&mut c);
        }
        vm.code = c.deref_vm_code();
        vm.code.push(VmOpcode::OP_HALT);
        // save current evaluation context
        let ctx = vm.new_exec_ctx();
        vm.jmp(target_ip);
        vm.execute();
        vm.restore_exec_ctx(ctx);
        return Value::True;
    }
    Value::False
}