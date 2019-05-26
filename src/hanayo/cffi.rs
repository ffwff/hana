//! Provides Cffi record for interfacing with C functions
extern crate libffi_sys;
use libffi_sys::*;
use std::ptr::{null, null_mut};
use libc::c_void;
use std::ffi::{CString, CStr};
use crate::vmbindings::vm::Vm;
use crate::vmbindings::vmerror::VmError;
use crate::vmbindings::value::Value;
use crate::vmbindings::carray::CArray;
use crate::vmbindings::record::Record;

// #region ffi type
#[repr(i64)]
enum FFI_Type {
    UInt8,   Int8,
    UInt16,  Int16,
    UInt32,  Int32,
    UInt64,  Int64,
    Float32, Float64,
    Pointer,
    String,
    Void
}

impl FFI_Type {
    fn from_i64(value: i64) -> Option<Self> {
        match value {
            0  => Some(FFI_Type::UInt8),
            1  => Some(FFI_Type::Int8),
            2  => Some(FFI_Type::UInt16),
            3  => Some(FFI_Type::Int16),
            4  => Some(FFI_Type::UInt32),
            5  => Some(FFI_Type::Int32),
            6  => Some(FFI_Type::UInt64),
            7  => Some(FFI_Type::Int64),
            8  => Some(FFI_Type::Float32),
            9  => Some(FFI_Type::Float64),
            10 => Some(FFI_Type::Pointer),
            11 => Some(FFI_Type::String),
            12 => Some(FFI_Type::Void),
            _ => None
        }
    }

    unsafe fn to_libffi_type(&self) -> *mut ffi_type {
        match self {
            FFI_Type::UInt8   => &mut ffi_type_uint8,
            FFI_Type::Int8    => &mut ffi_type_sint8,
            FFI_Type::UInt16  => &mut ffi_type_uint16,
            FFI_Type::Int16   => &mut ffi_type_sint16,
            FFI_Type::UInt32  => &mut ffi_type_uint32,
            FFI_Type::Int32   => &mut ffi_type_sint32,
            FFI_Type::UInt64  => &mut ffi_type_uint64,
            FFI_Type::Int64   => &mut ffi_type_sint64,
            FFI_Type::Float32 => &mut ffi_type_float,
            FFI_Type::Float64 => &mut ffi_type_double,
            FFI_Type::Pointer => &mut ffi_type_pointer,
            FFI_Type::String  => &mut ffi_type_pointer,
            FFI_Type::Void    => &mut ffi_type_void
        }
    }
}
// #endregion

struct FFIFunction {
    sym: Option<unsafe extern fn()>,
    cif: ffi_cif,
    argtypes: CArray<FFI_Type>,
    ffi_argtypes: CArray<*mut ffi_type>,
    rettype: FFI_Type,
    ffi_rettype: *mut ffi_type,
}

pub mod function {

use super::*;

#[hana_function()]
fn constructor(name_or_addr: Value::Any, argtypes: Value::Array, rettype: Value::Int) -> Value {
    // argtypes
    let argtypes = argtypes.as_ref();
    let mut inst_argtypes = CArray::new();
    let mut ffi_argtypes : CArray<*mut ffi_type> = CArray::new();
    for arg in argtypes.iter() {
        let ffi_type = FFI_Type::from_i64(arg.unwrap().int()).unwrap();
        ffi_argtypes.push(unsafe{ ffi_type.to_libffi_type() });
        inst_argtypes.push(ffi_type);
    }

    // rettype
    let rettype = FFI_Type::from_i64(rettype).unwrap();

    // create
    let ffi_fn = unsafe {
        // cif
        let mut cif: ffi_cif = Default::default();
        let ffi_rettype = unsafe{ rettype.to_libffi_type() };
        eprintln!("{:?}", ffi_argtypes);
        ffi_prep_cif(&mut cif, ffi_abi_FFI_DEFAULT_ABI,
            argtypes.len() as u32,
            ffi_rettype,
            ffi_argtypes.as_mut_ptr());

        // ffi fn
        let dl = libc::dlopen(null(), libc::RTLD_LAZY);
        FFIFunction {
            sym: {
                match &name_or_addr {
                    Value::Int(addr) => {
                        //Some(std::mem::transmute::<*mut c_void, unsafe extern fn()>(debug as *mut libc::c_void))
                        Some(std::mem::transmute::<i64, unsafe extern fn()>(*addr))
                    }
                    Value::Str(sym) => {
                        let cstr = CString::new(sym.as_ref().clone()).unwrap();
                        let sym = libc::dlsym(dl, cstr.as_c_str().as_ptr());
                        if sym.is_null() { panic!("is nul!") }
                        else { Some(std::mem::transmute::<*mut c_void, unsafe extern fn()>(sym)) }
                    }
                    _ => panic!("expected symbol address or name")
                }
            },
            cif,
            argtypes: inst_argtypes,
            ffi_argtypes,
            rettype,
            ffi_rettype
        }
    };

    let rec = vm.malloc(Record::new());
    rec.as_mut().native_field = Some(Box::new(ffi_fn));
    rec.as_mut().insert("prototype",
        vm.global().get("Cffi").unwrap().unwrap().record().get("Function").unwrap().clone());
    Value::Record(rec)
}

#[hana_function()]
fn call(ffi_fn_rec: Value::Record, args: Value::Array) {
    let field = ffi_fn_rec.as_mut().native_field.as_mut().unwrap();
    let mut ffi_fn = field.downcast_mut::<FFIFunction>().unwrap();

    use libc::c_void;
    use std::any::Any;
    use std::convert::TryInto;
    use std::mem::transmute;

    let mut managed_strs : Vec<Box<CStr>> = Vec::new();
    let mut aref : CArray<*mut c_void> = CArray::new();
    let mut argtypes_iter = ffi_fn.argtypes.iter();
    let slice = args.as_mut().as_mut_slice();
    for arg in slice {
        let argtype = &argtypes_iter.next().unwrap();
        unsafe {
            let ptr = transmute::<&u64, *mut c_void>(&arg.data);
            #[allow(safe_packed_borrows)]
            match argtype {
                FFI_Type::String => {
                    let cstr : Box<CStr> = CString::new(arg.unwrap().string().clone()).unwrap().into_boxed_c_str();
                    aref.push(transmute::<*const *const libc::c_char, *mut c_void>(&cstr.as_ptr()));
                    managed_strs.push(cstr);
                },
                // primitive types
                _ => { aref.push(transmute::<&mut u64, *mut c_void>(&mut arg.data)); },
            }
        }
    }

    unsafe { match &ffi_fn.rettype {
        FFI_Type::UInt8 | FFI_Type::Int8 |
        FFI_Type::UInt16 | FFI_Type::Int16 |
        FFI_Type::UInt32 | FFI_Type::Int32 |
        FFI_Type::UInt64 | FFI_Type::Int64 |
        FFI_Type::Pointer
            => {
                let mut rvalue = 0;
                ffi_call(&mut ffi_fn.cif, ffi_fn.sym, transmute::<&i64, *mut c_void>(&rvalue), aref.as_mut_ptr());
                Value::Int(rvalue)
            },
        FFI_Type::Float32
            => {
                let mut rvalue = 0.0f32;
                ffi_call(&mut ffi_fn.cif, ffi_fn.sym, transmute::<&f32, *mut c_void>(&rvalue), aref.as_mut_ptr());
                Value::Float(rvalue as f64)
            },
        FFI_Type::Float64
            => {
                let mut rvalue = 0.0f64;
                ffi_call(&mut ffi_fn.cif, ffi_fn.sym, transmute::<&f64, *mut c_void>(&rvalue), aref.as_mut_ptr());
                Value::Float(rvalue)
            },
        FFI_Type::String
            => {
                let mut rvalue : *const libc::c_char = null_mut();
                ffi_call(&mut ffi_fn.cif, ffi_fn.sym, transmute::<&*const libc::c_char, *mut c_void>(&rvalue), aref.as_mut_ptr());
                assert!(!rvalue.is_null());
                Value::Str(vm.malloc(CStr::from_ptr(rvalue).to_str().unwrap().to_string()))
            },
        FFI_Type::Void
            => {
                ffi_call(&mut ffi_fn.cif, ffi_fn.sym, null_mut(), aref.as_mut_ptr());
                Value::Nil
            }
    } }
}

}

pub mod gc_pointer {

use super::*;

struct GcPointer {
    data: *mut libc::c_void,
    free: unsafe extern fn(*mut libc::c_void),
}

impl std::ops::Drop for GcPointer {

    fn drop(&mut self) {
        unsafe{ (self.free)(self.data) }
    }

}

#[hana_function()]
fn constructor(addr: Value::Int, cffi_free: Value::Record) -> Value {
    let field = cffi_free.as_mut().native_field.as_mut().unwrap();
    let mut cffi_free = field.downcast_mut::<FFIFunction>().unwrap();

    let rec = vm.malloc(Record::new());
    unsafe{
        use std::mem::transmute;
        rec.as_mut().native_field = Some(Box::new(GcPointer {
            data: transmute::<i64, *mut libc::c_void>(addr),
            free: transmute::<*mut libc::c_void, unsafe extern fn(*mut libc::c_void)>(cffi_free.sym.unwrap() as *mut libc::c_void),
        }));
    }
    rec.as_mut().insert("prototype",
        vm.global().get("Cffi").unwrap().unwrap().record().get("GcPointer").unwrap().clone());
    Value::Record(rec)
}

#[hana_function()]
fn addr(pointer: Value::Record) -> Value {
    let field = pointer.as_mut().native_field.as_mut().unwrap();
    let mut gc_pointer = field.downcast_mut::<GcPointer>().unwrap();
    Value::Int(unsafe { std::mem::transmute::<*mut libc::c_void, i64>(gc_pointer.data) })
}

}

pub mod library {

use super::*;

struct Library {
    dl: *mut libc::c_void,
}

impl std::ops::Drop for Library {

    fn drop(&mut self) {
        unsafe {
            libc::dlclose(self.dl);
        }
    }

}

#[hana_function()]
fn constructor(filename: Value::Str) -> Value {
    let rec = vm.malloc(Record::new());
    unsafe {
        rec.as_mut().native_field = Some(Box::new({
            let cstr = CString::new(filename.as_ref().clone()).unwrap();
            let dl = libc::dlopen(cstr.as_ptr(), libc::RTLD_LAZY);
            Library {
                dl
            }
        }));
    }
    rec.as_mut().insert("prototype",
        vm.global().get("Cffi").unwrap().unwrap().record().get("Library").unwrap().clone());
    Value::Record(rec)
}

#[hana_function()]
fn sym(library: Value::Record, sym: Value::Str) -> Value {
    let field = library.as_mut().native_field.as_mut().unwrap();
    let mut dl = field.downcast_mut::<Library>().unwrap();
    unsafe {
        let cstr = CString::new(sym.as_ref().clone()).unwrap();
        let sym = libc::dlsym(dl.dl, cstr.as_c_str().as_ptr());
        Value::Int(sym as i64)
    }
}

}

// exports
pub fn load(vm: &mut Vm) {
    macro_rules! set_var {
        ($x:literal, $y:expr) => (vm.mut_global().insert($x.to_string(), $y.wrap()));
    }
    macro_rules! set_obj_var {
        ($o: expr, $x:literal, $y:expr) => ($o.as_mut().insert($x.to_string(), $y.wrap()));
    }

    let cffi_mod = vm.malloc(Record::new());

    // types
    set_obj_var!(cffi_mod, "UInt8",   Value::Int(FFI_Type::UInt8 as i64));
    set_obj_var!(cffi_mod, "Int8",    Value::Int(FFI_Type::Int8 as i64));
    set_obj_var!(cffi_mod, "UInt16",  Value::Int(FFI_Type::UInt16 as i64));
    set_obj_var!(cffi_mod, "Int16",   Value::Int(FFI_Type::Int16 as i64));
    set_obj_var!(cffi_mod, "UInt32",  Value::Int(FFI_Type::UInt32 as i64));
    set_obj_var!(cffi_mod, "Int32",   Value::Int(FFI_Type::Int32 as i64));
    set_obj_var!(cffi_mod, "UInt64",  Value::Int(FFI_Type::UInt64 as i64));
    set_obj_var!(cffi_mod, "Int64",   Value::Int(FFI_Type::Int64 as i64));
    set_obj_var!(cffi_mod, "Float32", Value::Int(FFI_Type::Float32 as i64));
    set_obj_var!(cffi_mod, "Float64", Value::Int(FFI_Type::Float64 as i64));
    set_obj_var!(cffi_mod, "Pointer", Value::Int(FFI_Type::Pointer as i64));
    set_obj_var!(cffi_mod, "String",  Value::Int(FFI_Type::String as i64));
    set_obj_var!(cffi_mod, "Void",    Value::Int(FFI_Type::Void as i64));

    // dynamically loaded library
    let library = vm.malloc(Record::new());
    set_obj_var!(library, "constructor", Value::NativeFn(library::constructor));
    set_obj_var!(library, "sym",         Value::NativeFn(library::sym));
    set_obj_var!(cffi_mod, "Library", Value::Record(library));

    // function
    let function = vm.malloc(Record::new());
    set_obj_var!(function, "constructor", Value::NativeFn(function::constructor));
    set_obj_var!(function, "call",        Value::NativeFn(function::call));
    set_obj_var!(cffi_mod, "Function", Value::Record(function));

    // garbage collected pointer
    let gc_pointer = vm.malloc(Record::new());
    set_obj_var!(gc_pointer, "constructor", Value::NativeFn(gc_pointer::constructor));
    set_obj_var!(gc_pointer, "addr",        Value::NativeFn(gc_pointer::addr));
    set_obj_var!(cffi_mod, "GcPointer", Value::Record(gc_pointer));

    set_var!("Cffi", Value::Record(cffi_mod));
}