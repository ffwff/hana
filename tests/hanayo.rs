extern crate haru;

#[cfg(test)]
pub mod hanayo_tests {

    use haru::ast::grammar;
    use haru::compiler;
    use haru::vmbindings::value::Value;
    use haru::vmbindings::vm::{Vm, VmOpcode};
    use haru::hanayo;

    macro_rules! eval {
        ($x:expr) => {{
            let prog = grammar::start($x).unwrap();
            let mut c = compiler::Compiler::new();
            hanayo::init(&mut c.vm);
            for stmt in prog {
                stmt.emit(&mut c);
            }
            c.vm.code.push(VmOpcode::OP_HALT);
            c.vm.gc_enable();
            c.execute();
            c.vm
        }};
    }

    // #region vm extensions
    #[test]
    fn of_expr_simple() {
        let vm: Vm = eval!(
            "
y = 1 of Int
"
        );
        assert_eq!(vm.global().get("y").unwrap().unwrap().int(), 1);
    }

    #[test]
    fn of_expr_record() {
        let vm: Vm = eval!(
            "
record Y
end
record X
    prototype = Y
end
y = X of Y
"
        );
        assert_eq!(vm.global().get("y").unwrap().unwrap().int(), 1);
    }

    #[test]
    fn literal_prototype() {
        let vm: Vm = eval!(
            "
y = (10).prototype == Int
"
        );
        assert_eq!(vm.global().get("y").unwrap().unwrap().int(), 1);
    }
    // #endregion

    // #region int
    #[test]
    fn int_constructor() {
        let vm: Vm = eval!(
            "
y = Int(1)
"
        );
        assert_eq!(vm.global().get("y").unwrap().unwrap().int(), 1);
    }

    #[test]
    fn int_constructor_float() {
        let vm: Vm = eval!(
            "
y = Int(1.2)
"
        );
        assert_eq!(vm.global().get("y").unwrap().unwrap().int(), 1);
    }

    #[test]
    fn int_constructor_str() {
        let vm: Vm = eval!(
            "
y = Int('10')
"
        );
        assert_eq!(vm.global().get("y").unwrap().unwrap().int(), 10);
    }

    #[test]
    fn int_chr() {
        let vm: Vm = eval!(
            "
y = (97).chr()
"
        );
        assert_eq!(vm.global().get("y").unwrap().unwrap().string(), "a");
    }

    #[test]
    fn int_hex() {
        let vm: Vm = eval!(
            "
y = (16).hex()
"
        );
        assert_eq!(vm.global().get("y").unwrap().unwrap().string(), "0x10");
    }
    // #end

    // #region float
    #[test]
    fn float_constructor() {
        let vm: Vm = eval!(
            "
y = Float(1.0)
"
        );
        assert_eq!(vm.global().get("y").unwrap().unwrap().float(), 1.0);
    }

    #[test]
    fn float_constructor_int() {
        let vm: Vm = eval!(
            "
y = Float(1)
"
        );
        assert_eq!(vm.global().get("y").unwrap().unwrap().float(), 1.0);
    }

    #[test]
    fn float_constructor_str() {
        let vm: Vm = eval!(
            "
y = Float('10.55')
"
        );
        assert_eq!(vm.global().get("y").unwrap().unwrap().float(), 10.55);
    }
    // #end

    // #region array
    #[test]
    fn array_constructor_no_args() {
        let vm: Vm = eval!(
            "
y = Array()
"
        );
        let arr = vm.global().get("y").unwrap().unwrap().array();
        assert_eq!(arr.len(), 0);
    }

    #[test]
    fn array_constructor() {
        let vm: Vm = eval!(
            "
y = Array(1,2,3)
"
        );
        let arr = vm.global().get("y").unwrap().unwrap().array();
        assert_eq!(arr.len(), 3);
        assert_eq!(arr[0].unwrap(), Value::Int(1));
        assert_eq!(arr[1].unwrap(), Value::Int(2));
        assert_eq!(arr[2].unwrap(), Value::Int(3));
    }

    #[test]
    fn array_length() {
        let vm: Vm = eval!(
            "
y = [1,2,3].length()
"
        );
        assert_eq!(vm.global().get("y").unwrap().unwrap().int(), 3);
    }

    #[test]
    fn array_delete() {
        let vm: Vm = eval!(
            "
y = [1,2,3]
y.delete!(1,1)
"
        );
        let arr = vm.global().get("y").unwrap().unwrap().array();
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0].unwrap(), Value::Int(1));
        assert_eq!(arr[1].unwrap(), Value::Int(3));
    }

    #[test]
    fn array_push() {
        let vm: Vm = eval!(
            "
y = []
y.push(10)
"
        );
        let arr = vm.global().get("y").unwrap().unwrap().array();
        assert_eq!(arr.len(), 1);
        assert_eq!(arr[0].unwrap(), Value::Int(10));
    }

    #[test]
    fn array_pop() {
        let vm: Vm = eval!(
            "
y = [1,2]
y.pop()
"
        );
        let arr = vm.global().get("y").unwrap().unwrap().array();
        assert_eq!(arr.len(), 1);
        assert_eq!(arr[0].unwrap(), Value::Int(1));
    }

    #[test]
    fn array_index() {
        let vm: Vm = eval!(
            "
a = ['a', 'b', 'c']
y = a.index('b')
"
        );
        assert_eq!(vm.global().get("y").unwrap().unwrap(), Value::Int(1));
    }

    #[test]
    fn array_insert() {
        let vm: Vm = eval!(
            "
y = [1,2,3]
y.insert!(1, 4)
"
        );
        let arr = vm.global().get("y").unwrap().unwrap().array();
        assert_eq!(arr.len(), 4);
        assert_eq!(arr[0].unwrap(), Value::Int(1));
        assert_eq!(arr[1].unwrap(), Value::Int(4));
        assert_eq!(arr[2].unwrap(), Value::Int(2));
        assert_eq!(arr[3].unwrap(), Value::Int(3));
    }

    #[test]
    fn array_sort_in_place() {
        let vm: Vm = eval!(
            "
y = [6,3,1]
y.sort!()
"
        );
        let arr = vm.global().get("y").unwrap().unwrap().array();
        assert_eq!(arr.len(), 3);
        assert_eq!(arr[0].unwrap(), Value::Int(1));
        assert_eq!(arr[1].unwrap(), Value::Int(3));
        assert_eq!(arr[2].unwrap(), Value::Int(6));
    }
    #[test]
    fn array_sort() {
        let vm: Vm = eval!(
            "
x = [6,3,1]
y = x.sort()
"
        );
        let arr = vm.global().get("y").unwrap().unwrap().array();
        assert_eq!(arr.len(), 3);
        assert_eq!(arr[0].unwrap(), Value::Int(1));
        assert_eq!(arr[1].unwrap(), Value::Int(3));
        assert_eq!(arr[2].unwrap(), Value::Int(6));
    }

    #[test]
    fn array_map() {
        let vm: Vm = eval!(
            "
a=[3,5,64,2]
y = a.map(f(x) = x+1)
"
        );
        let arr = vm.global().get("y").unwrap().unwrap().array();
        assert_eq!(arr.len(), 4);
        assert_eq!(arr[0].unwrap(), Value::Int(4));
        assert_eq!(arr[1].unwrap(), Value::Int(6));
        assert_eq!(arr[2].unwrap(), Value::Int(65));
        assert_eq!(arr[3].unwrap(), Value::Int(3));
    }

    #[test]
    fn array_map_native() {
        let vm: Vm = eval!(
            "
a=['1','2','3']
y = a.map(Int)
"
        );
        let arr = vm.global().get("y").unwrap().unwrap().array();
        assert_eq!(arr.len(), 3);
        assert_eq!(arr[0].unwrap(), Value::Int(1));
        assert_eq!(arr[1].unwrap(), Value::Int(2));
        assert_eq!(arr[2].unwrap(), Value::Int(3));
    }

    #[test]
    fn array_filter() {
        let vm: Vm = eval!(
            "
a=[3,5,64,2]
y = a.filter(f(x) = x>5)
"
        );
        let arr = vm.global().get("y").unwrap().unwrap().array();
        assert_eq!(arr.len(), 1);
        assert_eq!(arr[0].unwrap(), Value::Int(64));
    }

    #[test]
    fn array_reduce() {
        let vm: Vm = eval!(
            "
a=[1,2,3,4,5]
y = a.reduce(f(x, y) = x + y, 0)
"
        );
        assert_eq!(vm.global().get("y").unwrap().unwrap(), Value::Int(15));
    }

    #[test]
    fn array_chained_functional() {
        let vm: Vm = eval!(
            "
a=[1,2,3,5,6]
y = a.map(f(x) = x+1).filter(f(x) = x>5).reduce(f(prev, curr) = prev+curr, 0)
"
        );
        assert_eq!(vm.global().get("y").unwrap().unwrap(), Value::Int(13));
    }

    #[test]
    fn array_join() {
        let vm: Vm = eval!(
            "
a=[1,2,3,4,5,6]
y = a.join('')
"
        );
        assert_eq!(vm.global().get("y").unwrap().unwrap().string(), "123456");
    }
    // #endregion

    // #region string
    #[test]
    fn string_constructor_no_args() {
        let vm: Vm = eval!(
            "
y = String()
"
        );
        assert_eq!(vm.global().get("y").unwrap().unwrap().string(), "");
    }

    #[test]
    fn string_constructor() {
        let vm: Vm = eval!(
            "
y = String(10)
"
        );
        assert_eq!(vm.global().get("y").unwrap().unwrap().string(), "10");
    }

    #[test]
    fn string_length() {
        let vm: Vm = eval!(
            "
y = '日本'.length()
"
        );
        assert_eq!(vm.global().get("y").unwrap().unwrap().int(), 2);
    }

    #[test]
    fn string_bytesize() {
        let vm: Vm = eval!(
            "
y = '日本'.bytesize()
"
        );
        assert_eq!(vm.global().get("y").unwrap().unwrap().int(), 6);
    }

    #[test]
    fn string_startswith() {
        let vm: Vm = eval!(
            "
y = 'abc'.startswith?('a')
"
        );
        assert_eq!(vm.global().get("y").unwrap().unwrap().int(), 1);
    }

    #[test]
    fn string_endswith() {
        let vm: Vm = eval!(
            "
y = 'abc'.endswith?('bc')
"
        );
        assert_eq!(vm.global().get("y").unwrap().unwrap().int(), 1);
    }

    #[test]
    fn string_delete() {
        let vm: Vm = eval!(
            "
s = 'λκj'
y = s.delete(1,1)
"
        );
        assert_eq!(vm.global().get("y").unwrap().unwrap().string(), "λj");
    }
    #[test]
    fn string_delete_in_place() {
        let vm: Vm = eval!(
            "
s = 'λκj'
s.delete!(1,1)
"
        );
        assert_eq!(vm.global().get("s").unwrap().unwrap().string(), "λj");
    }

    #[test]
    fn string_copy() {
        let vm: Vm = eval!(
            "
s = 'λκj'
y = s.copy(1,2)
"
        );
        assert_eq!(vm.global().get("y").unwrap().unwrap().string(), "κj");
    }

    #[test]
    fn string_index() {
        let vm: Vm = eval!(
            "
s = 'λκj'
y = s.index('κ')
"
        );
        assert_eq!(vm.global().get("y").unwrap().unwrap(), Value::Int(1));
    }

    #[test]
    fn string_insert() {
        let vm: Vm = eval!(
            "
s = 'λκj'
s.insert!(1,'a')
"
        );
        assert_eq!(vm.global().get("s").unwrap().unwrap().string(), "λaκj");
    }

    #[test]
    fn string_split() {
        let vm: Vm = eval!(
            "
s = 'a b c'
y = s.split(' ')
"
        );
        let arr = vm.global().get("y").unwrap().unwrap().array();
        assert_eq!(arr.len(), 3);
        assert_eq!(arr[0].unwrap().string(), "a");
        assert_eq!(arr[1].unwrap().string(), "b");
        assert_eq!(arr[2].unwrap().string(), "c");
    }

    #[test]
    fn string_chars() {
        let vm: Vm = eval!(
            "
s = 'λκj'
y = s.chars()
"
        );
        let arr = vm.global().get("y").unwrap().unwrap().array();
        assert_eq!(arr.len(), 3);
        assert_eq!(arr[0].unwrap().string(), "λ");
        assert_eq!(arr[1].unwrap().string(), "κ");
        assert_eq!(arr[2].unwrap().string(), "j");
    }

    #[test]
    fn string_ord() {
        let vm: Vm = eval!(
            "
s = 'a'
y = s.ord()
"
        );
        assert_eq!(vm.global().get("y").unwrap().unwrap().int(), 97);
    }
    // #endregion

    // #region record
    #[test]
    fn record_new() {
        let vm: Vm = eval!(
            "
y = Record()
"
        );
        vm.global().get("y").unwrap().unwrap().record();
    }

    #[test]
    fn record_keys() {
        let vm: Vm = eval!(
            "
record x
    a = 10
    b = 10
end
y = Record::keys(x)
"
        );
        let arr = vm.global().get("y").unwrap().unwrap().array();
        assert_eq!(arr.len(), 2);
    }
    // #endregion

    // #region env
    #[test]
    fn env_get() {
        std::env::set_var("test_key", "value");
        let vm: Vm = eval!(
            "
y = Env::get('test_key')
"
        );
        assert_eq!(vm.global().get("y").unwrap().unwrap().string(), "value");
    }

    #[test]
    fn env_set() {
        eval!(
            "
Env::set('test_key_set', 'value')
"
        );
        assert_eq!(std::env::var("test_key_set").unwrap(), "value");
    }

    #[test]
    fn env_vars() {
        std::env::set_var("a_key", "value");
        let vm: Vm = eval!(
            "
y = Env::vars()['a_key']
"
        );
        assert_eq!(vm.global().get("y").unwrap().unwrap().string(), "value");
    }
    // #endregion

    // #region files
    #[test]
    fn file_read() {
        std::fs::write("/tmp/file_read", "test");
        let vm: Vm = eval!(
            "
f = File('/tmp/file_read', 'r')
y = f.read()
f.close()
"
        );
        assert_eq!(vm.global().get("y").unwrap().unwrap().string(), "test");
    }

    #[test]
    fn file_read_up_to() {
        std::fs::write("/tmp/file_read_up_to", "test");
        let vm: Vm = eval!(
            "
f = File('/tmp/file_read_up_to', 'r')
y = f.read_up_to(2)
"
        );
        assert_eq!(vm.global().get("y").unwrap().unwrap().string(), "te");
    }

    #[test]
    fn file_write() {
        eval!(
            "
f = File('/tmp/file_write', 'wc')
f.write('Hello World')
f.close()
"
        );
        assert_eq!(
            std::str::from_utf8(&std::fs::read("/tmp/file_write").unwrap()).unwrap(),
            "Hello World"
        );
    }
    // #endregion

    // #region cmd
    #[test]
    fn cmd_constructor_array() {
        let vm: Vm = eval!(
            "
y = Cmd(['echo', 'hello world']).out()
"
        );
        assert_eq!(
            vm.global().get("y").unwrap().unwrap().string(),
            "hello world\n"
        );
    }

    #[test]
    fn cmd_constructor_string() {
        let vm: Vm = eval!(
            "
y = Cmd('echo hello world').out()
"
        );
        assert_eq!(
            vm.global().get("y").unwrap().unwrap().string(),
            "hello world\n"
        );
    }

    #[test]
    fn cmd_err() {
        let vm: Vm = eval!(
            "
y = Cmd('echo hello world >&2').err()
"
        );
        assert_eq!(
            vm.global().get("y").unwrap().unwrap().string(),
            "hello world\n"
        );
    }

    #[test]
    fn cmd_in() {
        let vm: Vm = eval!(
            "
y = Cmd('cat -').in('nyaaa').out()
"
        );
        assert_eq!(vm.global().get("y").unwrap().unwrap().string(), "nyaaa");
    }

    #[test]
    fn cmd_outputs() {
        let vm: Vm = eval!(
            "
y = Cmd('echo hello world').outputs()
"
        );
        let arr = vm.global().get("y").unwrap().unwrap().array();
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0].unwrap().string(), "hello world\n");
        assert_eq!(arr[1].unwrap().string(), "");
    }
    // #endregion

    // #region proc
    #[test]
    fn proc_err() {
        let vm: Vm = eval!(
            "
y = Cmd('echo hello world >&2').spawn().err()
"
        );
        assert_eq!(
            vm.global().get("y").unwrap().unwrap().string(),
            "hello world\n"
        );
    }

    #[test]
    fn proc_in() {
        let vm: Vm = eval!(
            "
y = Cmd('cat -').spawn().in('nyaaa').out()
"
        );
        assert_eq!(vm.global().get("y").unwrap().unwrap().string(), "nyaaa");
    }

    #[test]
    fn proc_outputs() {
        let vm: Vm = eval!(
            "
y = Cmd('echo hello world').spawn().outputs()
"
        );
        let arr = vm.global().get("y").unwrap().unwrap().array();
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0].unwrap().string(), "hello world\n");
        assert_eq!(arr[1].unwrap().string(), "");
    }

    #[test]
    fn proc_sleep() {
        let _vm: Vm = eval!(
            "
y = Cmd('sleep 1s').spawn().wait()
"
        );
    }

    #[test]
    fn proc_kill() {
        let _vm: Vm = eval!(
            "
y = Cmd('sleep 1s').spawn().kill()
"
        );
    }
    // #endregion

    // #region math
    #[test]
    fn math_sqrt() {
        let vm: Vm = eval!(
            "
y = sqrt(4.0)
"
        );
        assert_eq!(vm.global().get("y").unwrap().unwrap(), Value::Float(2.0));
    }
    // #endregion

    // #region other
    #[test]
    fn eval() {
        let vm: Vm = eval!(
            "
eval('y = 10')
"
        );
        assert_eq!(vm.global().get("y").unwrap().unwrap(), Value::Int(10));
    }
    // #endregion

}
