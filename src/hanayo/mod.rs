//! Standard library implementation for the language.

use crate::vmbindings::gc::Gc;
use crate::vmbindings::record::Record;
use crate::vmbindings::value::*;
use crate::vmbindings::vm::Vm;
use crate::vmbindings::vmerror::VmError;

pub mod cmd;
pub mod env;
pub mod eval;
pub mod file;
pub mod io;
pub mod math;
pub mod proc;
pub mod time;
cfg_if! {
    if #[cfg(feature="cffi")] {
        pub mod cffi;
        use cffi::load as cffi_load;
    } else {
        fn cffi_load(_vm: &mut Vm) {}
    }
}

pub mod array;
pub mod float;
pub mod int;
pub mod record;
pub mod string;

/// Standard library context
pub struct HanayoCtx {
    pub file_rec: Gc<Record>,
    pub cmd_rec: Gc<Record>,
    pub proc_rec: Gc<Record>,
    pub time_rec: Gc<Record>,
}

/// Initialises hanayo for the virtual machine
pub fn init(vm: &mut Vm) {
    macro_rules! set_var {
        ($x:literal, $y:expr) => {
            vm.mut_global().insert($x.to_string(), $y.wrap())
        };
    }
    macro_rules! set_obj_var {
        ($o: expr, $x:literal, $y:expr) => {
            $o.as_mut().insert($x.to_string(), $y.wrap())
        };
    }
    // constants
    set_var!("nil", Value::Nil);
    set_var!("true", Value::Int(1));
    set_var!("false", Value::Int(0));
    set_var!("inf", Value::Float(std::f64::INFINITY));
    set_var!("nan", Value::Float(std::f64::NAN));

    // builtin functions
    set_var!("print", Value::NativeFn(io::print));
    set_var!("input", Value::NativeFn(io::input));
    set_var!("exit", Value::NativeFn(io::exit));
    set_var!("eval", Value::NativeFn(eval::eval));

    // maths
    set_var!("sqrt", Value::NativeFn(math::sqrt));

    // #region array
    {
        let array = vm.malloc(Record::new());
        set_obj_var!(array, "constructor", Value::NativeFn(array::constructor));
        set_obj_var!(array, "length", Value::NativeFn(array::length));
        set_obj_var!(array, "insert!", Value::NativeFn(array::insert_));
        set_obj_var!(array, "delete!", Value::NativeFn(array::delete_));
        set_obj_var!(array, "push", Value::NativeFn(array::push));
        set_obj_var!(array, "pop", Value::NativeFn(array::pop));
        set_obj_var!(array, "sort", Value::NativeFn(array::sort));
        set_obj_var!(array, "sort!", Value::NativeFn(array::sort_));
        set_obj_var!(array, "map", Value::NativeFn(array::map));
        set_obj_var!(array, "filter", Value::NativeFn(array::filter));
        set_obj_var!(array, "reduce", Value::NativeFn(array::reduce));
        set_obj_var!(array, "index", Value::NativeFn(array::index));
        set_obj_var!(array, "join", Value::NativeFn(array::join));
        vm.darray = array.clone();
        set_var!("Array", Value::Record(array));
    }
    // #endregion

    // #region string
    {
        let string = vm.malloc(Record::new());
        set_obj_var!(string, "constructor", Value::NativeFn(string::constructor));
        set_obj_var!(string, "length", Value::NativeFn(string::length));
        set_obj_var!(string, "bytesize", Value::NativeFn(string::bytesize));
        set_obj_var!(string, "startswith?", Value::NativeFn(string::startswith));
        set_obj_var!(string, "endswith?", Value::NativeFn(string::endswith));
        set_obj_var!(string, "delete", Value::NativeFn(string::delete));
        set_obj_var!(string, "delete!", Value::NativeFn(string::delete_));
        set_obj_var!(string, "copy", Value::NativeFn(string::copy));
        set_obj_var!(string, "insert!", Value::NativeFn(string::insert_));
        set_obj_var!(string, "split", Value::NativeFn(string::split));
        set_obj_var!(string, "index", Value::NativeFn(string::index));
        set_obj_var!(string, "chars", Value::NativeFn(string::chars));
        set_obj_var!(string, "ord", Value::NativeFn(string::ord));
        vm.dstr = string.clone();
        set_var!("String", Value::Record(string));
    }
    // #endregion

    // #region int
    {
        let int = vm.malloc(Record::new());
        set_obj_var!(int, "constructor", Value::NativeFn(int::constructor));
        set_obj_var!(int, "chr", Value::NativeFn(int::chr));
        set_obj_var!(int, "hex", Value::NativeFn(int::hex));
        vm.dint = int.clone();
        set_var!("Int", Value::Record(int));
    }
    // #endregion

    // #region float
    {
        let float = vm.malloc(Record::new());
        set_obj_var!(float, "constructor", Value::NativeFn(float::constructor));
        vm.dfloat = float.clone();
        set_var!("Float", Value::Record(float));
    }
    // #endregion

    // #region record
    {
        let record = vm.malloc(Record::new());
        set_obj_var!(record, "constructor", Value::NativeFn(record::constructor));
        set_obj_var!(record, "keys", Value::NativeFn(record::keys));
        vm.drec = record.clone();
        set_var!("Record", Value::Record(record));
    }
    // #endregion

    // #region files
    let file = vm.malloc(Record::new());
    set_obj_var!(file, "constructor", Value::NativeFn(file::constructor));
    set_obj_var!(file, "close", Value::NativeFn(file::close));
    set_obj_var!(file, "read", Value::NativeFn(file::read));
    set_obj_var!(file, "read_up_to", Value::NativeFn(file::read_up_to));
    set_obj_var!(file, "write", Value::NativeFn(file::write));
    set_obj_var!(file, "seek", Value::NativeFn(file::seek));
    set_obj_var!(
        file,
        "seek_from_start",
        Value::NativeFn(file::seek_from_start)
    );
    set_obj_var!(file, "seek_from_end", Value::NativeFn(file::seek_from_end));
    set_var!("File", Value::Record(file.clone()));
    // #endregion

    // #region cmd
    let cmd = vm.malloc(Record::new());
    set_obj_var!(cmd, "constructor", Value::NativeFn(cmd::constructor));
    set_obj_var!(cmd, "in", Value::NativeFn(cmd::in_));
    set_obj_var!(cmd, "out", Value::NativeFn(cmd::out));
    set_obj_var!(cmd, "err", Value::NativeFn(cmd::err));
    set_obj_var!(cmd, "outputs", Value::NativeFn(cmd::outputs));
    set_obj_var!(cmd, "spawn", Value::NativeFn(cmd::spawn));
    set_var!("Cmd", Value::Record(cmd.clone()));
    // #endregion

    // #region cmd
    let proc = vm.malloc(Record::new());
    set_obj_var!(proc, "in", Value::NativeFn(proc::in_));
    set_obj_var!(proc, "out", Value::NativeFn(proc::out));
    set_obj_var!(proc, "err", Value::NativeFn(proc::err));
    set_obj_var!(proc, "outputs", Value::NativeFn(proc::outputs));
    set_obj_var!(proc, "wait", Value::NativeFn(proc::wait));
    set_obj_var!(proc, "kill", Value::NativeFn(proc::kill));
    set_var!("Process", Value::Record(proc.clone()));
    // #endregion

    // #region env
    let env = vm.malloc(Record::new());
    set_obj_var!(env, "get", Value::NativeFn(env::get));
    set_obj_var!(env, "set", Value::NativeFn(env::set));
    set_obj_var!(env, "vars", Value::NativeFn(env::vars));
    set_var!("Env", Value::Record(env.clone()));
    // #endregion

    // #region time
    let time = vm.malloc(Record::new());
    set_obj_var!(time, "constructor", Value::NativeFn(time::constructor));
    set_obj_var!(time, "sleep", Value::NativeFn(time::sleep));
    set_obj_var!(time, "since", Value::NativeFn(time::since));
    set_obj_var!(time, "secs", Value::NativeFn(time::secs));
    set_obj_var!(time, "millis", Value::NativeFn(time::millis));
    set_obj_var!(time, "micros", Value::NativeFn(time::micros));
    set_obj_var!(time, "nanos", Value::NativeFn(time::nanos));
    set_var!("Time", Value::Record(time.clone()));
    // #endregion

    cffi_load(vm);

    vm.stdlib = Some(HanayoCtx {
        file_rec: file,
        cmd_rec: cmd,
        proc_rec: proc,
        time_rec: time,
    });
}
