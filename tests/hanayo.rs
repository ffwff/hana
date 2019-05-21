extern crate haru;

#[cfg(test)]
pub mod hanayo_tests {

    use haru::ast::grammar;
    use haru::compiler;
    use haru::vm::Vm;
    use haru::vm::VmOpcode;
    use haru::vm::Value;
    use haru::gc;
    use haru::hanayo;

    macro_rules! eval {
        ($x:expr) => {{
            let prog = grammar::start($x).unwrap();
            let mut c = compiler::Compiler::new();
            gc::set_root(&mut c.vm);
            hanayo::init(&mut c.vm);
            for stmt in prog {
                stmt.emit(&mut c);
            }
            c.vm.code.push(VmOpcode::OP_HALT);
            c.vm.execute();
            c.vm
        }};
    }

    // #region vm extensions
    #[test]
    fn of_expr_simple() {
        let mut vm : Vm = eval!("
y = 1 of Int
");
        assert_eq!(vm.global().get("y").unwrap().unwrap().int(), 1);
    }

    #[test]
    fn of_expr_record() {
        let mut vm : Vm = eval!("
record Y
end
record X
    prototype = Y
end
y = X of Y
");
        assert_eq!(vm.global().get("y").unwrap().unwrap().int(), 1);
    }

    #[test]
    fn literal_prototype() {
        let mut vm : Vm = eval!("
y = (10).prototype == Int
");
        assert_eq!(vm.global().get("y").unwrap().unwrap().int(), 1);
    }
    // #endregion

    // #region int
    #[test]
    fn int_constructor() {
        let mut vm : Vm = eval!("
y = Int(1)
");
        assert_eq!(vm.global().get("y").unwrap().unwrap().int(), 1);
    }

    #[test]
    fn int_constructor_str() {
        let mut vm : Vm = eval!("
y = Int('10')
");
        assert_eq!(vm.global().get("y").unwrap().unwrap().int(), 10);
    }

    #[test]
    fn int_chr() {
        let mut vm : Vm = eval!("
y = (97).chr()
");
        assert_eq!(vm.global().get("y").unwrap().unwrap().string(), "a");
    }
    // #end

    // #region float
    #[test]
    fn float_constructor() {
        let mut vm : Vm = eval!("
y = Float(1)
");
        assert_eq!(vm.global().get("y").unwrap().unwrap().float(), 1.0);
    }

    #[test]
    fn float_constructor_str() {
        let mut vm : Vm = eval!("
y = Float('10.55')
");
        assert_eq!(vm.global().get("y").unwrap().unwrap().float(), 10.55);
    }
    // #end


    // #region array
    #[test]
    fn array_constructor_no_args() {
        let mut vm : Vm = eval!("
y = Array()
");
        let arr = vm.global().get("y").unwrap().unwrap().array();
        assert_eq!(arr.len(), 0);
    }

    #[test]
    fn array_constructor() {
        let mut vm : Vm = eval!("
a = Array(1,2,3)
");
        let arr = vm.global().get("y").unwrap().unwrap().array();
        assert_eq!(arr.len(), 3);
        assert_eq!(arr[0].unwrap(), Value::Int(1));
        assert_eq!(arr[1].unwrap(), Value::Int(2));
        assert_eq!(arr[2].unwrap(), Value::Int(3));
    }

    #[test]
    fn array_length() {
        let mut vm : Vm = eval!("
y = [1,2,3].length()
");
        assert_eq!(vm.global().get("y").unwrap().unwrap().int(), 3);
    }

    #[test]
    fn array_delete() {
        let mut vm : Vm = eval!("
y = [1,2,3]
y.delete!(1,1)
");
        let arr = vm.global().get("y").unwrap().unwrap().array();
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0].unwrap(), Value::Int(1));
        assert_eq!(arr[1].unwrap(), Value::Int(3));
    }

    #[test]
    fn array_sort_in_place() {
        let mut vm : Vm = eval!("
y = [6,3,1]
y.sort!()
");
        let arr = vm.global().get("y").unwrap().unwrap().array();
        assert_eq!(arr.len(), 3);
        assert_eq!(arr[0].unwrap(), Value::Int(1));
        assert_eq!(arr[1].unwrap(), Value::Int(3));
        assert_eq!(arr[2].unwrap(), Value::Int(6));
    }

    #[test]
    fn array_sort() {
        let mut vm : Vm = eval!("
y = [6,3,1]
x = y.sort()
");
        let arr = vm.global().get("y").unwrap().unwrap().array();
        assert_eq!(arr.len(), 3);
        assert_eq!(arr[0].unwrap(), Value::Int(1));
        assert_eq!(arr[1].unwrap(), Value::Int(3));
        assert_eq!(arr[2].unwrap(), Value::Int(6));
    }

    #[test]
    fn array_push() {
        let mut vm : Vm = eval!("
y = []
y.push(10)
");
        let arr = vm.global().get("y").unwrap().unwrap().array();
        assert_eq!(arr.len(), 1);
        assert_eq!(arr[0].unwrap(), Value::Int(10));
    }

    #[test]
    fn array_pop() {
        let mut vm : Vm = eval!("
a = [1,2]
a.pop()
");
        let arr = vm.global().get("y").unwrap().unwrap().array();
        assert_eq!(arr.len(), 1);
        assert_eq!(arr[0].unwrap(), Value::Int(1));
    }

    #[test]
    fn array_index() {
        let mut vm : Vm = eval!("
a = ['a', 'b', 'c']
y = a.index('b')
");
        assert_eq!(vm.global().get("y").unwrap().unwrap(), Value::Int(1));
    }

    #[test]
    fn array_insert() {
        let mut vm : Vm = eval!("
a = [1,2,3]
a.insert!(1, 4)
");
        let arr = vm.global().get("y").unwrap().unwrap().array();
        assert_eq!(arr.len(), 4);
        assert_eq!(arr[0].unwrap(), Value::Int(1));
        assert_eq!(arr[1].unwrap(), Value::Int(4));
        assert_eq!(arr[2].unwrap(), Value::Int(2));
        assert_eq!(arr[3].unwrap(), Value::Int(3));
    }

    #[test]
    fn array_map() {
        let mut vm : Vm = eval!("
a=[3,5,64,2]
y = a.map(f(x) = x+1)
");
        let arr = vm.global().get("y").unwrap().unwrap().array();
        assert_eq!(arr.len(), 4);
        assert_eq!(arr[0].unwrap(), Value::Int(4));
        assert_eq!(arr[1].unwrap(), Value::Int(6));
        assert_eq!(arr[2].unwrap(), Value::Int(65));
        assert_eq!(arr[3].unwrap(), Value::Int(3));
    }
    #[test]
    fn array_map_native() {
        let mut vm : Vm = eval!("
a=['1','2','3']
y = a.map(Int)
");
        let arr = vm.global().get("y").unwrap().unwrap().array();
        assert_eq!(arr.len(), 3);
        assert_eq!(arr[0].unwrap(), Value::Int(1));
        assert_eq!(arr[1].unwrap(), Value::Int(2));
        assert_eq!(arr[2].unwrap(), Value::Int(3));
    }

    #[test]
    fn array_filter() {
        let mut vm : Vm = eval!("
a=[3,5,64,2]
y = a.filter(f(x) = x>5)
");
        let arr = vm.global().get("y").unwrap().unwrap().array();
        assert_eq!(arr.len(), 1);
        assert_eq!(arr[0].unwrap(), Value::Int(64));
    }

    #[test]
    fn array_reduce() {
        let mut vm : Vm = eval!("
a=[1,2,3,4,5]
y = a.reduce(f(x, y) = x + y, 0)
");
        assert_eq!(vm.global().get("y").unwrap().unwrap(), Value::Int(15));
    }

    #[test]
    fn array_chained_functional() { // TODO: MEMLEAK HERE
        let mut vm : Vm = eval!("
a=[1,2,3,5,6]
y = a.map(f(x) = x+1).filter(f(x) = x>5).reduce(f(prev, curr) = prev+curr, 0)
");
        assert_eq!(vm.global().get("y").unwrap().unwrap(), Value::Int(13));
    }

    #[test]
    fn array_join() {
        let mut vm : Vm = eval!("
a=[1,2,3,4,5,6]
y = a.join('')
");
        assert_eq!(vm.global().get("y").unwrap().unwrap().string(), "123456");
    }
    // #endregion

    // #region string
    #[test]
    fn string_constructor_no_args() {
        let mut vm : Vm = eval!("
y = String()
");
        assert_eq!(vm.global().get("y").unwrap().unwrap().string(), "");
    }

    #[test]
    fn string_constructor() {
        let mut vm : Vm = eval!("
y = String(10)
");
        assert_eq!(vm.global().get("y").unwrap().unwrap().string(), "10");
    }

    #[test]
    fn string_length() {
        let mut vm : Vm = eval!("
y = '日本'.length()
");
        assert_eq!(vm.global().get("y").unwrap().unwrap().int(), 2);
    }

    #[test]
    fn string_bytesize() {
        let mut vm : Vm = eval!("
y = '日本'.bytesize()
");
        assert_eq!(vm.global().get("y").unwrap().unwrap().int(), 6);
    }

    #[test]
    fn string_startswith() {
        let mut vm : Vm = eval!("
y = 'abc'.startswith?('a')
");
        assert_eq!(vm.global().get("y").unwrap().unwrap().int(), 1);
    }

    #[test]
    fn string_endswith() {
        let mut vm : Vm = eval!("
y = 'abc'.endswith?('bc')
");
        assert_eq!(vm.global().get("y").unwrap().unwrap().int(), 1);
    }

    #[test]
    fn string_delete() {
        let mut vm : Vm = eval!("
s = 'Honest Abe Lincoln'
y = s.delete(7, 4)
");
        assert_eq!(vm.global().get("y").unwrap().unwrap().string(), "Honest Lincoln");
    }
    #[test]
    fn string_delete_in_place() {
        let mut vm : Vm = eval!("
s = 'Honest Abe Lincoln'
s.delete!(7, 4)
");
        assert_eq!(vm.global().get("s").unwrap().unwrap().string(), "Honest Lincoln");
    }

    #[test]
    fn string_copy() {
        let mut vm : Vm = eval!("
s = 'Honest Abe Lincoln'
y = s.copy(7, 3)
");
        assert_eq!(vm.global().get("y").unwrap().unwrap().string(), "Abe");
    }

    #[test]
    fn string_index() {
        let mut vm : Vm = eval!("
s = 'Honest Abe Lincoln'
y = s.index('Lincoln')
");
        assert_eq!(vm.global().get("y").unwrap().unwrap(), Value::Int(11));
    }

    #[test]
    fn string_insert() {
        let mut vm : Vm = eval!("
s = 'Honest Abe Lincoln'
s.insert!(0, 'Not So ')
");
        assert_eq!(vm.global().get("s").unwrap().unwrap().string(), "Not So Honest Abe Lincoln");
    }

    #[test]
    fn string_split() {
        let mut vm : Vm = eval!("
s = 'a b c'
y = s.split(' ')
");
        let arr = vm.global().get("y").unwrap().unwrap().array();
        assert_eq!(arr.len(), 3);
        assert_eq!(arr[0].unwrap().string(), "a");
        assert_eq!(arr[1].unwrap().string(), "b");
        assert_eq!(arr[2].unwrap().string(), "c");
    }

    #[test]
    fn string_chars() {
        let mut vm : Vm = eval!("
s = 'abc'
y = s.chars()
");
        let arr = vm.global().get("y").unwrap().unwrap().array();
        assert_eq!(arr.len(), 3);
        assert_eq!(arr[0].unwrap().string(), "a");
        assert_eq!(arr[1].unwrap().string(), "b");
        assert_eq!(arr[2].unwrap().string(), "c");
    }

    #[test]
    fn string_ord() {
        let mut vm : Vm = eval!("
s = 'a'
y = s.ord()
");
        assert_eq!(vm.global().get("y").unwrap().unwrap().int(), 97);
    }
    // #endregion

    // #region record
    #[test]
    fn record_new() {
        let mut vm : Vm = eval!("
y = Record()
");
        vm.global().get("y").unwrap().unwrap().record();
    }

    #[test]
    fn record_keys() {
        let mut vm : Vm = eval!("
record x
    a = 10
    b = 10
end
y = Record::keys(x)
");
        let arr = vm.global().get("y").unwrap().unwrap().array();
        assert_eq!(arr.len(), 2);
    }
    // #endregion

    // #region env
    #[test]
    fn env_get() {
        std::env::set_var("test_key", "value");
        let mut vm : Vm = eval!("
y = Env::get('test_key')
");
        assert_eq!(vm.global().get("y").unwrap().unwrap().string(), "value");
    }

    #[test]
    fn env_set() {
        eval!("
Env::set('test_key_set', 'value')
");
        assert_eq!(std::env::var("test_key_set").unwrap(), "value");
    }

    #[test]
    fn env_vars() {
        std::env::set_var("a_key", "value");
        let mut vm : Vm = eval!("
y = Env::vars()['a_key']
");
        assert_eq!(vm.global().get("y").unwrap().unwrap().string(), "value");
    }
    // #endregion

    // #region files
    #[test]
    fn file_read() {
        std::fs::write("/tmp/file_read", "test");
        let mut vm : Vm = eval!("
f = File('/tmp/file_read', 'r')
y = f.read()
f.close()
");
        assert_eq!(vm.global().get("y").unwrap().unwrap().string(), "test");
    }

    #[test]
    fn file_read_up_to() {
        std::fs::write("/tmp/file_read_up_to", "test");
        let mut vm : Vm = eval!("
f = File('/tmp/file_read_up_to', 'r')
y = f.read_up_to(2)
");
        assert_eq!(vm.global().get("y").unwrap().unwrap().string(), "te");
    }

    #[test]
    fn file_write() {
        eval!("
f = File('/tmp/file_write', 'wc')
f.write('Hello World')
f.close()
");
        assert_eq!(std::str::from_utf8(&std::fs::read("/tmp/file_write").unwrap()).unwrap(), "Hello World");
    }
    // #endregion

    // #region cmd
    #[test]
    fn cmd_constructor_array() {
        let mut vm : Vm = eval!("
y = Cmd(['echo', 'hello world']).out()
");
        assert_eq!(vm.global().get("y").unwrap().unwrap().string(), "hello world\n");
    }

    #[test]
    fn cmd_constructor_string() {
        let mut vm : Vm = eval!("
y = Cmd('echo hello world').out()
");
        assert_eq!(vm.global().get("y").unwrap().unwrap().string(), "hello world\n");
    }

    #[test]
    fn cmd_err() {
        let mut vm : Vm = eval!("
y = Cmd('echo hello world >&2').err()
");
        assert_eq!(vm.global().get("y").unwrap().unwrap().string(), "hello world\n");
    }

    #[test]
    fn cmd_in() {
        let mut vm : Vm = eval!("
y = Cmd('cat -').in('nyaaa').out()
");
        assert_eq!(vm.global().get("y").unwrap().unwrap().string(), "nyaaa");
    }
    // #endregion

    // #region math
    #[test]
    fn math_sqrt() {
        let mut vm : Vm = eval!("
y = sqrt(4.0)
");
        assert_eq!(vm.global().get("y").unwrap().unwrap(), Value::Float(2.0));
    }
    // #endregion

    // #region other
    #[test]
    fn eval() {
        let mut vm : Vm = eval!("
eval('y = 10')
");
        assert_eq!(vm.global().get("y").unwrap().unwrap(), Value::Int(10));
    }
    // #endregion

}