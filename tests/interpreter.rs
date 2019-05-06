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

    // #region operators
    #[test]
    fn basic_arith() {
        let mut vm : Vm = eval!(vm, "y = 2*(3+5)");
        assert_eq!(vm.global().get("y").unwrap().unwrap(), Value::Int(16));
    }
    #[test]
    fn basic_cmp() {
        let mut vm : Vm = eval!(vm, "y = 1 > 0");
        assert_eq!(vm.global().get("y").unwrap().unwrap(), Value::Int(1));
    }
    // #endregion

    // #region if statement
    #[test]
    fn if_stmt() {
        let mut vm : Vm = eval!(vm, "
if 0 y = 1
");
        assert!(vm.global().get("y").is_none());
    }

    #[test]
    fn if_else_stmt() {
        let mut vm : Vm = eval!(vm, "
if 0 y = 1
else y = 2
");
        assert_eq!(vm.global().get("y").unwrap().unwrap(), Value::Int(2));
    }
    // #endregion

    // #region while statement
    #[test]
    fn while_stmt() {
        let mut vm : Vm = eval!(vm, "
i = 0
while i < 10 begin
i = i + 1
end
");
        assert_eq!(vm.global().get("i").unwrap().unwrap(), Value::Int(10));
    }
    // #endregion

    // #region for statement
    #[test]
    fn for_stmt() {
        let mut vm : Vm = eval!(vm, "
for i=0 to 10 begin
end
");
        assert_eq!(vm.global().get("i").unwrap().unwrap(), Value::Int(10));
    }

    #[test]
    fn for_downto_stmt() {
        let mut vm : Vm = eval!(vm, "
for i=10 downto 0 begin
end
");
        assert_eq!(vm.global().get("i").unwrap().unwrap(), Value::Int(0));
    }
    // #endregion

    // #region continue/break
    #[test]
    fn break_stmt() {
        let mut vm : Vm = eval!(vm, "
for i=0 to 10 begin
if i == 5 break
end
");
        assert_eq!(vm.global().get("i").unwrap().unwrap(), Value::Int(5));
    }
    // #endregion

    // #region functions
    #[test]
    fn function_stmt() {
        let mut vm : Vm = eval!(vm, "
function A() begin
end
");
        assert!(match vm.global().get("A").unwrap().unwrap() {
            Value::Fn => true,
            _ => false
        });
    }
    #[test]
    fn function_stmt_call() {
        let mut vm : Vm = eval!(vm, "
function A() begin
return 10
end
y = A()
");
        assert_eq!(vm.global().get("y").unwrap().unwrap(), Value::Int(10));
    }
    #[test]
    fn function_stmt_call_args() {
        let mut vm : Vm = eval!(vm, "
function A(x) begin
return 10+x
end
y = A(10)
");
        assert_eq!(vm.global().get("y").unwrap().unwrap(), Value::Int(20));
    }
    // #endregion

}