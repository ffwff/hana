extern crate haru;

#[cfg(test)]
pub mod interpreter_tests {

    use haru::ast::grammar;
    use haru::compiler;
    use haru::vm::Vm;
    use haru::vm::VmOpcode;
    use haru::vm::Value;

    macro_rules! eval {
        ($vm:expr, $str:expr) => {{
            let prog = grammar::start($str).unwrap();
            let mut c = compiler::Compiler::new();
            for stmt in prog {
                stmt.emit(&mut c);
            }
            c.vm.code.push(VmOpcode::OP_HALT);
            c.vm.execute();
            c.vm
        }};
    }

    // #region vars
    #[test]
    fn global_var() {
        let mut vm : Vm = eval!(vm, "y = 10");
        assert_eq!(vm.global().get("y").unwrap().unwrap(), Value::Int(10));
    }
    // #endregion

    // #region arith
    #[test]
    fn basic_arith() {
        let mut vm : Vm = eval!(vm, "y = 2*(3+5)");
        assert_eq!(vm.global().get("y").unwrap().unwrap(), Value::Int(16));
    }
    // #endregion

}