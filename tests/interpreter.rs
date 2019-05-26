extern crate haru;

#[cfg(test)]
pub mod interpreter_tests {

    use haru::ast::grammar;
    use haru::compiler;
    use haru::vmbindings::vm::{Vm, VmOpcode};
    use haru::vmbindings::value::Value;
    use haru::vmbindings::vmerror::VmError;

    use std::rc::Rc;

    macro_rules! eval {
        ($x:expr) => {{
            let prog = grammar::start($x).unwrap();
            let mut c = compiler::Compiler::new();
            for stmt in prog {
                stmt.emit(&mut c);
            }
            {
                let mut vm = c.vm.borrow_mut();
                vm.code.push(VmOpcode::OP_HALT);
                vm.gc_enable();
                vm.execute();
            }
            if let Ok(vm) = Rc::try_unwrap(c.vm) {
                vm.into_inner()
            } else {
                panic!("can't eval")
            }
        }};
    }

    // #region vars
    #[test]
    fn int_literal() {
        let vm : Vm = eval!("y = 10");
        assert_eq!(vm.global().get("y").unwrap().unwrap(), Value::Int(10));
    }

    #[test]
    fn float_literal() {
        let vm : Vm = eval!("y = 420.69");
        assert_eq!(vm.global().get("y").unwrap().unwrap(), Value::Float(420.69));
    }

    #[test]
    fn string_literal() {
        let vm : Vm = eval!("y = 'test'");
        assert_eq!(vm.global().get("y").unwrap().unwrap().string(), "test");
    }
    // #endregion

    // #region vars
    #[test]
    fn global_var() {
        let vm : Vm = eval!("y = 10");
        assert_eq!(vm.global().get("y").unwrap().unwrap(), Value::Int(10));
    }

    #[test]
    fn global_var_dollar() {
        let vm : Vm = eval!("$y = 10");
        assert_eq!(vm.global().get("y").unwrap().unwrap(), Value::Int(10));
    }
    // #endregion

    // #region operators
    #[test]
    fn basic_arith() {
        let vm : Vm = eval!("y = 2*(3+5)");
        assert_eq!(vm.global().get("y").unwrap().unwrap(), Value::Int(16));
    }

    #[test]
    fn cmp_gt() {
        let vm : Vm = eval!("y = 1 > 0");
        assert_eq!(vm.global().get("y").unwrap().unwrap(), Value::Int(1));
    }

    #[test]
    fn cmp_lt() {
        let vm : Vm = eval!("y = 1 < 0");
        assert_eq!(vm.global().get("y").unwrap().unwrap(), Value::Int(0));
    }

    #[test]
    fn cmp_gte() {
        let vm : Vm = eval!("y = 0 >= 0");
        assert_eq!(vm.global().get("y").unwrap().unwrap(), Value::Int(1));
    }

    #[test]
    fn cmp_lte() {
        let vm : Vm = eval!("y = 0 <= 0");
        assert_eq!(vm.global().get("y").unwrap().unwrap(), Value::Int(1));
    }

    #[test]
    fn cmp_eq() {
        let vm : Vm = eval!("y = 0 == 0");
        assert_eq!(vm.global().get("y").unwrap().unwrap(), Value::Int(1));
    }

    #[test]
    fn and_op() {
        let vm : Vm = eval!("y = 5 and 0");
        assert_eq!(vm.global().get("y").unwrap().unwrap(), Value::Int(0));
    }

    #[test]
    fn or_op() {
        let vm : Vm = eval!("y = 5 or 0");
        assert_eq!(vm.global().get("y").unwrap().unwrap(), Value::Int(5));
    }

    #[test]
    fn condexpr() {
        let vm : Vm = eval!("y = 1 ? 2*2 : 0");
        assert_eq!(vm.global().get("y").unwrap().unwrap(), Value::Int(4));
    }

    #[test]
    fn adds_not_in_place() {
        let vm : Vm = eval!("
a = 0
a += 1
");
        assert_eq!(vm.global().get("a").unwrap().unwrap().int(), 1);
    }

    #[test]
    fn adds_in_place() {
        let vm : Vm = eval!("
a = 'a'
a += 'b'
");
        assert_eq!(*vm.global().get("a").unwrap().unwrap().string(), "ab".to_string());
    }

    #[test]
    fn adds_indexed() {
        let vm : Vm = eval!("
a = ['b','c']
y = 'a'
y += a[0]
");
        assert_eq!(*vm.global().get("y").unwrap().unwrap().string(), "ab".to_string());
    }

    #[test]
    fn adds_to_array_indexed_in_place() {
        let vm : Vm = eval!("
a = ['a','c']
a[0] += 'b'
y = a[0]
");
        assert_eq!(*vm.global().get("y").unwrap().unwrap().string(), "ab".to_string());
    }

    #[test]
    fn adds_to_record_indexed_in_place() {
        let vm : Vm = eval!("
record a
    y = 'a'
end
a.y += 'b'
y = a.y
");
        assert_eq!(*vm.global().get("y").unwrap().unwrap().string(), "ab".to_string());
    }

    #[test]
    fn muls_in_place() {
        let vm : Vm = eval!("
x = 'a'
x *= 3
");
        assert_eq!(vm.global().get("x").unwrap().unwrap().string(), &"aaa".to_string());
    }
    // #endregion

    // #region if statement
    #[test]
    fn if_stmt() {
        let vm : Vm = eval!("
if 0 then y = 1
");
        assert!(vm.global().get("y").is_none());
    }

    #[test]
    fn if_else_stmt() {
        let vm : Vm = eval!("
if 0 then y = 1
else y = 2
");
        assert_eq!(vm.global().get("y").unwrap().unwrap(), Value::Int(2));
    }
    // #endregion

    // #region while statement
    #[test]
    fn while_stmt() {
        let vm : Vm = eval!("
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
        let vm : Vm = eval!("
for i=0 to 10 begin
end
");
        assert_eq!(vm.global().get("i").unwrap().unwrap(), Value::Int(10));
    }

    #[test]
    fn for_downto_stmt() {
        let vm : Vm = eval!("
for i=10 downto 0 begin
end
");
        assert_eq!(vm.global().get("i").unwrap().unwrap(), Value::Int(0));
    }

    #[test]
    fn for_in_stmt() {
        let vm : Vm = eval!("
for i in [1,2,3,10] begin
end
");
        assert_eq!(vm.global().get("i").unwrap().unwrap(), Value::Int(10));
    }

    #[test]
    fn for_in_stmt_empty() {
        let vm : Vm = eval!("
for i in [] begin
end
");
        assert!(vm.global().get("i").is_none());
        assert_eq!(vm.stack.len(), 0);
    }

    #[test]
    fn for_in_stmt_string() {
        let vm : Vm = eval!("
y = 0
for i in 'abcd' begin
    y += 1
end
");
        assert_eq!(vm.global().get("y").unwrap().unwrap(), Value::Int(4));
    }

    #[test]
    fn for_in_stmt_iterator() {
        let vm : Vm = eval!("
record x

    i = 1

    function next(self) begin
        self.i += 1
        if self.i == 10 begin
            self.stopped = 1
        end
        return self.i
    end

end

for i in x begin
end
");
        let rec = vm.global().get("x").unwrap().unwrap().record();
        assert_eq!(rec.get(&"stopped".to_string()).unwrap().unwrap().int(), 1);
        assert_eq!(rec.get(&"i".to_string()).unwrap().unwrap().int(), 10);
    }
    // #endregion

    // #region continue/break
    #[test]
    fn break_stmt() {
        let vm : Vm = eval!("
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
        let vm : Vm = eval!("
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
        let vm : Vm = eval!("
function A() begin
return 10
end
y = A()
");
        assert_eq!(vm.global().get("y").unwrap().unwrap(), Value::Int(10));
    }
    #[test]
    fn function_stmt_call_args() {
        let vm : Vm = eval!("
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
        let vm : Vm = eval!("
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
        let vm : Vm = eval!("
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
        let vm : Vm = eval!("
(function() begin
$y = 0
end)()
");
        assert_eq!(vm.global().get("y").unwrap().unwrap(), Value::Int(0));
    }
    #[test]
    fn function_expr() {
        let vm : Vm = eval!("
fib(n) = n <= 1 ? 1 : fib(n-1) + fib(n-2)
");
        assert!(match vm.global().get("fib").unwrap().unwrap() {
            Value::Fn(_) => true,
            _ => false
        });
    }
    #[test]
    fn function_return() {
        let vm : Vm = eval!("
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
        let vm : Vm = eval!("
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
        let vm : Vm = eval!("
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
        let vm : Vm = eval!("
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
        let vm : Vm = eval!("
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
        let vm : Vm = eval!("
record A
end
");
        vm.global().get("A").unwrap().unwrap().record();
    }

    #[test]
    fn record_stmt_with_body() {
        let vm : Vm = eval!("
record A
    y = 0
    function x() begin

    end
end
");
        let rec = vm.global().get("A").unwrap().unwrap().record();
        assert_eq!(rec.get(&"y".to_string()).unwrap().unwrap(), Value::Int(0));
        assert!(match rec.get(&"x".to_string()).unwrap().unwrap() {
            Value::Fn(_) => true,
            _ => false
        });
    }

    #[test]
    fn memexpr_get_unk() {
        let vm : Vm = eval!("
record A
end
y = A.x
");
        assert_eq!(vm.global().get("y").unwrap().unwrap(), Value::Nil);
    }

    #[test]
    fn memexpr_indexed_record() {
        let vm : Vm = eval!("
record A
    x = 10
end
y = A['x']
");
        assert_eq!(vm.global().get("y").unwrap().unwrap().int(), 10);
    }

    #[test]
    fn memexpr_set() {
        let vm : Vm = eval!("
record A
    y = 0
end
function x() begin
    A.y = 1
end
x()
");
        let rec = vm.global().get("A").unwrap().unwrap().record();
        assert_eq!(rec.get(&"y".to_string()).unwrap().unwrap(), Value::Int(1));
    }

    #[test]
    fn memexpr_adds() {
        let vm : Vm = eval!("
record A
    y = 0
end
function x() begin
    A.y += 1
end
x()
");
        let rec = vm.global().get("A").unwrap().unwrap().record();
        assert_eq!(rec.get(&"y".to_string()).unwrap().unwrap(), Value::Int(1));
    }

    #[test]
    fn memexpr_adds_ordering() {
        let vm : Vm = eval!("
record A
    y = 'a'
end
function x() begin
    A.y += 'b'
end
x()
");
        let rec = vm.global().get("A").unwrap().unwrap().record();
        assert_eq!(*rec.get(&"y".to_string()).unwrap().unwrap().string(), "ab".to_string());
    }

    #[test]
    fn record_stmt_constructor() {
        let vm : Vm = eval!("
record A
    function constructor(self) begin
        return self
    end
end

a = A()
");
        let a = vm.global().get("a").unwrap().unwrap().record();
        assert_eq!(*a.get(&"prototype".to_string()).unwrap(), *vm.global().get("A").unwrap());
    }

    #[test]
    fn record_stmt_prototype_method() {
        let vm : Vm = eval!("
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
        let vm : Vm = eval!("
a = []
");
        vm.global().get("a").unwrap().unwrap().array();
    }

    #[test]
    fn array_repeat() {
        let vm : Vm = eval!("
a = [1]*5
");
        let arr = vm.global().get("a").unwrap().unwrap().array();
        assert_eq!(arr.len(), 5);
    }

    #[test]
    fn array_multiple() {
        let vm : Vm = eval!("
a = ['a', 'b']
");
        let arr = vm.global().get("a").unwrap().unwrap().array();
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0].unwrap().string(), &"a".to_string());
        assert_eq!(arr[1].unwrap().string(), &"b".to_string());
    }

    #[test]
    fn array_index() {
        let vm : Vm = eval!("
a = ['a', 'b']
y = a[0]
");
        assert_eq!(vm.global().get("y").unwrap().unwrap().string(), &"a".to_string());
    }

    #[test]
    fn array_index_set() {
        let vm : Vm = eval!("
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
        let vm : Vm = eval!("
a = 'abcdef'
y = a[0]
");
        assert_eq!(vm.global().get("y").unwrap().unwrap().string(), &"a".to_string());
    }
    // #endregion

    // #region modules
    #[test]
    fn module_absolute_import() {
        std::fs::write("/tmp/module_absolute_import", "$y = 10").unwrap();
        let prog = grammar::start("
use '/tmp/module_absolute_import'
        ").unwrap();
        let mut c = compiler::Compiler::new();
        c.files.push("/tmp/x".to_string());
        c.vm.borrow_mut().compiler = Some(&mut c);
        for stmt in prog {
            stmt.emit(&mut c);
        }
        c.vm.borrow_mut().code.push(VmOpcode::OP_HALT);
        c.vm.borrow_mut().execute();
        assert_eq!(c.vm.borrow().global().get("y").unwrap().unwrap().int(), 10);
    }

    #[test]
    fn module_relative_import() {
        std::fs::write("/tmp/module_relative_import", "$y = 10").unwrap();
        let prog = grammar::start("
use './module_relative_import'
        ").unwrap();
        let mut c = compiler::Compiler::new();
        c.files.push("/tmp/x".to_string());
        c.vm.borrow_mut().compiler = Some(&mut c);
        for stmt in prog {
            stmt.emit(&mut c);
        }
        c.vm.borrow_mut().code.push(VmOpcode::OP_HALT);
        c.vm.borrow_mut().execute();
        assert_eq!(c.vm.borrow().global().get("y").unwrap().unwrap().int(), 10);
    }

    /*#[test] // FIXME: This doesn't work for some reason
    fn module_native_import() {
        std::fs::write("/tmp/module_native_import", "$y = 10").unwrap();
        std::env::set_var("HANA_PATH", "/tmp");
        let prog = grammar::start("
use 'module_native_import'
        ").unwrap();
        let mut c = compiler::Compiler::new();
        c.files.push("/tmp/x".to_string());
        c.vm.borrow_mut().compiler = Some(&mut c);
        for stmt in prog {
            stmt.emit(&mut c);
        }
        c.vm.borrow_mut().code.push(VmOpcode::OP_HALT);
        c.vm.borrow_mut().execute();
        assert_eq!(c.vm.borrow().global().get("y").unwrap().unwrap().int(), 10);
    }*/
    // #endregion
}