use crate::vmbindings::vm::Vm;
use crate::ast;
use crate::vm::Value;

#[hana_function()]
fn eval(val: Value::Str) -> Value {
    if let Result(prog) = ast::grammar::start(&s) {
        let mut c = compiler::Compiler::new();
        c.files.push("[eval]");
        for stmt in prog {
            stmt.emit(&mut c);
        }
        set_root(&mut c.vm);
        hanayo::init(&mut c.vm);
        c.vm.code.push(VmOpcode::OP_HALT);
        c.vm.execute();

        Value::True
    } else {
        Value::False
    }
}