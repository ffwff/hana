mod io;
mod array;
mod string;
use crate::vmbindings::vm::Vm;
use crate::vmbindings::chmap::CHashMap;
use crate::vmbindings::value::*;
use crate::vmbindings::gc::malloc;

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
    { // array
    let mut array : CHashMap = std::collections::HashMap::new();
    set_obj_var!(array, "constructor", Value::NativeFn(array::constructor));
    set_obj_var!(array, "length",      Value::NativeFn(array::length));
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
    set_var!("Array", Value::Dict(unsafe{ &*Box::into_raw(Box::new(array)) }));
    }

    { // string
    let mut string : CHashMap = std::collections::HashMap::new();
    set_obj_var!(string, "constructor", Value::NativeFn(string::constructor));
    set_var!("Array", Value::Dict(unsafe{ &*Box::into_raw(Box::new(string)) }));
    }

}