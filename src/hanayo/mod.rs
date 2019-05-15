use crate::vmbindings::vm::Vm;
use crate::vmbindings::record::Record;
use crate::vmbindings::value::*;
use crate::vmbindings::gc::*;

mod io;
mod array;
mod string;
mod int;
mod float;

pub fn init(vm : &mut Vm) {
    let globalenv = unsafe { &mut *vm.globalenv };
    macro_rules! set_var {
        ($x:literal, $y:expr) => (globalenv.insert($x.to_string(), $y.wrap()));
    }
    macro_rules! set_obj_var {
        ($o: expr, $x:literal, $y:expr) => ($o.insert($x.to_string(), $y.wrap()));
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

    // builtin objects
    let rec_free = |ptr| unsafe{ drop::<Record>(ptr) };

    // #region array
    {
    let mut array : Record = Record::new();
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

    let ptr = unsafe { malloc(array, rec_free) };
    set_var!("Array", Value::Record(unsafe{ &*ptr }));
    vm.darray = ptr;
    }
    // #endregion

    // #region string
    {
    let mut string : Record = Record::new();
    set_obj_var!(string, "constructor", Value::NativeFn(string::constructor));
    set_obj_var!(string, "length",      Value::NativeFn(string::length));
    set_obj_var!(string, "bytesize",    Value::NativeFn(string::bytesize));
    set_obj_var!(string, "startswith",  Value::NativeFn(string::startswith));
    set_obj_var!(string, "endswith",    Value::NativeFn(string::endswith));
    set_obj_var!(string, "delete",      Value::NativeFn(string::delete));
    set_obj_var!(string, "delete!",     Value::NativeFn(string::delete_));
    set_obj_var!(string, "copy",        Value::NativeFn(string::copy));
    set_obj_var!(string, "insert!",     Value::NativeFn(string::insert_));
    set_obj_var!(string, "split",       Value::NativeFn(string::split));
    set_obj_var!(string, "index",       Value::NativeFn(string::index));

    let ptr = unsafe { malloc(string, rec_free) };
    set_var!("String", Value::Record(unsafe{ &*ptr }));
    vm.dstr = ptr;
    }
    // #endregion

    // #region int
    {
    let mut int : Record = Record::new();
    set_obj_var!(int, "constructor", Value::NativeFn(int::constructor));

    let ptr = unsafe { malloc(int, rec_free) };
    set_var!("Int", Value::Record(unsafe{ &*ptr }));
    vm.dint = ptr;
    }
    // #endregion

    // #region float
    {
    let mut float : Record = Record::new();
    set_obj_var!(float, "constructor", Value::NativeFn(float::constructor));

    let ptr = unsafe { malloc(float, rec_free) };
    set_var!("Float", Value::Record(unsafe{ &*ptr }));
    vm.dfloat = ptr;
    }
    // #endregion

    // #region record

    // #endregion

}