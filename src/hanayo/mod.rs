use crate::vmbindings::vm::Vm;
use crate::vmbindings::vmerror::VmError;
use crate::vmbindings::record::Record;
use crate::vmbindings::value::*;
use crate::vmbindings::gc::{Gc, pin};

mod io;
mod file;
mod cmd;
mod env;
mod eval;
mod math;

mod array;
mod string;
mod int;
mod float;
mod record;

pub fn init(vm : &mut Vm) {
    let globalenv = unsafe { &mut *vm.globalenv };
    macro_rules! set_var {
        ($x:literal, $y:expr) => (globalenv.insert($x.to_string(), $y.wrap()));
    }
    macro_rules! set_obj_var {
        ($o: expr, $x:literal, $y:expr) => ($o.as_mut().insert($x.to_string(), $y.wrap()));
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
    set_var!("eval", Value::NativeFn(eval::eval));

    // maths
    set_var!("sqrt", Value::NativeFn(math::sqrt));

    // #region array
    {
    let mut array = Gc::new(Record::new());
    set_obj_var!(array, "constructor", Value::NativeFn(array::constructor));
    set_obj_var!(array, "length",      Value::NativeFn(array::length));
    set_obj_var!(array, "insert!",     Value::NativeFn(array::insert_));
    set_obj_var!(array, "delete!",     Value::NativeFn(array::delete_));
    set_obj_var!(array, "push",        Value::NativeFn(array::push));
    set_obj_var!(array, "pop",         Value::NativeFn(array::pop));
    set_obj_var!(array, "sort",        Value::NativeFn(array::sort));
    set_obj_var!(array, "sort!",       Value::NativeFn(array::sort_));
    set_obj_var!(array, "map",         Value::NativeFn(array::map));
    set_obj_var!(array, "filter",      Value::NativeFn(array::filter));
    set_obj_var!(array, "reduce",      Value::NativeFn(array::reduce));
    set_obj_var!(array, "index",       Value::NativeFn(array::index));
    set_obj_var!(array, "join",        Value::NativeFn(array::join));
    vm.darray = array.to_mut_raw(); pin(vm.darray as *mut libc::c_void);
    set_var!("Array", Value::Record(array));
    }
    // #endregion

    // #region string
    {
    let mut string = Gc::new(Record::new());
    set_obj_var!(string, "constructor", Value::NativeFn(string::constructor));
    set_obj_var!(string, "length",      Value::NativeFn(string::length));
    set_obj_var!(string, "bytesize",    Value::NativeFn(string::bytesize));
    set_obj_var!(string, "startswith?", Value::NativeFn(string::startswith));
    set_obj_var!(string, "endswith?",   Value::NativeFn(string::endswith));
    set_obj_var!(string, "delete",      Value::NativeFn(string::delete));
    set_obj_var!(string, "delete!",     Value::NativeFn(string::delete_));
    set_obj_var!(string, "copy",        Value::NativeFn(string::copy));
    set_obj_var!(string, "insert!",     Value::NativeFn(string::insert_));
    set_obj_var!(string, "split",       Value::NativeFn(string::split));
    set_obj_var!(string, "index",       Value::NativeFn(string::index));
    set_obj_var!(string, "chars",       Value::NativeFn(string::chars));
    set_obj_var!(string, "ord",         Value::NativeFn(string::ord));
    vm.dstr = string.to_mut_raw(); pin(vm.dstr as *mut libc::c_void);
    set_var!("String", Value::Record(string));
    }
    // #endregion

    // #region int
    {
    let mut int = Gc::new(Record::new());
    set_obj_var!(int, "constructor", Value::NativeFn(int::constructor));
    set_obj_var!(int, "chr",         Value::NativeFn(int::chr));
    vm.dint = int.to_mut_raw(); pin(vm.dint as *mut libc::c_void);
    set_var!("Int", Value::Record(int));
    }
    // #endregion

    // #region float
    {
    let mut float = Gc::new(Record::new());
    set_obj_var!(float, "constructor", Value::NativeFn(float::constructor));
    vm.dfloat = float.to_mut_raw(); pin(vm.dfloat as *mut libc::c_void);
    set_var!("Float", Value::Record(float));
    }
    // #endregion

    // #region record
    {
    let mut record = Gc::new(Record::new());
    set_obj_var!(record, "constructor", Value::NativeFn(record::constructor));
    set_obj_var!(record, "keys",        Value::NativeFn(record::keys));
    vm.drec = record.to_mut_raw(); pin(vm.drec as *mut libc::c_void);
    set_var!("Record", Value::Record(record));
    }
    // #endregion

    // #region files
    {
    let mut file = Gc::new(Record::new());
    set_obj_var!(file, "constructor", Value::NativeFn(file::constructor));
    set_obj_var!(file, "close",       Value::NativeFn(file::close));
    set_obj_var!(file, "read",        Value::NativeFn(file::read));
    set_obj_var!(file, "read_up_to",  Value::NativeFn(file::read_up_to));
    set_obj_var!(file, "write",       Value::NativeFn(file::write));
    set_obj_var!(file, "seek",        Value::NativeFn(file::seek));
    set_obj_var!(file, "seek_from_start", Value::NativeFn(file::seek_from_start));
    set_obj_var!(file, "seek_from_end",   Value::NativeFn(file::seek_from_end));
    set_var!("File", Value::Record(file));
    }
    // #endregion

    // #region cmd
    {
    let mut cmd = Gc::new(Record::new());
    set_obj_var!(cmd, "constructor",  Value::NativeFn(cmd::constructor));
    set_obj_var!(cmd, "in" ,          Value::NativeFn(cmd::in_));
    set_obj_var!(cmd, "out",          Value::NativeFn(cmd::out));
    set_obj_var!(cmd, "err",          Value::NativeFn(cmd::err));
    set_obj_var!(cmd, "outputs",      Value::NativeFn(cmd::outputs));
    set_var!("Cmd", Value::Record(cmd));
    }
    // #endregion

    // #region env
    {
    let mut env = Gc::new(Record::new());
    set_obj_var!(env, "get",  Value::NativeFn(env::get));
    set_obj_var!(env, "set",  Value::NativeFn(env::set));
    set_obj_var!(env, "vars", Value::NativeFn(env::vars));
    set_var!("Env", Value::Record(env));
    }
    // #endregion

}