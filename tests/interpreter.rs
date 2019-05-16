extern crate haru;

#[cfg(test)]
pub mod interpreter_tests {

    use haru::ast::grammar;
    use haru::compiler;
    use haru::vm::{Vm, VmOpcode, Value};
    use haru::vmbindings::vmerror::VmError;
    use haru::gc;

    // TODO
    use haru::vmbindings::carray::CArray;

    macro_rules! eval {
        ($x:expr) => {{
            gc::disable();
            let prog = grammar::start($x).unwrap();
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
        let mut vm : Vm = eval!("y = 10");
        assert_eq!(vm.global().get("y").unwrap().unwrap(), Value::Int(10));
    }

    #[test]
    fn global_var_dollar() {
        let mut vm : Vm = eval!("$y = 10");
        assert_eq!(vm.global().get("y").unwrap().unwrap(), Value::Int(10));
    }
    // #endregion

    // #region operators
    #[test]
    fn basic_arith() {
        let mut vm : Vm = eval!("y = 2*(3+5)");
        assert_eq!(vm.global().get("y").unwrap().unwrap(), Value::Int(16));
    }
    #[test]
    fn basic_cmp() {
        let mut vm : Vm = eval!("y = 1 > 0");
        assert_eq!(vm.global().get("y").unwrap().unwrap(), Value::Int(1));
    }

    #[test]
    fn adds_ordering() {
        let mut vm : Vm = eval!("
a = 'a'
a += 'b'
");
        assert_eq!(*vm.global().get("a").unwrap().unwrap().string(), "ab".to_string());
    }
    // #endregion

    // #region if statement
    #[test]
    fn if_stmt() {
        let mut vm : Vm = eval!("
if 0 then y = 1
");
        assert!(vm.global().get("y").is_none());
    }

    #[test]
    fn if_else_stmt() {
        let mut vm : Vm = eval!("
if 0 then y = 1
else y = 2
");
        assert_eq!(vm.global().get("y").unwrap().unwrap(), Value::Int(2));
    }
    // #endregion

    // #region while statement
    #[test]
    fn while_stmt() {
        let mut vm : Vm = eval!("
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
        let mut vm : Vm = eval!("
for i=0 to 10 begin
end
");
        assert_eq!(vm.global().get("i").unwrap().unwrap(), Value::Int(10));
    }

    #[test]
    fn for_downto_stmt() {
        let mut vm : Vm = eval!("
for i=10 downto 0 begin
end
");
        assert_eq!(vm.global().get("i").unwrap().unwrap(), Value::Int(0));
    }

    #[test]
    fn for_in_stmt() {
        let mut vm : Vm = eval!("
for i in [1,2,3,10] begin
end
");
        assert_eq!(vm.global().get("i").unwrap().unwrap(), Value::Int(10));
    }
    // #endregion

    // #region continue/break
    #[test]
    fn break_stmt() {
        let mut vm : Vm = eval!("
for i=0 to 10 begin
if i == 5 then break
end
");
        assert_eq!(vm.global().get("i").unwrap().unwrap(), Value::Int(5));
    }
    // #endregion

    // #region functions
    #[test]
    fn function_stmt() {
        let mut vm : Vm = eval!("
function A() begin
end
");
        assert!(match vm.global().get("A").unwrap().unwrap() {
            Value::Fn(_) => true,
            _ => false
        });
    }
    #[test]
    fn function_stmt_call() {
        let mut vm : Vm = eval!("
function A() begin
return 10
end
y = A()
");
        assert_eq!(vm.global().get("y").unwrap().unwrap(), Value::Int(10));
    }
    #[test]
    fn function_stmt_call_args() {
        let mut vm : Vm = eval!("
function A(x) begin
return 10+x
end
y = A(10)
");
        assert_eq!(vm.global().get("y").unwrap().unwrap(), Value::Int(20));
        assert!(vm.global().get("x").is_none());
    }
    #[test]
    fn function_stmt_scope() {
        let mut vm : Vm = eval!("
$x = 1
function outer() begin
    x = 2
    function inner() begin
        x = 3
        $z = x
    end
    inner()
    $y = x
end
outer()
");
        assert_eq!(vm.global().get("x").unwrap().unwrap(), Value::Int(1));
        assert_eq!(vm.global().get("y").unwrap().unwrap(), Value::Int(2));
        assert_eq!(vm.global().get("z").unwrap().unwrap(), Value::Int(3));
    }
    #[test]
    fn function_stmt_scope_up() {
        let mut vm : Vm = eval!("
(function() begin

    a = 10
    function A() begin
        function B() begin
            $x = a
        end
        B()
    end
    A()

end)()
");
        assert_eq!(vm.global().get("x").unwrap().unwrap(), Value::Int(10));
    }
    #[test]
    fn function_stmt_iife() {
        let mut vm : Vm = eval!("
(function() begin
$y = 0
end)()
");
        assert_eq!(vm.global().get("y").unwrap().unwrap(), Value::Int(0));
    }
    #[test]
    fn function_expr() {
        let mut vm : Vm = eval!("
fib(n) = n <= 1 ? 1 : fib(n-1) + fib(n-2)
");
        assert!(match vm.global().get("fib").unwrap().unwrap() {
            Value::Fn(_) => true,
            _ => false
        });
    }
    #[test]
    fn function_return() {
        let mut vm : Vm = eval!("
function a() begin
    return 1
    $y = 0
end

y = a()
");
        assert_eq!(vm.global().get("y").unwrap().unwrap(), Value::Int(1));
    }
    #[test]
    fn function_tco() {
        let mut vm : Vm = eval!("
y = 0
function a() begin
    if $y == 1000 then return
    $y += 1
    return a()
end
a()
");
        assert_eq!(vm.global().get("y").unwrap().unwrap(), Value::Int(1000));
    }
    #[test]
    fn function_tco_short() {
        let mut vm : Vm = eval!("
a(x) = x == 1000 ? x : a(x+1)
y = a(0)
");
        assert_eq!(vm.global().get("y").unwrap().unwrap(), Value::Int(1000));
    }
    /* // TODO: this test won't work because vm halts
    #[test]
    fn function_call_from_native() {
        let mut vm : Vm = eval!("
function a() begin
    return 10
end
");
        let val = vm.global().get("a").unwrap().clone();
        assert_eq!(vm.call(val, CArray::new_nil()).unwrap().unwrap(), Value::Int(10));
    } */
    // #endregion

    // #region exceptions
    #[test]
    fn try_stmt_simple() {
        let mut vm : Vm = eval!("
try
    y = 10
end
");
        assert_eq!(vm.global().get("y").unwrap().unwrap(), Value::Int(10));
    }

    #[test]
    fn try_stmt_unhandled_raise() {
        let vm : Vm = eval!("
raise 0
");
        assert_eq!(vm.error, VmError::ERROR_UNHANDLED_EXCEPTION);
    }

    #[test]
    fn try_stmt_handled_raise() {
        let mut vm : Vm = eval!("
record A
    function constructor(self) begin
        return self
    end
end
try
    raise A()
case A
    y = 10
end
");
        assert_eq!(vm.global().get("y").unwrap().unwrap(), Value::Int(10));
    }
    // #endregion

    // #region record
    #[test]
    fn record_stmt_simple() {
        let mut vm : Vm = eval!("
record A
end
");
        assert!(match vm.global().get("A").unwrap().unwrap() {
            Value::Record(_) => true,
            _ => false
        });
    }

    #[test]
    fn record_stmt_with_body() {
        let mut vm : Vm = eval!("
record A
    y = 0
    function x() begin

    end
end
");
        let rec = match vm.global().get("A").unwrap().unwrap() {
            Value::Record(x) => x,
            _ => panic!("expected record")
        };
        assert_eq!(rec.get(&"y".to_string()).unwrap().unwrap(), Value::Int(0));
        assert!(match rec.get(&"x".to_string()).unwrap().unwrap() {
            Value::Fn(_) => true,
            _ => false
        });
    }

    #[test]
    fn memexpr_set() {
        let mut vm : Vm = eval!("
record A
    y = 0
end
function x() begin
    A.y = 1
end
x()
");
        let rec = match vm.global().get("A").unwrap().unwrap() {
            Value::Record(x) => x,
            _ => panic!("expected record")
        };
        assert_eq!(rec.get(&"y".to_string()).unwrap().unwrap(), Value::Int(1));
    }

    #[test]
    fn memexpr_adds() {
        let mut vm : Vm = eval!("
record A
    y = 0
end
function x() begin
    A.y += 1
end
x()
");
        let rec = match vm.global().get("A").unwrap().unwrap() {
            Value::Record(x) => x,
            _ => panic!("expected record")
        };
        assert_eq!(rec.get(&"y".to_string()).unwrap().unwrap(), Value::Int(1));
    }

    #[test]
    fn memexpr_adds_ordering() {
        let mut vm : Vm = eval!("
record A
    y = 'a'
end
function x() begin
    A.y += 'b'
end
x()
");
        let rec = match vm.global().get("A").unwrap().unwrap() {
            Value::Record(x) => x,
            _ => panic!("expected record")
        };
        assert_eq!(*rec.get(&"y".to_string()).unwrap().unwrap().string(), "ab".to_string());
    }

    #[test]
    fn record_stmt_constructor() {
        let mut vm : Vm = eval!("
record A
    function constructor(self) begin
        return self
    end
end

a = A()
");
        let a = match vm.global().get("a").unwrap().unwrap() {
            Value::Record(x) => x,
            _ => panic!("expected record")
        };
        assert_eq!(*a.get(&"prototype".to_string()).unwrap(), *vm.global().get("A").unwrap());
    }

    #[test]
    fn record_stmt_prototype_method() {
        let mut vm : Vm = eval!("
record A
    function constructor(self) begin
        return self
    end

    function test(self) begin
        return 10
    end
end

a = A()
y = a.test()
");
        assert_eq!(vm.global().get("y").unwrap().unwrap(), Value::Int(10));
    }
    // #endregion

    // #region array
    #[test]
    fn array_simple() {
        let mut vm : Vm = eval!("
a = []
");
        assert!(match vm.global().get("a").unwrap().unwrap() {
            Value::Array(_) => true,
            _ => false
        });
    }

    #[test]
    fn array_multiple() {
        let mut vm : Vm = eval!("
a = ['a', 'b']
");
        let arr = match vm.global().get("a").unwrap().unwrap() {
            Value::Array(x) => x,
            _ => panic!("expected array")
        };
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0].unwrap().string(), &"a".to_string());
        assert_eq!(arr[1].unwrap().string(), &"b".to_string());
    }

    #[test]
    fn array_index() {
        let mut vm : Vm = eval!("
a = ['a', 'b']
y = a[0]
");
        assert_eq!(vm.global().get("y").unwrap().unwrap().string(), &"a".to_string());
    }

    #[test]
    fn array_index_set() {
        let mut vm : Vm = eval!("
a = ['a', 'b']
a[0] = 'x'
y = a[0]
");
        assert_eq!(vm.global().get("y").unwrap().unwrap().string(), &"x".to_string());
    }
    // #endregion

    // #region string
    #[test]
    fn string_index() {
        let mut vm : Vm = eval!("
a = 'abcdef'
y = a[0]
");
        assert_eq!(vm.global().get("y").unwrap().unwrap().string(), &"a".to_string());
    }
    // #endregion
}